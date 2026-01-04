pub mod macros {
    pub use dyn_rt_macros::command;
    pub use dyn_rt_macros::expose;
    pub use dyn_rt_macros::plugin;
}

pub mod attach;
pub mod registry;

pub use dyn_rt_utils as utils;
pub use serde;
use serde::{Deserialize, Serialize};
pub use serde_json;

#[derive(Serialize, Deserialize)]
pub struct WrappedResult<T> {
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FnDescriptor {
    pub function_name: String,
    pub parameters: Vec<FnParameterDescriptor>,
    pub return_type: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FnParameterDescriptor {
    pub name: String,
    pub dtype: String,
}

#[deprecated]
pub trait DescriptableFn {
    fn get_function_descriptor(&self) -> FnDescriptor;
}

