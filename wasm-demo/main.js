"use strict";

// Load WASM module with feature detection, then call wasmDemo()
if ("instantiateStreaming" in WebAssembly) {
    // The new, more efficient way
    WebAssembly.instantiateStreaming(fetch('ime-engine.wasm'))
        .then(wasmDemo);
} else {
    // Fallback for Safari
    fetch('ime-engine.wasm')
        .then(response => response.arrayBuffer())
        .then(bytes => WebAssembly.instantiate(bytes))
        .then(wasmDemo);
}

// Look up the hanzi for a list of pinyin search keys
function wasmDemo(result) {
    // Text output goes to pre#stdout
    let preStdout = document.querySelector("#stdout");

    // Shared memory bindings for JS <--> WASM message passing using UTF-8 strings
    let exports = result.instance.exports;
    let wasmShared = new Uint8Array(exports.memory.buffer);
    let wasmInbox = exports.wasm_inbox_ptr();
    let wasmOutbox = exports.wasm_outbox_ptr();
    let mailboxMaxSize = exports.wasm_mailbox_size();

    // send is a synchronous message passing function
    //   str: string to be sent from JS --> WASM
    //   returns: reply string from WASM --> JS
    let utf8dec = new TextDecoder();
    let utf8enc = new TextEncoder();
    function send(str) {
        let utf8Message = utf8enc.encode(str);
        let inboxMsgSize = 0;
        for (let i=0; i<utf8Message.length && i<mailboxMaxSize; i++) {
            wasmShared[wasmInbox+i] = utf8Message[i];
            inboxMsgSize += 1;
        }
        let outboxMsgSize = exports.exchange_messages(inboxMsgSize);
        return utf8dec.decode(wasmShared.subarray(wasmOutbox, wasmOutbox + outboxMsgSize));
    }

    // Send search keys to the WASM module and log the response text in pre#stdout
    let lines = [];
    let pinyinSearchKeys = ["he", "hao", "na", "shi", "dianshi", "jintian", "shui", "mianbao"];
    for (const messageToWASM of pinyinSearchKeys) {
        let replyToJS = send(messageToWASM);
        lines.push("[js->wasm]: \"" + messageToWASM + "\"\t[wasm->js]: \"" + replyToJS + "\"");
    }
    preStdout.textContent = lines.join("\n");
}
