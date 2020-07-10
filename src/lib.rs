// 1. This is a WASM example (not WASI). Because the standard library depends
//    on system call bindings that aren't available in WASM, we have to specify
//    #[no_std] and implement a panic handler.
//
// 2. For info on the `-> !` return type in the panic handler declaration,
//    search for "never type" or "diverging function".
//
// 3. In the `#[link(wasm_import_module = "js")]` attribute for the extern
//    bindings, "js" matches the name of an importObject key on the javascript
//    side. For example: `var importObject = {js: {...}};`. For more on module
//    imports and exports, see https://webassembly.org/docs/modules/
//
// 4. Each imported or exported function needs an #[no_mangle] attribute so the
//    rust function names will be preserved for the javascript side to use.

#![no_std]
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_pi: &PanicInfo) -> ! {
    loop {}
}

pub const BUF_SIZE: usize = 64;
pub static mut BUF: [u8; BUF_SIZE] = [0; BUF_SIZE];

#[link(wasm_import_module = "js")]
extern "C" {
    #[no_mangle]
    fn log(n: i32);
}

#[no_mangle]
pub unsafe extern "C" fn msg_buf_ptr() -> *const u8 {
    BUF.as_ptr()
}

#[no_mangle]
fn send_message(message: &str) -> usize {
    unsafe {
        for (i, b) in message.bytes().enumerate() {
            BUF[i] = b;
        }
    }
    message.len()
}

#[no_mangle]
// Returns number of bytes copied into BUF
pub extern "C" fn request_message(n: i32) -> usize {
    match n {
        1 => send_message(&"🌄✨"),
        2 => send_message(&"你好"),
        _ => send_message(&""),
    }
}
