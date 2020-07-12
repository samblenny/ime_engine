#![no_std]

// Include the static arrays generated from vocab list files
mod autogen_hsk;

// Boilerplate to avoid need for wasm-bindgen and wasm-pack
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_pi: &PanicInfo) -> ! {
    loop {}
}

// Shared message buffer for communicating with javascript
const JS_INBOX_SIZE: usize = 73;
pub static mut JS_INBOX: [u8; JS_INBOX_SIZE] = [0; JS_INBOX_SIZE];

// Copy message into js inbox buffer; return message size in bytes.
// Using no_mangle makes disassembly prettier on the browser side.
#[no_mangle]
fn copy_message(message: &str) -> usize {
    unsafe {
        for (i, b) in message.bytes().enumerate() {
            JS_INBOX[i] = b;
        }
    }
    message.len()
}

// Return pointer to shared message buffer (the js inbox).
#[no_mangle]
pub unsafe extern "C" fn js_inbox_ptr() -> *const u8 {
    JS_INBOX.as_ptr()
}

// Receive 1 keystroke; update state machine & js inbox; return message size (bytes).
// Side-effect: copy utf8 string representing current state into js inbox buffer
#[no_mangle]
pub extern "C" fn keystroke_event(n: i32) -> usize {
    if n >= 0 && n < autogen_hsk::HANZI.len() as i32 {
        copy_message(autogen_hsk::HANZI[n as usize])
    } else {
        copy_message(autogen_hsk::PINYIN[5])
    }
}
