# dyn-rt
`dyn-rt` is a library created to attempt to simplify internal invocation on runtime linked libraries.

# dyn-rt-macros
`dyn-rt-macros` contains procedural macros to support the `dyn-rt` crate and exposes macros to simplify creation of plugins.
The macros define `unsafe` and `no_mangle` interfaces that `dyn-rt` uses for its invocation.

# dyn-rt goals
- Allow `Async`
- FFI/ABI Stability
- Binary Encoding/Decoding
- Plugin Reflection through static interface
- Sandboxing

# Usage/Examples
## Defining a plugin
For now the example below has no return type. In the future this will request a `expose![]` macro. Allowing you to define functions you wish to expose on lookup.
```Rust
// Assign a entry-point for your plugin like:
#[dyn_rt::macros::plugin()]
pub fn dllmain() {

}
```

## Defining a command
`References/Pointers/Unsafe` are currently unsupported due to serialization/deserialization.
```Rust
#[dyn_rt::macros::command]
pub fn sum(a: i32, b: i32) -> Result<i32, i32> {
    Ok(a + b)
}
```

## Loading a plugin
```Rust
// Loads a plugin
let plugin = dyn_rt::attach::attach_library("dyn_rt_modules.dll".into());
```

## Creating a registry (multiple plugins)
```Rust
// Creates a plugin registry using the PluginRegistryBuilder
let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
    .add_library("dyn_rt_modules.dll".into())
    // AND/OR
    .add_libraries(vec![
        "dyn_rt_modules.dll".into(),
        ...
    ])
    .build();
```

## Subtle reflection through `__impl_fd_schematic_...`
```Rust
// Calling
let plugin_registry = dyn_rt::registry::PluginRegistryBuilder::new()
    .add_library("dyn_rt_modules.dll")
    .build();

let descriptor_result = plugin_registry.func_descriptor("dyn-rt-modules", "sum");
let descriptor = descriptor_result.unwrap();

// ---

// Should result in:
FnDescriptor { 
    function_name: "sum", 
    parameters: [
        FnParameterDescriptor { name: "a", dtype: "i32" }, 
        FnParameterDescriptor { name: "b", dtype: "i32" }
    ], 
    return_type: "Result<i32,i32>" 
}
```

# why the use of serde_json
For now invocation is done by Serialization/Deserialization using a JSON buffer. The goal is that in the future this will be replaced with FFI/ABI Stable interfaces.

# LICENSE
MIT License

Copyright (c) 2025 Stijn

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.