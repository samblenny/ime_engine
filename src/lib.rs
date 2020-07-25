#![no_std]
// WASM Notes:
// 1. The panic() boilerplate below allows use of no_std without wasm-bindgen
//    and wasm-pack.
// 2. Using #[no_mangle] on public functions is necessary for linking.
// 3. Using #[no_mangle] on other functions reduces binary size and helps with
//    disassebly and step tracing in browser dev tools.
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

// Copy message into outbox buffer; return cumulative message size in bytes.
#[no_mangle]
fn copy_to_outbox(message: &str, mut cumulative_bytes: usize) -> usize {
    unsafe {
        for b in message.bytes() {
            // TODO: better strategy for overflow (vs. silently drop extra)
            if cumulative_bytes + 1 >= MAILBOX_SIZE {
                break;
            }
            WASM_OUTBOX[cumulative_bytes] = b;
            cumulative_bytes += 1;
        }
    }
    cumulative_bytes
}

// Export location & size of utf8 mailbox buffers in VM shared memory
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

// Look up characters for search query (ASCII romanized text, maybe mixed with Unicode).
// Side-effect: copies utf8 result string to outbox buffer (append possible).
// Returns: cumulative bytes of result copied to outbox (recursion possible).
#[no_mangle]
fn look_up(query: &str, outbox_cumulative_bytes: usize) -> usize {
    match autogen_hsk::PINYIN.binary_search(&query) {
        Ok(i) => copy_to_outbox(&autogen_hsk::CIYU[i], outbox_cumulative_bytes),
        Err(_) => outbox_cumulative_bytes,
    }
}

// Receive message; update state machine & outbox; return outbound message size (bytes).
#[no_mangle]
pub extern "C" fn exchange_messages(n: usize) -> usize {
    let inbox_msg: &str;
    unsafe {
        inbox_msg = match core::str::from_utf8(&WASM_INBOX[0..n]) {
            Ok(s) => &s,
            Err(_) => &"", // TODO: handle mal-formed utf8 strings better
        };
    }
    let outbox_cumulative_bytes = 0;
    look_up(&inbox_msg, outbox_cumulative_bytes)
}
