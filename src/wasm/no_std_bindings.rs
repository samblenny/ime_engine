// WASM Notes:
// 1. The panic() boilerplate below allows use of no_std without wasm-bindgen
//    and wasm-pack.
// 2. Using #[no_mangle] on public functions is necessary for linking.
// 3. Using #[no_mangle] on other functions reduces binary size and helps with
//    disassembly and step tracing in browser dev tools. But, name collisions
//    can cause SIGSEGV. Be careful with common names like 'write'.

#[link(wasm_import_module = "js")]
extern "C" {
    fn js_warn_wasm_panic();
    pub fn js_log_trace(code: i32);
}

use core::panic::PanicInfo;
#[panic_handler]
pub fn panic(_panic_info: &PanicInfo) -> ! {
    unsafe {
        js_warn_wasm_panic();
    }
    loop {}
}

// Export location & size of IPC message buffers in VM shared memory
#[no_mangle]
pub unsafe extern "C" fn wasm_inbox_ptr() -> *const u8 {
    super::ipc_mem::IN.as_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn wasm_outbox_ptr() -> *const u8 {
    super::ipc_mem::OUT.as_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn wasm_mailbox_size() -> usize {
    super::ipc_mem::BUF_SIZE
}
