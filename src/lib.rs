#![no_std]

// Boilerplate to use no_std without wasm-bindgen and wasm-pack
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_pi: &PanicInfo) -> ! {
    loop {}
}

// Include the static arrays generated from vocab list files
mod autogen_hsk;

// Shared mailbox buffers for copying messages in & out of the VM
pub const MAILBOX_SIZE: usize = 73;
pub static mut WASM_INBOX: [u8; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
pub static mut WASM_OUTBOX: [u8; MAILBOX_SIZE] = [0; MAILBOX_SIZE];

// Copy message into outbox buffer; return message size in bytes.
// no_mangle makes disassembly more readable in browser dev tools.
#[no_mangle]
fn copy_to_outbox(message: &str, bytes_to_skip: usize) -> usize {
    let mut copied_bytes = 0;
    unsafe {
        for (i, b) in message.bytes().enumerate() {
            // TODO: better strategy for overflow (vs. silently drop extra)
            if bytes_to_skip + i < MAILBOX_SIZE {
                WASM_OUTBOX[bytes_to_skip + i] = b;
                copied_bytes += 1;
            } else {
                break;
            }
        }
    }
    copied_bytes
}

// Export location & size of utf8 mailbox buffers in VM shared memory
// no_mangle is necessary here to get predictable names for linking
#[no_mangle]
pub unsafe extern "C" fn wasm_inbox_ptr() -> *const u8 {
    WASM_INBOX.as_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn wasm_outbox_ptr() -> *const u8 {
    WASM_OUTBOX.as_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn wasm_mailbox_size() -> usize {
    MAILBOX_SIZE
}

// Receive message; update state machine & outbox; return outbound message size (bytes).
// Side-effect: copy utf8 string representing current state into outbox buffer
#[no_mangle]
pub extern "C" fn exchange_messages(n: usize) -> usize {
    let inbox_msg: &str;
    unsafe {
        inbox_msg = match core::str::from_utf8(&WASM_INBOX[0..n]) {
            Ok(s) => &s,
            Err(_) => &"", // TODO: handle mal-formed utf8 strings better
        };
    }
    let mut outbox_bytes = 0;
    match autogen_hsk::PINYIN.binary_search(&inbox_msg) {
        Ok(i) => outbox_bytes += copy_to_outbox(&autogen_hsk::CIYU[i], outbox_bytes),
        Err(_) => outbox_bytes += copy_to_outbox(&"...", outbox_bytes),
    }
    outbox_bytes
}
