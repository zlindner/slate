use crate::session::ChannelSession;
use convert_case::{Case, Casing};
use pyo3::{
    coroutine::Coroutine,
    pyclass, pyfunction, pymethods,
    types::{PyFuture, PyModule},
    Py, PyResult, Python,
};
use sqlx::Row;
use std::{fs, path::Path};

pub struct PortalScriptEngine {}

impl PortalScriptEngine {
    pub async fn execute_script(name: &str, session: &mut ChannelSession) -> bool {
        // Get script camel case path
        let camel_case_name = name.to_case(Case::Snake);
        let script_name = format!("{}.py", camel_case_name);
        let script_path = format!("slate-channel/scripts/portal/{}", script_name);
        let path = Path::new(&script_path);

        // Ensure the path exists -- many scripts aren't yet implemented
        if !path.exists() {
            log::warn!("Portal script {} doesn't exist", script_path);
            return false;
        }

        let py_code = match fs::read_to_string(path) {
            Ok(py_code) => py_code,
            Err(e) => {
                log::error!("Error reading script {:?}: {}", path, e);
                return false;
            }
        };

        let mut result = false;

        Python::with_gil(|py| {
            // SAFETY: proxy is created within this scope to ensure `handle` does not leak, and is not
            // stored anywhere in scripts -- so transmuting is fine
            let proxy = PortalScriptProxy {
                handle: session.into(),
                script: name.to_string(),
            };

            // TODO error handle
            let py_module =
                PyModule::from_code(py, &py_code, &script_name, &camel_case_name).unwrap();

            let future: Py<PyFuture> = Py::from_object(
                py,
                py_module
                    .getattr("enter")
                    .unwrap()
                    .call1((proxy,))
                    .unwrap()
                    .extract()
                    .unwrap(),
            )
            .unwrap();

            Coroutine::new("test".into(), future);
        });

        result
    }
}

#[pyclass]
struct PortalScriptProxy {
    handle: ChannelSessionHandle,
    script: String,
}

#[pymethods]
impl PortalScriptProxy {
    async fn has_level_30_character(&mut self) -> bool {
        log::debug!("has_level_30_character");
        let session: &mut ChannelSession = self.handle.as_mut();

        let levels = sqlx::query("SELECT level FROM characters WHERE account_id = ?")
            .bind(session.account_id.unwrap())
            .fetch_all(&session.db)
            .await
            .unwrap();

        for level in levels {
            if level.get::<i32, _>("level") >= 30 {
                return true;
            }
        }

        false
    }

    fn open_npc(&mut self, id: i32) {
        log::debug!("open_npc: {}", id);
    }

    fn block_portal(&mut self) {
        log::debug!("block_portal");
        let session: &mut ChannelSession = self.handle.as_mut();

        session
            .character
            .as_mut()
            .unwrap()
            .blocked_portals
            .insert(self.script.clone());
    }
}

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
