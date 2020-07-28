#![no_std]
// WASM Notes:
// 1. The panic() boilerplate below allows use of no_std without wasm-bindgen
//    and wasm-pack.
// 2. Using #[no_mangle] on public functions is necessary for linking.
// 3. Using #[no_mangle] on other functions reduces binary size and helps with
//    disassembly and step tracing in browser dev tools.
#[link(wasm_import_module = "js")]
extern "C" {
    #[no_mangle]
    fn js_warn_wasm_panic();
    fn js_log_trace(code: i32);
}
use core::panic::PanicInfo;
#[panic_handler]
fn panic(_panic_info: &PanicInfo) -> ! {
    unsafe {
        js_warn_wasm_panic();
    }
    loop {}
}

// Include the static arrays generated from vocab list files
mod autogen_hsk;
// Index type for phrases listed in autogen_hsk::CIYU array
type CiyuIndex = usize;

// Shared mailbox buffers for copying messages in & out of the VM
pub const MAILBOX_SIZE: usize = 200;
pub static mut WASM_INBOX: [u8; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
pub static mut WASM_OUTBOX: [u8; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
static mut OUTBOX_BYTES: usize = 0;

// Append copy of message into outbox buffer.
// Side-effect: Update outbox buffer and byte count.
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

// Log trace codes to the javascript console to help debug control flow.
// Side-effect: Add to javascript console log.
#[no_mangle]
fn trace(trace_code: i32) {
    if true {
        unsafe {
            js_log_trace(trace_code);
        }
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
    // This follows start..end range semantics (upper bound exclusive).
    #[no_mangle]
    pub fn char_slice(&self, start: usize, end: usize) -> Option<&str> {
        // Subtle point: implicit test for end > 0
        if start < end && end <= MAILBOX_SIZE {
            let start_b = self.char_start_list[start];
            // Must not allow end==0 here. For usize, (0 - 1) will panic.
            let end_b = self.char_end_list[end - 1];
            self.str_slice.get(start_b..end_b)
        } else {
            None
        }
    }
}

// Find longest 词语 match in start..end character window of query buffer.
// Side-effect: None.
// Return: (index in 词语 array for match, end boundary character in query for match)
fn longest_match(query: &Utf8Str, start: usize, mut end: usize) -> Option<(CiyuIndex, usize)> {
    end = min(query.char_count, end);
    // Subtle point: implicit test for end > 0
    while end > start {
        if let Some(query_slice) = query.char_slice(start, end) {
            if let Ok(ciyu) = autogen_hsk::PINYIN.binary_search(&query_slice) {
                return Some((ciyu, end));
            }
        }
        // Must not allow end==0 here. For usize, (0 - 1) will panic.
        end -= 1;
    }
    return None;
}

// Expand 词语 matches to include choice prompts.
// Side-effect: Send results to oubox buffer.
fn expand_ciyu(ciyu: &str) {
    let n = ciyu.split("\t").count();
    if n == 1 {
        send(ciyu);
    } else {
        send(&" (");
        for (i, choice) in ciyu.split("\t").enumerate() {
            send(match i {
                0 => &"1",
                1 => &"2",
                2 => &"3",
                3 => &"4",
                4 => &"5",
                5 => &"6",
                6 => &"7",
                7 => &"8",
                _ => &"9",
            });
            send(choice);
            if i + 1 < n {
                send(&" ");
            }
        }
        send(&") ");
    }
}

// Search for 词语 matches in substrings of query.
// Side-effect: Send results to outbox buffer.
#[no_mangle]
fn search(query: &Utf8Str, mut start: usize, end: usize) {
    while start < end {
        // Limit window size to length of longest phrase in pinyin array
        let window_end = min(start + autogen_hsk::PINYIN_SIZE_MAX, end);
        if let Some((ciyu_i, match_end)) = longest_match(query, start, window_end) {
            // Match: skip matching portion of query (possibly all of it)
            expand_ciyu(&autogen_hsk::CIYU[ciyu_i]);
            start = match_end;
        } else {
            // No match... skip one character
            if let Some(skip) = query.char_slice(start, start + 1) {
                send(skip);
            }
            start += 1;
        }
    }
}

// Look up 词语 for search query (pinyin keys are ASCII, but inbox is UTF-8).
// Side-effect: copies utf8 result string to outbox buffer.
#[no_mangle]
fn look_up(inbox_query: &str) {
    let query = Utf8Str::new(inbox_query);
    let start = 0;
    let end = query.char_count;
    search(&query, start, end)
}

// Receive query message, search, put results in outbox.
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
