// ime-engine CLI demo
use ime_engine;

// Send query string to ime-engine.
// Returns: reply string
fn query(qry: &str) -> &str {
    // Encode UTF-8 bytes to inbox buffer
    let mut i: usize = 0;
    unsafe {
        for b in qry.bytes() {
            if i < ime_engine::WASM_IPC_BUF_SIZE {
                ime_engine::WASM_IPC_IN[i] = b;
                i += 1;
            }
        }
    }
    // Run query
    let query_len = i;
    let reply_len = ime_engine::exchange_messages(query_len);
    // Decode reply string as UTF-8 byts from outbox
    unsafe { core::str::from_utf8(&ime_engine::WASM_IPC_OUT[0..reply_len]).unwrap() }
}

// Minimal example of using ime-engine as library with std and CLI
fn main() {
    let queries = &[
        &"woxiangheguozhi",
        &"woxiang heguozhi",
        &"woxiang he guozhi",
        &"woxianheguozhi11",
    ];
    for q in queries.iter() {
        println!("\n{}\n{}", q, query(q));
    }
}
