// ime_engine CLI demo
use ime_engine;

// Minimal example of using ime_engine as library with std and CLI
fn main() {
    let queries = &[
        &"woxiangheguozhi",
        &"woxiang heguozhi",
        &"woxiang he guozhi",
        &"woxianheguozhi11",
    ];
    // Make a stack allocated string buffer with the Writer trait
    // that ime_engine::query() expects
    let mut sink = ime_engine::BufWriter::new();
    // Run the queries
    for q in queries.iter() {
        println!("\n{}\n{}", q, ime_engine::query(q, &mut sink));
        sink.rewind();
    }
}
