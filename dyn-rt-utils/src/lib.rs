use std::os::raw::c_char;

pub type PluginRegistrationFn = extern "C" fn() -> Plugin;
pub const PLUGIN_DECL_APPENDIX: &str = "_dyn_rt_plugin_wrapper_impl_for_";
pub const PLUGIN_ENTRY_POINT_DECL: &str = "_impl_attach_dyn_plugin";

#[repr(C)]
pub struct Plugin {
    pub name: *const c_char,
    pub description: *const c_char,
    pub cargo_version: *const c_char,
}

impl Plugin {
    pub fn new(name: &'static str, desc: &'static str, version: &'static str) -> Self {
        Self {
            name: name.as_ptr() as *const c_char,
            description: desc.as_ptr() as *const c_char,
            cargo_version: version.as_ptr() as *const c_char,
        }
    }
}