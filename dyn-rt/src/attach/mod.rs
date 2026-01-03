use std::{collections::HashMap, path::PathBuf, sync::Arc};

use dyn_rt_utils::{PLUGIN_ENTRY_POINT_DECL, Plugin};

use crate::FnDescriptor;

#[derive(Debug)]
pub struct AttachedPlugin {
    pub name: String,
    pub description: String,
    pub cargo_version: String,
    pub functions: HashMap<String, FnDescriptor>,
    pub library: Arc<libloading::Library>,
}

/// Attempts to attach a dynamic-link-library to the host process using [`libloading`].
///
/// Calls `_impl_attach_dyn_plugin` with signature: [`crate::utils::PluginRegistrationFn`] to start plugin binding.
///
/// Example:
/// ```
/// let plugin = dyn_rt::attach::attach_library(&std::path::PathBuf::from("dyn_rt_modules.dll"));
/// ```
pub fn attach_library(filename: &PathBuf) -> Result<AttachedPlugin, String> {
    let lib = unsafe { libloading::Library::new(filename) };

    if lib.is_err() {
        let ef = format!(
            "Failed to load plugin. Libloading threw an error: {}",
            lib.unwrap_err()
        );
        eprintln!("{ef}");
        return Err(ef);
    }

    let lib = lib.unwrap();
    let plugin_attach_fn =
        unsafe { lib.get::<crate::utils::PluginRegistrationFn>(PLUGIN_ENTRY_POINT_DECL) };

    if plugin_attach_fn.is_err() {
        let ef = format!(
            "Failed to load plugin. Could not find: _impl_attach_dyn_plugin. Error: {}",
            plugin_attach_fn.unwrap_err()
        );
        eprintln!("{ef}");
        let _ = lib.close();
        return Err(ef);
    }

    let plugin_attach_fn = plugin_attach_fn.unwrap();
    let plugin = plugin_attach_fn();

    Ok(AttachedPlugin::from(plugin, lib))
}

impl AttachedPlugin {
    pub fn from(value: Plugin, lib: libloading::Library) -> Self {
        let name = unsafe {
            std::ffi::CStr::from_ptr(value.name)
                .to_string_lossy()
                .into_owned()
        };
        let desc = unsafe {
            std::ffi::CStr::from_ptr(value.description)
                .to_string_lossy()
                .into_owned()
        };
        let version = unsafe {
            std::ffi::CStr::from_ptr(value.cargo_version)
                .to_string_lossy()
                .into_owned()
        };
        let functions = unsafe {
            std::ffi::CStr::from_ptr(value.commands)
                .to_string_lossy()
                .into_owned()
        };

        let lib_arc = Arc::new(lib);
        AttachedPlugin {
            library: lib_arc.clone(),
            name,
            description: desc,
            cargo_version: version,
            functions: functions
                .split(';')
                .filter(|f| !f.is_empty()) // Prevent empty strings from being processed
                .map(|f| {
                    let name = Self::create_fn_impl_name(f.to_string());
                    let call_result = Self::impl_call::<FnDescriptor>(&lib_arc, name.clone(), serde_json::json!({}));
                    (name, call_result)
                })
                .filter_map(|(name, res)| res.ok().map(|val| (name, val))) // Filters and unwraps in one step
                .collect::<HashMap<String, FnDescriptor>>(),
        }
    }

    fn create_fn_impl_name(fn_name: impl std::fmt::Display) -> String {
        format!(
            "__impl_fd_schematic_{}{}",
            dyn_rt_utils::PLUGIN_DECL_APPENDIX,
            fn_name
        )
    }

    pub fn func_descriptor(&self, fn_name: impl std::fmt::Display) -> Result<FnDescriptor, String> {
        self.call::<crate::FnDescriptor>(Self::create_fn_impl_name(fn_name), serde_json::json!({}))
    }

    pub fn call<T: serde::de::DeserializeOwned>(
        &self,
        fn_name: impl std::fmt::Display,
        args: impl serde::Serialize,
    ) -> Result<T, String> {
        Self::impl_call::<T>(&self.library, fn_name, args)
    }

    fn impl_call<T: serde::de::DeserializeOwned>(
        lib: &libloading::Library,
        fn_name: impl std::fmt::Display,
        args: impl serde::Serialize,
    ) -> Result<T, String> {
        use std::ffi::{CStr, CString};
        use std::os::raw::c_char;

        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_char> = lib
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

    pub fn unload(self) {
        drop(self);
    }
}
