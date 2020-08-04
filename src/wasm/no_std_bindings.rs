// WASM Notes:
// 1. The panic() boilerplate below allows use of no_std without wasm-bindgen
//    and wasm-pack.
// 2. Using #[no_mangle] on public functions is necessary for linking.
// 3. Using #[no_mangle] on other functions can help with disassembly and
//    tracing using browser developer console tools.

#[link(wasm_import_module = "js")]
extern "C" {
    pub fn js_log_trace(code: i32);
}

use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_panic_info: &PanicInfo) -> ! {
    // Rust semantics require panic handler to never return, and docs for
    // embedded no_std rust suggest to accomplish that with loop {}. In my
    // wasm32 testing, loop {} pegs CPU at 100% and makes browser UI
    // unresponsive. Better alternative is to use WebAssembly unreachable trap
    // instruction (available in stable since late 2019).
    unsafe {
        core::arch::wasm32::unreachable();
    }
}

// Export location & size of IPC message buffers in VM shared memory
#[no_mangle]
pub unsafe extern "C" fn wasm_query_buf_ptr() -> *const u8 {
    super::ipc_mem::IN.as_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn wasm_reply_buf_ptr() -> *const u8 {
    super::ipc_mem::OUT.as_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn wasm_buffer_size() -> usize {
    super::ipc_mem::BUF_SIZE
}
