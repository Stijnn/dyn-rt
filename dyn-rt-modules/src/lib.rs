use dyn_rt::{
    registry::PluginRegistryBuilder, serde_json, utils::{Plugin, PluginBuilder}
};

#[dyn_rt::macros::plugin]
pub fn plugin_entry_point() -> Plugin {
    PluginBuilder::new()
        .set_name("dyn_rt_modules".into())
        .set_description("Test Description".into())
        .add_commands(vec![
            "sum".into(),
            "to_json_pretty".into(),
            "from_json_string".into(),
            "massive_vec_sort".into(),
            "create_a_new_registry".into(),
        ])
        .set_version("1.0.0".into())
        .build()
}

#[dyn_rt::macros::command]
pub fn sum(a: i32, b: i32) -> Result<i32, i32> {
    Ok(a + b)
}

#[dyn_rt::macros::command]
pub fn to_json_pretty(input: serde_json::Value) -> String {
    serde_json::to_string_pretty(&input).unwrap()
}

#[dyn_rt::macros::command]
pub fn from_json_string(input: String) -> serde_json::Value {
    serde_json::from_str(&input).unwrap()
}

#[dyn_rt::macros::command]
pub fn massive_vec_sort(input: Vec<i32>) -> Vec<i32> {
    let mut to_sort = input;
    to_sort.sort();
    return to_sort;
}

#[dyn_rt::macros::command]
pub fn create_a_new_registry() {
    let linker = PluginRegistryBuilder::new().add_library(std::path::PathBuf::from("dyn_rt_modules.dll")).build();
    println!("{:?}", linker.get_plugins_vec());
}
