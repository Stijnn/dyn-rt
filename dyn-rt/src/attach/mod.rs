use std::{path::PathBuf};

use dyn_rt_utils::{PLUGIN_ENTRY_POINT_DECL, Plugin};

#[derive(Debug)]
pub struct AttachedPlugin {
    pub name: String,
    pub description: String,
    pub cargo_version: String,
    pub library: libloading::Library
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
        let ef = format!("Failed to load plugin. Libloading threw an error: {}", lib.unwrap_err());
        eprintln!("{ef}");
        return Err(ef);
    }

    let lib = lib.unwrap();
    let plugin_attach_fn = unsafe { lib.get::<crate::utils::PluginRegistrationFn>(PLUGIN_ENTRY_POINT_DECL) };

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

        AttachedPlugin {
            library: lib,
            name,
            description: desc,
            cargo_version: version,
        }
    }

    pub fn unload(self) {
        drop(self);
    }
}
