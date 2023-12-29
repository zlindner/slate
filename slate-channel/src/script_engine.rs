use convert_case::{Case, Casing};
use pyo3::{pyclass, pymethods, types::PyModule, Python};
use std::{fs, path::Path};

use crate::session::ChannelSession;

pub struct PortalScriptEngine {}

impl PortalScriptEngine {
    pub fn execute_script(name: &str) -> bool {
        // Get script camel case path
        let camel_case_name = name.to_case(Case::Snake);
        let script_name = format!("{}.py", camel_case_name);
        let script_path = format!("scripts/portal/{}", script_name);
        let path = Path::new(&script_path);

        // Ensure the path exists -- many scripts aren't yet implemented
        if !path.exists() {
            log::warn!("Portal script {} doesn't exist", camel_case_name);
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
        let proxy = PortalScriptProxy {};

        Python::with_gil(|py| {
            // TODO error handle
            let py_module =
                PyModule::from_code(py, &py_code, &script_name, &camel_case_name).unwrap();

            result = py_module
                .getattr("enter")
                .unwrap()
                .call1((proxy,))
                .unwrap()
                .extract()
                .unwrap();
        });

        result
    }
}

#[pyclass]
struct PortalScriptProxy;

#[pymethods]
impl PortalScriptProxy {
    fn has_level_30_character(&self) -> bool {
        log::debug!("has_level_30_character");
        true
    }

    fn open_npc(&self, id: i32) {
        log::debug!("open_npc: {}", id);
    }

    fn block_portal(&self) {
        log::debug!("block_portal");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let result = PortalScriptEngine::execute_script("tutoChatNPC");
        assert!(result);
    }
}
