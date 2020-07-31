#![no_std]
// WASM Notes:
// 1. The panic() boilerplate below allows use of no_std without wasm-bindgen
//    and wasm-pack.
// 2. Using #[no_mangle] on public functions is necessary for linking.
// 3. Using #[no_mangle] on other functions reduces binary size and helps with
//    disassembly and step tracing in browser dev tools.
#[cfg(target_arch = "wasm32")]
mod wasm32_no_std {
    #[link(wasm_import_module = "js")]
    extern "C" {
        fn js_warn_wasm_panic();
        pub fn js_log_trace(code: i32);
    }
    use core::panic::PanicInfo;
    #[panic_handler]
    #[no_mangle]
    pub fn panic(_panic_info: &PanicInfo) -> ! {
        unsafe {
            js_warn_wasm_panic();
        }
        loop {}
    }
}
#[cfg(target_arch = "wasm32")]
use wasm32_no_std::*;
#[cfg(not(target_arch = "wasm32"))]
unsafe fn js_log_trace(_: i32) {}

// Include the static arrays generated from vocab list files
mod autogen_hsk;
// Index type for phrases listed in autogen_hsk::CIYU array
type CiyuIndex = usize;

// Shared memory buffers for interprocess communication between
// WebAssembly VM host (javscript) and WASM module (ime-engine)
// These ARE NOT thread safe! Be careful!
pub const WASM_IPC_BUF_SIZE: usize = 150;
pub static mut WASM_IPC_IN: [u8; WASM_IPC_BUF_SIZE] = [0; WASM_IPC_BUF_SIZE];
pub static mut WASM_IPC_OUT: [u8; WASM_IPC_BUF_SIZE] = [0; WASM_IPC_BUF_SIZE];
static mut WASM_IPC_OUT_POS: usize = 0;

// Append copy of message into out buffer.
// Side-effect: Update out buffer and out buffer byte count (position).
#[no_mangle]
pub fn wasm_ipc_write(message: &str) {
    unsafe {
        for b in message.bytes() {
            // TODO: better strategy for overflow (vs. silently drop extra)
            if WASM_IPC_OUT_POS < WASM_IPC_BUF_SIZE {
                WASM_IPC_OUT[WASM_IPC_OUT_POS] = b;
                WASM_IPC_OUT_POS += 1;
            }
        }
    }
}

