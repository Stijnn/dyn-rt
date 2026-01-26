use std::os::raw::c_char;

pub type PluginRegistrationFn = extern "C" fn() -> Plugin;

pub const PLUGIN_DECL_APPENDIX: &str = "_dyn_rt_plugin_wrapper_impl_for_";
pub const PLUGIN_ENTRY_POINT_DECL: &str = "_impl_attach_dyn_plugin";

#[repr(C)]
pub struct Plugin {
    pub name: *const c_char,
    pub description: *const c_char,
    pub cargo_version: *const c_char,
    pub commands: *const c_char,
}

impl Plugin {
    pub fn new(name: &'static str, desc: &'static str, version: &'static str, commands: &'static str) -> Self {
        Self {
            name: name.as_ptr() as *const c_char,
            description: desc.as_ptr() as *const c_char,
            cargo_version: version.as_ptr() as *const c_char,
            commands: commands.as_ptr() as *const c_char,
        }
    }
}

pub struct PluginBuilder {
    name: String,
    description: String,
    functions: Vec<String>,
    version: String
}

impl PluginBuilder {

    pub fn new() -> Self {
        Self { name: "".into(), description: "".into(), functions: vec![], version: "".into() }
    }

    pub fn add_command(&mut self, function: impl Into<String>) -> &mut Self {
        self.functions.push(function.into());
        self
    }

    pub fn add_commands(&mut self, mut functions: Vec<String>) -> &mut Self {
        self.functions.append(&mut functions);
        self
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = description.into();
        self
    }

    pub fn set_version(&mut self, version: impl Into<String>) -> &mut Self {
        self.version = version.into();
        self
    }

    pub fn build(&self) -> Plugin {
        let functions_str = self.functions
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(";");
    
        let leaked_functions: &'static str = Box::leak((functions_str + "\0").into_boxed_str());
        let leaked_name: &'static str = Box::leak((self.name.clone() + "\0").into_boxed_str());
        let leaked_desc: &'static str = Box::leak((self.description.clone() + "\0").into_boxed_str());
        let leaked_version: &'static str = Box::leak((self.version.clone() + "\0").into_boxed_str());
    
        Plugin::new(
            leaked_name, 
            leaked_desc,
            leaked_version, 
            leaked_functions
        )
    }
}