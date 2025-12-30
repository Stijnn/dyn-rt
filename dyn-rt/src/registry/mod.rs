use std::{collections::HashMap, path::PathBuf, sync::Arc};

use dyn_rt_utils::PLUGIN_DECL_APPENDIX;
use serde_json::json;

pub struct PluginRegistry {
    plugins: HashMap<String, Arc<crate::attach::AttachedPlugin>>,
}

impl PluginRegistry {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn unload_plugin(&mut self, name: &str) {
        self.plugins.retain(|k, _v| k != name);
    }

    pub fn unload_all(&mut self) {
        self.plugins.clear();
    }

    pub fn get_plugins_vec(&self) -> Vec<Arc<crate::attach::AttachedPlugin>> {
        self.plugins.iter().map(|kv| Arc::clone(kv.1)).collect()
    }

    pub fn get_plugins_map(&self) -> HashMap<String, Arc<crate::attach::AttachedPlugin>> {
        self.plugins.clone()
    }

    pub fn invoke_function<T: serde::de::DeserializeOwned>(
        &self,
        lib_name: impl std::fmt::Display,
        fn_name: impl std::fmt::Display,
        args: impl serde::Serialize,
    ) -> Result<T, String> {
        let name_str = lib_name.to_string();
        let loaded_library = self
            .plugins
            .get(&name_str)
            .ok_or_else(|| format!("Library {} not found", name_str))?;

        let unmangled_wrapper_name = format!("{}{}", PLUGIN_DECL_APPENDIX, fn_name);
        let fnd = PluginRegistry::call::<crate::FnDescriptor>(loaded_library, format!("__impl_fd_schematic_{}", unmangled_wrapper_name), json!({}));
        println!("{:?}", fnd.unwrap());
        PluginRegistry::call::<T>(loaded_library, unmangled_wrapper_name, args)
    }

    fn call<T: serde::de::DeserializeOwned>(
        lib: &crate::attach::AttachedPlugin,
        fn_name: impl std::fmt::Display,
        args: impl serde::Serialize,
    ) -> Result<T, String> {
        use std::ffi::{CStr, CString};
        use std::os::raw::c_char;

        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_char> = lib
                .library
                .get(fn_name.to_string().as_bytes())
                .map_err(|e| e.to_string())?;

            let json_payload = serde_json::to_string(&args).map_err(|e| e.to_string())?;
            let c_payload = CString::new(json_payload).unwrap();

            let raw_res_ptr = func(c_payload.as_ptr());
            if raw_res_ptr.is_null() {
                return Err("Plugin returned a null pointer".into());
            }

            let res_cstr = CStr::from_ptr(raw_res_ptr);
            let res_json = res_cstr.to_string_lossy().into_owned();

            match lib
                .library
                .get::<unsafe extern "C" fn(*mut c_char)>(b"_dyn_rt_free_string")
            {
                Ok(f) => f(raw_res_ptr),
                Err(_) => {
                    eprintln!(
                        "Failed to clean raw_res_ptr by calling _dyn_rt_free_string on pub fn call<T: serde::de::DeserializeOwned>(lib: &crate::attach::AttachedPlugin, fn_name: impl std::fmt::Display, args: impl serde::Serialize)"
                    )
                }
            };

            let wrapped: crate::WrappedResult<T> = serde_json::from_str(&res_json)
                .map_err(|e| format!("Failed to parse plugin response: {}", e))?;

            if let Some(err_msg) = wrapped.error {
                return Err(err_msg);
            }

            wrapped
                .data
                .ok_or_else(|| "Plugin returned no data and no error".into())
        }
    }
}

pub struct PluginRegistryBuilder {
    libraries: Vec<PathBuf>,
}

impl Default for PluginRegistryBuilder {
    fn default() -> Self {
        Self { libraries: vec![] }
    }
}

impl PluginRegistryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_library(&mut self, path: PathBuf) -> &mut Self {
        if path.exists() {
            self.libraries.push(path);
        }
        self
    }

    pub fn add_libraries(&mut self, paths: Vec<PathBuf>) -> &mut Self {
        paths.iter().for_each(|p| {
            self.add_library(p.to_path_buf());
        });
        self
    }

    pub fn build(&self) -> PluginRegistry {
        let mut registry = PluginRegistry::new();

        self.libraries.iter().for_each(|lib| {
            let plugin_result = crate::attach::attach_library(lib);
            match plugin_result {
                Ok(attached) => registry
                    .plugins
                    .insert(attached.name.clone(), Arc::from(attached)),
                Err(_) => todo!(),
            };
        });

        registry
    }
}
