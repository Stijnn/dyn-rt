#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use dyn_rt::{
        serde::{Deserialize, Serialize},
        serde_json::json,
    };
    use serde_json::to_string;

    pub fn get_plugin_binary_path(name: &str) -> PathBuf {
        let mut path = std::env::current_exe().expect("Failed to get current exe path");

        path.pop();

        #[cfg(target_os = "windows")]
        let filename = format!("{}.dll", name);
        #[cfg(target_os = "linux")]
        let filename = format!("lib{}.so", name);
        #[cfg(target_os = "macos")]
        let filename = format!("lib{}.dylib", name);

        let full_path = path.join(filename);

        assert!(full_path.exists(), "Plugin not found at: {:?}", full_path);

        full_path
    }

    #[test]
    fn load_plugin() {
        let plugin = dyn_rt::attach::attach_library(&get_plugin_binary_path("dyn_rt_modules"));

        assert!(plugin.is_ok(), "Failed to load plugin");

        let plugin = plugin.unwrap();
        println!("Loaded Plugin: {:?}", plugin);
    }

    #[test]
    fn plugin_registry_builder_should_succeed() {
        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        assert_eq!(
            plugin_registry.get_plugins_vec().len(),
            1,
            "Registry plugin len was {} but should have been 1",
            plugin_registry.get_plugins_vec().len()
        );
    }

    #[test]
    fn plugin_registry_run_function_that_does_not_exist() {
        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        let res = plugin_registry.invoke_function::<i32>(
            "dyn_rt_modules",
            "call_me_from_tests",
            json!({
                "a": 1i32,
                "b": 1i32,
            }),
        );

        assert!(
            res.is_err(),
            "PluginRegistry::invoke_function should not be able to find function({}+{}) because it should not exist",
            "dyn_rt_modules",
            "call_me_from_tests"
        );
    }

    #[test]
    fn plugin_registry_run_function_that_does_exist() {
        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        let res = plugin_registry.invoke_function::<Result<i32, i32>>(
            "dyn_rt_modules",
            "sum",
            json!({
                "a": 1i32,
                "b": 1i32,
            }),
        );

        assert!(
            res.is_ok(),
            "PluginRegistry::invoke_function should be able to find function({}+{}) because it exists. Err: {}",
            "dyn_rt_modules",
            "sum",
            res.unwrap_err()
        );

        let result = res.unwrap();
        assert_eq!(
            result,
            Ok(2),
            "1+1=2. If this fails, something doesn't make sense. Check encoding, payload and/or endianess"
        );
    }

    #[test]
    fn plugin_registry_run_function_that_runs_library_function() {
        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        let res = plugin_registry.invoke_function::<String>(
            "dyn_rt_modules",
            "to_json_pretty",
            json!({
                "input": {
                    "Hello": "World!",
                    "From": "dyn-rt-tests"
                }
            }),
        );

        assert!(
            res.is_ok(),
            "PluginRegistry::invoke_function should be able to find function({}+{}) because it exists. Err: {}",
            "dyn_rt_modules",
            "sum",
            res.unwrap_err()
        );

        let result = res.unwrap();
        assert!(
            !result.is_empty(),
            "Something went wrong with calling lib inside main-lib"
        );
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
    struct SerializeTest {
        hello: String,
        world: i32,
        something: bool,
    }

    #[test]
    fn plugin_registry_run_function_that_runs_library_function_should_also_work() {
        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        let input = to_string(&SerializeTest {
            hello: "World".to_owned(),
            world: 2026i32,
            something: true,
        })
        .unwrap();

        let res = plugin_registry.invoke_function::<SerializeTest>(
            "dyn_rt_modules",
            "from_json_string",
            json!({
                "input": input
            }),
        );

        assert!(
            res.is_ok(),
            "PluginRegistry::invoke_function should be able to find function({}+{}) because it exists. Err: {}",
            "dyn_rt_modules",
            "from_json_string",
            res.unwrap_err()
        );

        let result = res.unwrap();
        assert_eq!(
            result,
            SerializeTest {
                hello: "World".to_owned(),
                world: 2026i32,
                something: true,
            },
            "Output does not match. Should be: {:?} is {:?}",
            SerializeTest {
                hello: "World".to_owned(),
                world: 2026i32,
                something: true,
            },
            result
        );
    }

    #[test]
    fn sort_massive_vec() {
        let mut input_will_be: Vec<i32> = vec![];
        for x in 0..40000 {
            input_will_be.push(40000 - x);
        }

        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        let r = plugin_registry.invoke_function::<Vec<i32>>(
            "dyn_rt_modules",
            "massive_vec_sort",
            json!({
                "input": input_will_be
            }),
        );

        if r.is_err() {
            panic!("{}", r.unwrap_err());
        }

        input_will_be.sort();
        let v = r.unwrap();
        assert_eq!(v, input_will_be, "The sort does not match");
    }

    #[test]
    fn create_registry_in_registry_fn() {
        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        let _ = plugin_registry.invoke_function::<()>("dyn_rt_modules", "create_a_new_registry", json!({}));
    }

    #[test]
    fn retrieve_func_descriptor_should_work() {
        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        let plugin = plugin_registry.get_plugin("dyn_rt_modules");
        assert!(plugin.is_some(), "Should be some");

        let plugin = plugin.unwrap();
        let descriptor = plugin.func_descriptor("sum");
        assert!(descriptor.is_ok(), "Should have sum function descriptor");

        let descriptor = descriptor.unwrap();
        println!("{:?}", descriptor);
    }

    #[test]
    fn run_async_fn() {
        let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
            .add_library(get_plugin_binary_path("dyn_rt_modules"))
            .build();

        let plugin = plugin_registry.get_plugin("dyn-rt-modules");
        assert!(plugin.is_some(), "Should be some");

        let _ = plugin_registry.invoke_function::<()>("dyn-rt-modules", "async_test_delay", json!({
            "ms": 10000
        }));
    }
}
