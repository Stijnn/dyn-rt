use std::{collections::HashMap, path::PathBuf, sync::Arc};

use dyn_rt_utils::PLUGIN_DECL_APPENDIX;

use crate::attach::AttachedPlugin;

pub struct PluginRegistry {
    plugins: HashMap<String, Arc<crate::attach::AttachedPlugin>>,
}

impl PluginRegistry {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn add_plugin(&mut self, plugin: Arc<crate::attach::AttachedPlugin>) {
        self.plugins.insert(plugin.name.clone(), plugin);
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

    pub fn get_plugin(&self, plugin_name: &str) -> Option<Arc<AttachedPlugin>> {
        self.get_plugins_map().get(plugin_name).map(Arc::clone)
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
        loaded_library.call(unmangled_wrapper_name, args)
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