// Reset the out buffer position to zero.
fn wasm_ipc_rewind() {
    unsafe {
        WASM_IPC_OUT_POS = 0;
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
    WASM_IPC_IN.as_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn wasm_outbox_ptr() -> *const u8 {
    WASM_IPC_OUT.as_ptr()
}
#[no_mangle]
pub unsafe extern "C" fn wasm_mailbox_size() -> usize {
    WASM_IPC_BUF_SIZE
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
    char_start_list: [usize; WASM_IPC_BUF_SIZE],
    char_end_list: [usize; WASM_IPC_BUF_SIZE],
    char_count: usize,
}
impl<'a> Utf8Str<'a> {
    #[no_mangle]
    pub fn new(str_slice: &'a str) -> Utf8Str {
        // Find start (inclusive lower bound) and end (exclusive upper bound) byte
        // index of each UTF-8 character in string slice
        let mut char_start_list: [usize; WASM_IPC_BUF_SIZE] = [0; WASM_IPC_BUF_SIZE];
        let mut char_end_list: [usize; WASM_IPC_BUF_SIZE] = [0; WASM_IPC_BUF_SIZE];
        let mut char_count = 0;
        for i in 1..str_slice.len() + 1 {
            if str_slice.is_char_boundary(i) {
                if char_count + 1 < WASM_IPC_BUF_SIZE {
                    char_start_list[char_count + 1] = i;
                }
                if char_count < WASM_IPC_BUF_SIZE {
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
        if start < end && end <= WASM_IPC_BUF_SIZE {
            let start_b = self.char_start_list[start];
            // Must not allow end==0 here. For usize, (0 - 1) will panic.
            let end_b = self.char_end_list[end - 1];
            self.str_slice.get(start_b..end_b)
        } else {
            None
        }
    }
}

// Data structure for tracking lexemes of query input and their meanings.
// TokenQueue is no_std, stack-only substitute for Vec<Token>. If TokenQueue
// were Vec<Token>, it would require heap allocation and linking std.
pub mod lex {
    const TOKEN_QUEUE_SIZE: usize = crate::WASM_IPC_BUF_SIZE;
    // Holds one Token.
    #[derive(Copy, Clone)]
    pub enum Token {
        CiOne(crate::CiyuIndex),
        CiOpenChoice(crate::CiyuIndex),
        MaybeChoice(char),
        Other(char),
        Skip,
    }
    // Holds queue of Tokens (append only)
    pub struct TokenQueue {
        pub queue: [Token; TOKEN_QUEUE_SIZE],
        pub count: usize,
    }
    impl TokenQueue {
        // Initialize queue.
        pub fn new() -> TokenQueue {
            TokenQueue {
                queue: [Token::Skip; TOKEN_QUEUE_SIZE],
                count: 0,
            }
        }
        // Add Token to queue.
        pub fn push(&mut self, tk: Token) -> bool {
            if self.count < TOKEN_QUEUE_SIZE {
                self.queue[self.count] = tk;
                self.count += 1;
                true
            } else {
                // Error: Queue is full
                false
            }
        }
        // Iterate through tokens, resolve choices, render to oubox.
        // Side-effect: Send strings to outbox buffer.
        // Possible surprising behavior:
        // - Value of CiOpenChoice depends on lookahead for MaybeChoice
        // - MaybeChoice gets consumed (skipped) if used to resolve choice
        pub fn render_and_write(&mut self) {
            let mut current = 0;
            let mut utf8_buf = [0u8; 4];
            while current < self.count {
                match self.queue[current] {
                    Token::CiOne(ciyu_i) => {
                        crate::wasm_ipc_write(&crate::autogen_hsk::CIYU[ciyu_i])
                    }
                    Token::CiOpenChoice(ciyu_i) => {
                        let ciyu = &crate::autogen_hsk::CIYU[ciyu_i];
                        // Look ahead for choice pick
                        let mut choice_resolved = false;
                        for i in current..self.count {
                            if let Token::MaybeChoice(tk) = self.queue[i] {
                                match crate::expand_choice_and_write(ciyu, tk) {
                                    crate::ExpandChoiceResult::WasChoice => {
                                        self.queue[i] = Token::Skip;
                                        choice_resolved = true;
                                        break;
                                    }
                                    crate::ExpandChoiceResult::WasNotChoice => {}
                                }
                            }
                        }
                        if !choice_resolved {
                            // TODO: use enum variant instead of '0' to indicate no MaybeChoice found
                            let _ = crate::expand_choice_and_write(ciyu, '0');
                        }
                    }
                    // Space or number (pass through since not consumed by a CiOpenChoice)
                    Token::MaybeChoice(tk) => crate::wasm_ipc_write(tk.encode_utf8(&mut utf8_buf)),
                    Token::Other(tk) => crate::wasm_ipc_write(tk.encode_utf8(&mut utf8_buf)),
                    Token::Skip => {}
                }
                current += 1;
            } // end while
        } // end render_and_write()
    } // end impl TokenQueue
} // end lex

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

// Render 词语 multi-matches as resolved choice or prompt for choice.
// Side-effect: Send results to oubox buffer.
// Return: Was the maybe_choice token used to resolve a choice?
enum ExpandChoiceResult {
    WasChoice,
    WasNotChoice,
}
fn expand_choice_and_write(ciyu: &str, maybe_choice: char) -> ExpandChoiceResult {
    let n = ciyu.split("\t").count();
    if n == 1 {
        // If this ever happens, there's a bug. Log and recover.
        trace(901);
        wasm_ipc_write(ciyu);
        return ExpandChoiceResult::WasNotChoice;
    }
    // Try to pick a choice (return immediately if number out of range)
    let pick = match maybe_choice {
        ' ' => 1, // Spacebar picks default option (label=1)
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9, // TODO: Fix. This only works for <=9 choices
        _ => 0,
    };
    if pick > 0 {
        for (i, choice) in ciyu.split("\t").enumerate() {
            if i + 1 == pick {
                wasm_ipc_write(choice);
                return ExpandChoiceResult::WasChoice;
            }
        }
        // Out of range for possible choice, so return without wasm_ipc_write() to
        // prevent duplicate choice prompting
        return ExpandChoiceResult::WasNotChoice;
    }
    // Show all choices
    wasm_ipc_write(&" (");
    for (i, choice) in ciyu.split("\t").enumerate() {
        wasm_ipc_write(match i {
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
        wasm_ipc_write(choice);
        if i + 1 < n {
            wasm_ipc_write(&" ");
        }
    }
    wasm_ipc_write(&") ");
    return ExpandChoiceResult::WasNotChoice;
}

// Search for 词语 matches in substrings of query.
// Side-effect: Send results to outbox buffer.
#[no_mangle]
fn search(query: &Utf8Str, queue: &mut lex::TokenQueue, mut start: usize, end: usize) {
    while start < end {
        // Limit window size to length of longest phrase in pinyin array
        let window_end = min(start + autogen_hsk::PINYIN_SIZE_MAX, end);
        if let Some((ciyu_i, match_end)) = longest_match(query, start, window_end) {
            // Got Match: push match, continue search in remainder of query
            if autogen_hsk::CIYU[ciyu_i].contains("\t") {
                queue.push(lex::Token::CiOpenChoice(ciyu_i));
            } else {
                queue.push(lex::Token::CiOne(ciyu_i));
            }
            start = match_end;
        } else {
            // No match... push one character, continue search in remainder of query
            if let Some(s) = query.char_slice(start, start + 1) {
                // TODO: Better solution than silently ignoring possible full queue
                let _ = match s {
                    " " => queue.push(lex::Token::MaybeChoice(' ')),
                    "1" => queue.push(lex::Token::MaybeChoice('1')),
                    "2" => queue.push(lex::Token::MaybeChoice('2')),
                    "3" => queue.push(lex::Token::MaybeChoice('3')),
                    "4" => queue.push(lex::Token::MaybeChoice('4')),
                    "5" => queue.push(lex::Token::MaybeChoice('5')),
                    "6" => queue.push(lex::Token::MaybeChoice('6')),
                    "7" => queue.push(lex::Token::MaybeChoice('7')),
                    "8" => queue.push(lex::Token::MaybeChoice('8')),
                    "9" => queue.push(lex::Token::MaybeChoice('9')),
                    _ => {
                        if let Some(c) = s.chars().nth(0) {
                            queue.push(lex::Token::Other(c))
                        } else {
                            // If this ever happens, there's a bug.
                            trace(902);
                            false
                        }
                    }
                };
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
    let mut queue = lex::TokenQueue::new();
    let start = 0;
    let end = query.char_count;
    search(&query, &mut queue, start, end);
    queue.render_and_write();
}

// Receive query message, search, put results in outbox.
#[no_mangle]
pub extern "C" fn exchange_messages(n: usize) -> usize {
    let inbox_query: &str;
    unsafe {
        inbox_query = match core::str::from_utf8(&WASM_IPC_IN[0..n]) {
            Ok(s) => &s,
            Err(_) => &"", // TODO: handle mal-formed utf8 strings better
        };
    }
    // TODO: better way to track bytes count for the outbox buffer
    wasm_ipc_rewind();
    look_up(&inbox_query);
    unsafe {
        return WASM_IPC_OUT_POS;
    }
}

#[cfg(test)]
mod tests {
    // Send query string to ime-engine; THIS IS NOT THREAD SAFE.
    // Returns: reply string
    fn query(qry: &str) -> &str {
        // Encode UTF-8 bytes to inbox buffer
        let mut i: usize = 0;
        unsafe {
            for b in qry.bytes() {
                if i < crate::WASM_IPC_BUF_SIZE {
                    crate::WASM_IPC_IN[i] = b;
                    i += 1;
                }
            }
        }
        // Run query
        let query_len = i;
        let reply_len = crate::exchange_messages(query_len);
        // Decode reply string as UTF-8 byts from outbox
        unsafe { core::str::from_utf8(&crate::WASM_IPC_OUT[0..reply_len]).unwrap() }
    }

    #[test]
    fn min_query() {
        assert_eq!("", query(&""));
    }

    #[test]
    fn max_query() {
        let buf_max = ['A' as u8; crate::WASM_IPC_BUF_SIZE];
        let qry_max = core::str::from_utf8(&buf_max).unwrap();
        // This should be passed through unchanged as ASCII
        assert_eq!(qry_max, query(qry_max));
    }

    #[test]
    fn max_query_plus_1_truncate() {
        let buf_max = ['A' as u8; crate::WASM_IPC_BUF_SIZE];
        let qry_max = core::str::from_utf8(&buf_max).unwrap();
        let buf_1_too_big = ['A' as u8; crate::WASM_IPC_BUF_SIZE + 1];
        let qry_1_too_big = core::str::from_utf8(&buf_1_too_big).unwrap();
        // This should truncate the query
        assert_eq!(qry_max, query(qry_1_too_big));
    }

    #[test]
    fn choice_xiang1() {
        assert_eq!("想", query(&"xiang1"));
    }

    #[test]
    fn zhang3chang2() {
        assert!(query(&"zhang").contains("长"));
        assert!(query(&"chang").contains("长"));
    }

    #[test]
    fn query_all_pinyin_search_keys_verify_ciyu() {
        let test_data = &crate::autogen_hsk::PINYIN_CIYU_TEST_DATA;
        for (normalized_pinyin, ciyu) in test_data.iter() {
            assert!(query(normalized_pinyin).contains(ciyu));
        }
    }

    #[test]
    fn choosing_ciyu_with_numbers_and_spaces() {
        assert!(query(&"xiang").contains("(1想"));
        assert!(query(&"xiang").contains("2向"));
        assert_eq!(query(&"xiang "), "想");
        assert!(query(&" xiang").starts_with(" "));
        assert!(query(&" xiang").contains("(1想"));
        assert_eq!(query(&"xiang1"), "想");
        assert_eq!(query(&"xiang2"), "向");
        assert!(query(&"xianghe").contains("(1想"));
        assert!(query(&"xianghe").contains("2向"));
        assert!(query(&"xianghe").contains("(1喝"));
        assert!(query(&"xianghe").contains("2和"));
        assert!(query(&"xiang he").starts_with("想"));
        assert!(query(&"xiang he").contains("(1喝"));
        assert!(query(&"xiang1he").starts_with("想"));
        assert!(query(&"xiang1he").contains("(1喝"));
        assert!(query(&"xianghe1").starts_with("想"));
        assert!(query(&"xianghe1").contains("(1喝"));
        assert!(query(&"xianghe ").starts_with("想"));
        assert!(query(&"xianghe ").contains("(1喝"));
        assert_eq!(query(&"xianghe 1"), "想喝");
        assert_eq!(query(&"xianghe11"), "想喝");
        assert_eq!(query(&"xiang he1"), "想喝");
        assert_eq!(query(&"xiang he "), "想喝");
        assert_eq!(query(&"xianghe 2"), "想和");
    }

    #[test]
    fn query_chars_not_matched_should_pass_through() {
        assert_eq!(query(&"🐇✨"), "🐇✨");
        assert_eq!(query(&"baiSEde🐇✨11"), "白SE的🐇✨");
        assert_eq!(query(&"RABBIT SPARKLES 11"), "RABBIT SPARKLES 11");
        assert_eq!(query(&"XIANGHE"), "XIANGHE");
    }
}
