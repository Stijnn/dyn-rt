pub mod macros {
    pub use dyn_rt_macros::command;
    pub use dyn_rt_macros::plugin;
    pub use dyn_rt_macros::reflect;
}

pub mod attach;
pub mod registry;

use std::sync::OnceLock;

pub use schemars;
pub use tokio;
pub use dyn_rt_utils as utils;
pub use serde;
use serde::{Deserialize, Serialize};
pub use serde_json;
use tokio::runtime::Runtime;

pub static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create Tokio runtime")
    })
}

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
    pub schema: schemars::Schema
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

#[macro_export]
macro_rules! expose {
    ($($cmd:ident),* $(,)?) => {
        vec![$(stringify!($cmd).to_string()),*]
    };
}