use crate::session::ChannelSession;
use boa_engine::{
    context::ContextBuilder,
    job::{FutureJob, JobQueue, NativeJob},
    object::ObjectInitializer,
    property::Attribute,
    Context, JsData, JsResult, JsString, JsValue, NativeFunction, Source,
};
use boa_gc::{Finalize, Trace};
use futures_util::{stream::FuturesUnordered, StreamExt};
use sqlx::Row;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    path::Path,
    pin::Pin,
    rc::Rc,
    str::FromStr,
};
use tokio::task::{self, LocalSet};
use tokio_util::task::LocalPoolHandle;

pub struct PortalScriptEngine {}

impl PortalScriptEngine {
    pub async fn execute_script(name: String, session: &mut ChannelSession) {
        let proxy = PortalScriptProxy {
            handle: session.into(),
            script: name.clone(),
        };

        let pool = LocalPoolHandle::new(1);

        pool.spawn_pinned(|| async move {
            let script_path = format!("slate-channel/scripts/portal/{}.js", name.clone());
            let path = Path::new(&script_path);

            // Ensure the path exists -- many scripts aren't yet implemented
            if !path.exists() {
                log::warn!("Portal script {} doesn't exist", script_path);
                return;
            }

            let script = Source::from_filepath(path).unwrap();
            let job_queue = Rc::new(AsyncJobQueue::new());
            let mut context = ContextBuilder::new().job_queue(job_queue).build().unwrap();

            context.realm().clone().host_defined_mut().insert(proxy);

            let js_proxy = ObjectInitializer::new(&mut context)
                .function(
                    NativeFunction::from_async_fn(PortalScriptProxy::has_level_30_character),
                    JsString::from_str("hasLevel30Character").unwrap(),
                    0,
                )
                .function(
                    NativeFunction::from_fn_ptr(PortalScriptProxy::open_npc),
                    JsString::from_str("openNpc").unwrap(),
                    1,
                )
                .function(
                    NativeFunction::from_fn_ptr(PortalScriptProxy::block_portal),
                    JsString::from_str("blockPortal").unwrap(),
                    0,
                )
                .build();

            context
                .register_global_property(
                    JsString::from_str("pi").unwrap(),
                    js_proxy,
                    Attribute::all(),
                )
                .unwrap();

            context.eval(script).unwrap();
            context.eval(Source::from_bytes("enter(pi)")).unwrap();
            context.run_jobs_async().await;
        });
    }
}

#[derive(Trace, Finalize, JsData)]
struct PortalScriptProxy {
    handle: ChannelSessionHandle,
    script: String,
}

impl PortalScriptProxy {
    fn has_level_30_character(
        _this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> impl Future<Output = JsResult<JsValue>> {
        let realm = context.realm().clone();

        async move {
            log::debug!("hasLevel30Character()");

            let mut host_defined = realm.host_defined_mut();
            let proxy = host_defined.get_mut::<PortalScriptProxy>().unwrap();
            let session: &mut ChannelSession = proxy.handle.as_mut();

            log::debug!("Start query!");
            let levels = sqlx::query("SELECT level FROM characters WHERE account_id = ?")
                .bind(session.account_id.unwrap())
                .fetch_all(&session.db)
                .await
                .unwrap();

            log::debug!("End query!");

            for level in levels {
                if level.get::<i32, _>("level") >= 30 {
                    return Ok(JsValue::Boolean(true));
                }
            }

            Ok(JsValue::Boolean(false))
        }
    }

    fn open_npc(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(id) = args[0].as_number() {
            log::debug!("openNpc({})", id);
        }

        Ok(().into())
    }

    fn block_portal(
        _this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        log::debug!("blockPortal()");

        let realm = context.realm().clone();
        let mut host_defined = realm.host_defined_mut();
        let proxy = host_defined.get_mut::<PortalScriptProxy>().unwrap();
        let session: &mut ChannelSession = proxy.handle.as_mut();

        // Add the current portal's script name to blocked_portals
        session
            .character
            .as_mut()
            .unwrap()
            .blocked_portals
            .insert(proxy.script.clone());

        Ok(().into())
    }
}

#[derive(Trace, Finalize, JsData)]
struct ChannelSessionHandle(usize);

/// Converts a `ChannelSessionHandle` to an `&mut ChannelSession`
impl AsMut<ChannelSession> for ChannelSessionHandle {
    fn as_mut(&mut self) -> &mut ChannelSession {
        unsafe { std::mem::transmute(self.0) }
    }
}

/// Converts an `&mut ChannelSession` to a `ChannelSessionHandle`
impl From<&mut ChannelSession> for ChannelSessionHandle {
    fn from(session: &mut ChannelSession) -> Self {
        Self(unsafe { std::mem::transmute(session) })
    }
}

/// An event queue that also drives futures to completion.
struct AsyncJobQueue {
    futures: RefCell<FuturesUnordered<FutureJob>>,
    jobs: RefCell<VecDeque<NativeJob>>,
}

impl AsyncJobQueue {
    fn new() -> Self {
        Self {
            futures: RefCell::default(),
            jobs: RefCell::default(),
        }
    }
}

impl JobQueue for AsyncJobQueue {
    fn enqueue_promise_job(&self, job: NativeJob, _context: &mut Context) {
        self.jobs.borrow_mut().push_back(job);
    }

