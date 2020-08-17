#![no_std]
extern crate ime_engine;
use ime_engine::Writer;

// Always include IPC shared memory buffer stuff
pub mod ipc_mem;

// For building wasm32 no_std, add panic handler and imports/exports for
// functions used in IPC between WebAssembly and Javascript. This panic handler
// cannot be included for `cargo test` because it would conflict with the test
// panic handler.
#[cfg(target_arch = "wasm32")]
pub mod no_std_bindings;

// For wasm32 build, use debug trace WebAssembly IPC function binding
#[cfg(target_arch = "wasm32")]
use no_std_bindings::js_log_trace;

// For other builds (test), replace debug trace binding with stub
#[cfg(not(target_arch = "wasm32"))]
unsafe fn js_log_trace(_: i32) {}

// IPCWriter is a Writer for UTF-8 bytes backed by static IPC shared memory.
struct IPCWriter {}
impl Writer for IPCWriter {
    fn write(&mut self, message: &str) {
        ipc_mem::write(message);
    }

    // Log trace codes to the javascript console to help debug control flow.
    fn trace(&mut self, trace_code: i32) {
        unsafe {
            js_log_trace(trace_code);
        }
    }

    fn to_s(&self) -> &str {
        ipc_mem::out_to_s()
    }
}

// Receive query message, search, write results to IPC out buffer.
// This is for calling from Javascript with WebAssembly.
// Returns: number of bytes written to IPC out buffer.
#[no_mangle]
pub extern "C" fn query_shared_mem_ipc(n: usize) -> usize {
    let mut ipc_writer = IPCWriter {};
    let qry = ipc_mem::get_query(n);
    ipc_mem::rewind();
    ime_engine::look_up(&qry, &mut ipc_writer);
    ipc_mem::position()
}
