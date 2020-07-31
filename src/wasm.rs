// For building wasm32 no_std, add panic handler and imports/exports for
// functions used in IPC between WebAssembly and Javascript. This panic handler
// cannot be included for building test or main because it would conflict with
// their panic handlers.
#[cfg(target_arch = "wasm32")]
pub mod no_std_bindings;

// Always include IPC shared memory buffer stuff
pub mod ipc_mem;
