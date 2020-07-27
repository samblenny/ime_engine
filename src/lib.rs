#![no_std]
// WASM Notes:
// 1. The panic() boilerplate below allows use of no_std without wasm-bindgen
//    and wasm-pack.
// 2. Using #[no_mangle] on public functions is necessary for linking.
// 3. Using #[no_mangle] on other functions reduces binary size and helps with
//    disassebly and step tracing in browser dev tools.
#[link(wasm_import_module = "js")]
extern "C" {
    #[no_mangle]
    fn warn_wasm_panic();
}
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_panic_info: &PanicInfo) -> ! {
    unsafe {
        warn_wasm_panic();
    }
    loop {}
}

// Include the static arrays generated from vocab list files
mod autogen_hsk;

// Shared mailbox buffers for copying messages in & out of the VM
pub const MAILBOX_SIZE: usize = 200;
pub static mut WASM_INBOX: [u8; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
pub static mut WASM_OUTBOX: [u8; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
static mut OUTBOX_BYTES: usize = 0;

// Copy message into outbox buffer; return cumulative message size in bytes.
#[no_mangle]
fn send(message: &str) {
    unsafe {
        for b in message.bytes() {
            // TODO: better strategy for overflow (vs. silently drop extra)
            if OUTBOX_BYTES < MAILBOX_SIZE {
                WASM_OUTBOX[OUTBOX_BYTES] = b;
                OUTBOX_BYTES += 1;
            }
        }
    }
}

// Conditionally copy debug messages to the outbox buffer
#[no_mangle]
fn trace(message: &str) {
    if false {
        send(message);
    }
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

#[no_mangle]
fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

// Utf8Str adds character boundary metadata to &str to help with safely slicing
// substrings. "Safely" means avoid panic from requesting slice with byte range
// not aligned on encoded Unicode character boundaries.
struct Utf8Str<'a> {
    str_slice: &'a str,
    char_start_list: [usize; MAILBOX_SIZE],
    char_end_list: [usize; MAILBOX_SIZE],
    char_count: usize,
}
impl<'a> Utf8Str<'a> {
    #[no_mangle]
    pub fn new(str_slice: &'a str) -> Utf8Str {
        // Find start (inclusive lower bound) and end (exclusive upper bound) byte
        // index of each UTF-8 character in string slice
        let mut char_start_list: [usize; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
        let mut char_end_list: [usize; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
        let mut char_count = 0;
        for i in 1..str_slice.len() + 1 {
            if str_slice.is_char_boundary(i) {
                if char_count + 1 < MAILBOX_SIZE {
                    char_start_list[char_count + 1] = i;
                }
                if char_count < MAILBOX_SIZE {
                    char_end_list[char_count] = i;
                    char_count += 1;
                }
            }
        }
        Utf8Str {
            str_slice,
            char_start_list: char_start_list,
            char_end_list: char_end_list,
            char_count,
        }
    }

    // Slice a substring using character range (not bytes!).
    // Using get(start..end) instead of [start..end] avoids possible panic.
    // This follows ..= start/inclusive end/inclusive range semantics.
    // If you want the first character, call `char_slice(0, 0)`.
    #[no_mangle]
    pub fn char_slice(&self, start: usize, end: usize) -> Option<&str> {
        if start < MAILBOX_SIZE && end < MAILBOX_SIZE {
            let start_b = self.char_start_list[start];
            let end_b = self.char_end_list[end];
            self.str_slice.get(start_b..end_b)
        } else {
            None
        }
    }
}

// Look up 词语 for search query (pinyin keys are ASCII, but inbox is UTF-8).
// Side-effect: copies utf8 result string to outbox buffer.
#[no_mangle]
fn look_up(inbox_query: &str) {
    let query = Utf8Str::new(inbox_query);

    // Recursively search for matching substrings of query in character range start..end.
    // Side-effect: copy (append) matching substrings to outbox buffer.
    #[no_mangle]
    fn search(query: &Utf8Str, start: usize, end: usize, depth: usize) {
        // Stop Conditions: query slice empty or recursion too deep
        if start >= end || depth == 0 {
            return;
        }
        let limit = end - start;
        for i in 1..=limit {
            if let Some(query_slice) = query.char_slice(start, end - i) {
                if let Ok(ciyu_i) = autogen_hsk::PINYIN.binary_search(&query_slice) {
                    send(&autogen_hsk::CIYU[ciyu_i]);
                    // Full match: stop
                    // Partial match: continue search on remainder of query
                    if i > 0 {
                        let rest = end - i + 1;
                        search(&query, rest, end, depth - 1);
                    }
                    return;
                }
            }
        }
        // Err(_) => {
        //     if i == limit {
        //         // No match... skip first character and try again
        //         if let Some(skip) = query.char_slice(start, start) {
        //             send(skip);
        //         } ese {
        //             trace(&"[E_SKP]");
        //         }
        //         if limit > 1 {
        //             let rest = start + 1;
        //             search(&query, rest, end, depth - 1);
        //         }
        //         return;
        //     }
        // }
        trace(&"6");
    }

    let start = 0;
    let end = query.char_count;
    let depth = 20;
    search(&query, start, end, depth)
}

// Receive message; update state machine & outbox; return outbound message size (bytes).
#[no_mangle]
pub extern "C" fn exchange_messages(n: usize) -> usize {
    let inbox_query: &str;
    unsafe {
        inbox_query = match core::str::from_utf8(&WASM_INBOX[0..n]) {
            Ok(s) => &s,
            Err(_) => &"", // TODO: handle mal-formed utf8 strings better
        };
    }
    // TODO: better way to track bytes count for the outbox buffer
    unsafe {
        OUTBOX_BYTES = 0;
    }
    look_up(&inbox_query);
    unsafe {
        return OUTBOX_BYTES;
    }
}
