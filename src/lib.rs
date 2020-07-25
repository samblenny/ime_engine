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
fn copy_to_outbox(message: &str, mut total_bytes: usize) -> usize {
    unsafe {
        for b in message.bytes() {
            // TODO: better strategy for overflow (vs. silently drop extra)
            if total_bytes + 1 >= MAILBOX_SIZE {
                break;
            }
            WASM_OUTBOX[total_bytes] = b;
            total_bytes += 1;
        }
    }
    total_bytes
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

// Look up 词语 for search query (pinyin keys are ASCII, but inbox is UTF-8).
// Side-effect: copies utf8 result string to outbox buffer (append possible).
// Returns: cumulative bytes of result copied to outbox (recursion possible).
#[no_mangle]
fn look_up(inbox_query: &str, outbox_total_bytes: usize) -> usize {
    // Find start (inclusive lower bound) and end (exclusive upper bound) byte
    // indexes for slicing each UTF-8 character of the query string.
    let mut c_start_list: [usize; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
    let mut c_end_list: [usize; MAILBOX_SIZE] = [0; MAILBOX_SIZE];
    let mut query_char_count = 0;
    // Note: implicit `c_start_list[0] = 0;` from declaration above
    for i in 1..inbox_query.len() + 1 {
        if inbox_query.is_char_boundary(i) {
            query_char_count += 1;
            c_end_list[query_char_count - 1] = i;
            if i < MAILBOX_SIZE {
                c_start_list[query_char_count] = i;
            }
        }
    }

    // Recursively look up the longest matching substring of the query, starting from start_char.
    // Side-effect: copy (append) matching substrings to the outbox buffer.
    // Returns: total bytes copied to the outbox buffer.
    fn recursive_slice_query(
        inbox_query: &str,
        c_start_list: &[usize; MAILBOX_SIZE],
        c_end_list: &[usize; MAILBOX_SIZE],
        start_char: usize,
        end_char: usize,
        outbox_total_bytes: usize,
    ) -> usize {
        for i in 0..(end_char - start_char) {
            let start = c_start_list[start_char];
            let end = c_end_list[end_char - i];
            let query_slice = &inbox_query[start..end];
            match autogen_hsk::PINYIN.binary_search(&query_slice) {
                Ok(ciyu_i) => {
                    let outbox_total_bytes =
                        copy_to_outbox(&autogen_hsk::CIYU[ciyu_i], outbox_total_bytes);
                    if i == 0 {
                        // Full match
                        return outbox_total_bytes;
                    } else {
                        let start_char = end_char - i + 1;
                        return recursive_slice_query(
                            inbox_query,
                            c_start_list,
                            c_end_list,
                            start_char,
                            end_char,
                            outbox_total_bytes,
                        );
                    }
                }
                Err(_) => {}
            }
        }
        return outbox_total_bytes;
    }

    let start_char = 0;
    let end_char = query_char_count - 1;
    recursive_slice_query(
        inbox_query,
        &c_start_list,
        &c_end_list,
        start_char,
        end_char,
        outbox_total_bytes,
    )
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
    let outbox_total_bytes = 0;
    look_up(&inbox_query, outbox_total_bytes)
}
