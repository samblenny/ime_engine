#![no_std]

// Static word list arrays generated by vocab precompute ruby script
// CiyuIndex is type for phrases listed in autogen_hsk::CIYU array
mod autogen_hsk;
type CiyuIndex = usize;

// IPC function bindings and static IPC shared memory stuff
pub mod wasm;
use wasm::ipc_mem;

// For wasm32 build, use debug trace WebAssembly IPC function binding
#[cfg(target_arch = "wasm32")]
use wasm::no_std_bindings::js_log_trace;

// For other builds, replace debug trace binding with stub
#[cfg(not(target_arch = "wasm32"))]
unsafe fn js_log_trace(_: i32) {}

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
    char_start_list: [usize; ipc_mem::BUF_SIZE],
    char_end_list: [usize; ipc_mem::BUF_SIZE],
    char_count: usize,
}
impl<'a> Utf8Str<'a> {
    #[no_mangle]
    pub fn new(str_slice: &'a str) -> Utf8Str {
        // Find start (inclusive lower bound) and end (exclusive upper bound) byte
        // index of each UTF-8 character in string slice
        let mut char_start_list: [usize; ipc_mem::BUF_SIZE] = [0; ipc_mem::BUF_SIZE];
        let mut char_end_list: [usize; ipc_mem::BUF_SIZE] = [0; ipc_mem::BUF_SIZE];
        let mut char_count = 0;
        for i in 1..str_slice.len() + 1 {
            if str_slice.is_char_boundary(i) {
                if char_count + 1 < ipc_mem::BUF_SIZE {
                    char_start_list[char_count + 1] = i;
                }
                if char_count < ipc_mem::BUF_SIZE {
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
        if start < end && end <= ipc_mem::BUF_SIZE {
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
    const TOKEN_QUEUE_SIZE: usize = crate::ipc_mem::BUF_SIZE;
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
        pub fn render_and_write(&mut self, sink: &mut impl super::Writer) {
            let mut current = 0;
            let mut utf8_buf = [0u8; 4];
            while current < self.count {
                match self.queue[current] {
                    Token::CiOne(ciyu_i) => sink.write(&crate::autogen_hsk::CIYU[ciyu_i]),
                    Token::CiOpenChoice(ciyu_i) => {
                        let ciyu = &crate::autogen_hsk::CIYU[ciyu_i];
                        // Look ahead for choice pick
                        let mut choice_resolved = false;
                        for i in current..self.count {
                            if let Token::MaybeChoice(tk) = self.queue[i] {
                                match crate::expand_choice_and_write(ciyu, tk, sink) {
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
                            let _ = crate::expand_choice_and_write(ciyu, '0', sink);
                        }
                    }
                    // Space or number (pass through since not consumed by a CiOpenChoice)
                    Token::MaybeChoice(tk) => sink.write(tk.encode_utf8(&mut utf8_buf)),
                    Token::Other(tk) => sink.write(tk.encode_utf8(&mut utf8_buf)),
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
fn expand_choice_and_write(
    ciyu: &str,
    maybe_choice: char,
    sink: &mut impl Writer,
) -> ExpandChoiceResult {
    let n = ciyu.split("\t").count();
    if n == 1 {
        // If this ever happens, there's a bug. Log and recover.
        trace(901);
        sink.write(ciyu);
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
                sink.write(choice);
                return ExpandChoiceResult::WasChoice;
            }
        }
        // Out of range for possible choice, so return without sink.write() to
        // prevent duplicate choice prompting
        return ExpandChoiceResult::WasNotChoice;
    }
    // Show all choices
    sink.write(&" (");
    for (i, choice) in ciyu.split("\t").enumerate() {
        sink.write(match i {
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
        sink.write(choice);
        if i + 1 < n {
            sink.write(&" ");
        }
    }
    sink.write(&") ");
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
// Side-effect: copies utf8 result string to IPC out buffer.
fn look_up(query_bytes: &str, sink: &mut impl Writer) {
    let query = Utf8Str::new(query_bytes);
    let mut queue = lex::TokenQueue::new();
    let start = 0;
    let end = query.char_count;
    search(&query, &mut queue, start, end);
    queue.render_and_write(sink);
}

// Writer decouples query response formatting from shared memory IPC stuff.
pub trait Writer {
    fn write(&mut self, message: &str);
    fn to_s(&self) -> &str;
}

// IPCWriter is a Writer for UTF-8 bytes backed by static IPC shared memory.
struct IPCWriter {}
impl Writer for IPCWriter {
    fn write(&mut self, message: &str) {
        ipc_mem::write(message);
    }
    fn to_s(&self) -> &str {
        ipc_mem::out_to_s()
    }
}

// BufWriter is a Writer for string slices backed by stack allocated [u8].
pub struct BufWriter {
    buf: [u8; ipc_mem::BUF_SIZE],
    buf_pos: usize,
}
impl BufWriter {
    // Return empty buffer ready for use.
    pub fn new() -> BufWriter {
        BufWriter {
            buf: [0; ipc_mem::BUF_SIZE],
            buf_pos: 0,
        }
    }
    // Truncate buffer position back to 0 bytes.
    pub fn rewind(&mut self) {
        self.buf_pos = 0;
    }
}
impl Writer for BufWriter {
    // Append message to buffer
    fn write(&mut self, message: &str) {
        for b in message.bytes() {
            // TODO: better strategy for overflow (vs. silently drop extra)
            if self.buf_pos < self.buf.len() {
                self.buf[self.buf_pos] = b;
                self.buf_pos += 1;
            }
        }
    }
    // Return string slice of buffer contents.
    fn to_s(&self) -> &str {
        match core::str::from_utf8(&self.buf[0..self.buf_pos]) {
            Ok(s) => &s,
            Err(_) => &"", // TODO: handle mal-formed utf8 strings better
        }
    }
}

// Look up query, write results to sink.
// This is for calling as a library function from rust.
// Returns: string slice of results backed by sink.
pub fn query<'a>(qry: &str, sink: &'a mut impl Writer) -> &'a str {
    look_up(&qry, sink);
    sink.to_s()
}

// Receive query message, search, write results to IPC out buffer.
// This is for calling from Javascript with WebAssembly.
// Returns: number of bytes written to IPC out buffer.
#[no_mangle]
pub extern "C" fn query_shared_mem_ipc(n: usize) -> usize {
    let mut ipc_writer = IPCWriter {};
    let qry = ipc_mem::get_query(n);
    ipc_mem::rewind();
    look_up(&qry, &mut ipc_writer);
    ipc_mem::position()
}

#[cfg(test)]
mod tests {
    use super::wasm::ipc_mem;
    use super::BufWriter;

    // Send query string to ime-engine; THIS IS NOT THREAD SAFE.
    // Returns: reply string.
    fn query(qry: &str) -> &str {
        // Encode UTF-8 bytes to inbox buffer
        let mut i: usize = 0;
        unsafe {
            for b in qry.bytes() {
                if i < ipc_mem::BUF_SIZE {
                    ipc_mem::IN[i] = b;
                    i += 1;
                }
            }
        }
        // Run query
        let ipc_query_len = i;
        let _ = crate::query_shared_mem_ipc(ipc_query_len);
        // Decode reply string as UTF-8 byts from outbox
        let ipc_reply = ipc_mem::out_to_s();
        // Run the same query using the rust string slice function
        let mut sink = BufWriter::new();
        let reply = super::query(&qry, &mut sink);
        // Make sure the reply matches the ipc version
        assert_eq!(reply, ipc_reply);
        // Cannot return reply here since is owned by this function
        ipc_reply
    }

    #[test]
    fn min_query() {
        assert_eq!("", query(&""));
    }

    #[test]
    fn max_query() {
        let buf_max = ['A' as u8; ipc_mem::BUF_SIZE];
        let qry_max = core::str::from_utf8(&buf_max).unwrap();
        // This should be passed through unchanged as ASCII
        assert_eq!(qry_max, query(qry_max));
    }

    #[test]
    fn max_query_plus_1_truncate() {
        let buf_max = ['A' as u8; ipc_mem::BUF_SIZE];
        let qry_max = core::str::from_utf8(&buf_max).unwrap();
        let buf_1_too_big = ['A' as u8; ipc_mem::BUF_SIZE + 1];
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
