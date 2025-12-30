pub mod macros {
    pub use dyn_rt_macros::plugin as plugin;
    pub use dyn_rt_macros::command;
}

pub mod attach;
pub mod registry;

pub use serde;
use serde::{Deserialize, Serialize};
pub use serde_json;
pub use dyn_rt_utils as utils;

#[derive(Serialize, Deserialize)]
pub struct WrappedResult<T> {
    pub data: Option<T>,
    pub error: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FnDescriptor {
    function_name: String,
    parameters: Vec<FnParameterDescriptor>,
    return_type: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FnParameterDescriptor {
    name: String,
    #[serde(alias = "type")]
    dtype: String,
}