    fn enqueue_future_job(&self, future: FutureJob, _context: &mut Context) {
        self.futures.borrow().push(future);
    }

    fn run_jobs_async<'a, 'ctx, 'fut>(
        &'a self,
        context: &'ctx mut Context,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut>>
    where
        'a: 'fut,
        'ctx: 'fut,
    {
        Box::pin(async move {
            // Early return in case there were no jobs scheduled.
            if self.jobs.borrow().is_empty() && self.futures.borrow().is_empty() {
                return;
            }

            let context = RefCell::new(context);

            LocalSet::new()
                .run_until(async {
                    // Used to sync the finalization of both tasks
                    let finished = Cell::new(0b00u8);

                    let fut_queue = async {
                        loop {
                            if self.futures.borrow().is_empty() {
                                finished.set(finished.get() | 0b01);

                                if finished.get() >= 0b11 {
                                    // All possible futures and jobs were completed. Exit.
                                    return;
                                }

                                // All possible jobs were completed, but `jqueue` could have
                                // pending jobs. Yield to the executor to try to progress on
                                // `jqueue` until we have more pending futures.
                                task::yield_now().await;
                                continue;
                            }

                            finished.set(finished.get() & 0b10);

                            // Blocks on all the enqueued futures, driving them all to completion.
                            let futures = &mut std::mem::take(&mut *self.futures.borrow_mut());

                            while let Some(job) = futures.next().await {
                                // Important to schedule the returned `job` into the job queue, since that's
                                // what allows updating the `Promise` seen by ECMAScript for when the future
                                // completes.
                                self.enqueue_promise_job(job, &mut context.borrow_mut());
                            }
                        }
                    };

                    let job_queue = async {
                        loop {
                            if self.jobs.borrow().is_empty() {
                                finished.set(finished.get() | 0b10);

                                if finished.get() >= 0b11 {
                                    // All possible futures and jobs were completed. Exit.
                                    return;
                                }

                                // All possible jobs were completed, but `fqueue` could have
                                // pending futures. Yield to the executor to try to progress on
                                // `fqueue` until we have more pending jobs.
                                task::yield_now().await;
                                continue;
                            };

                            finished.set(finished.get() & 0b01);

                            let jobs = std::mem::take(&mut *self.jobs.borrow_mut());

                            for job in jobs {
                                if let Err(e) = job.call(&mut context.borrow_mut()) {
                                    eprintln!("Uncaught {e}");
                                }

                                task::yield_now().await;
                            }
                        }
                    };

                    tokio::join!(fut_queue, job_queue);
                })
                .await;
        })
    }

    fn run_jobs(&self, context: &mut Context) {
        todo!()
    }
}
