// Shared memory buffers for interprocess communication between
// WebAssembly VM host (javscript) and WASM module (ime-engine)
// These ARE NOT thread safe! Be careful!
pub const BUF_SIZE: usize = 150;
pub static mut IN: [u8; BUF_SIZE] = [0; BUF_SIZE];
pub static mut OUT: [u8; BUF_SIZE] = [0; BUF_SIZE];
pub static mut OUT_POS: usize = 0;

// Reset the out buffer position to zero.
pub fn rewind() {
    unsafe {
        OUT_POS = 0;
    }
}

// Append copy of message into out buffer.
// Side-effect: Update out-buffer and out-buffer byte count (position).
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
