// Shared memory buffers for interprocess communication between
// WebAssembly VM host (javscript) and WASM module (ime-engine)
// These ARE NOT thread safe! Be careful!
pub const BUF_SIZE: usize = 150;
pub static mut IN: [u8; BUF_SIZE] = [0; BUF_SIZE];
pub static mut OUT: [u8; BUF_SIZE] = [0; BUF_SIZE];
pub static mut OUT_POS: usize = 0;

// Decode the UTF-8 query string from IN buffer.
pub fn get_query(n: usize) -> &'static str {
    unsafe {
        match core::str::from_utf8(&IN[0..n]) {
            Ok(s) => &s,
            Err(_) => &"", // TODO: handle mal-formed utf8 strings better
        }
    }
}

// Reset the OUT buffer position to zero.
pub fn rewind() {
    unsafe {
        OUT_POS = 0;
    }
}

// Append copy of message into OUT buffer, starting at OUT_POS.
// Side-effect: Update OUT buffer and OUT buffer byte count (OUT_POS, position).
// CAUTION: no_mangle here causes SIGSEGV (maybe collision on name "write"?).
pub fn write(message: &str) {
    for b in message.bytes() {
        unsafe {
            // TODO: better strategy for overflow (vs. silently drop extra)
            if OUT_POS < BUF_SIZE {
                OUT[OUT_POS] = b;
                OUT_POS += 1;
            }
        }
    }
}

// Use this to get the number of bytes that write() put in the OUT buffer
pub fn position() -> usize {
    unsafe { OUT_POS }
}
