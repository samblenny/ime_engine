// Relative URL to ime-engine WASM module
const wasmModule = "ime-engine.wasm";

// Load ime-engine WASM module, bind shared memory for mailboxes, then invoke callback
export function loadIMEEngineWasm(callback) {
    if ("instantiateStreaming" in WebAssembly) {
        // The new, more efficient way
        WebAssembly.instantiateStreaming(fetch(wasmModule))
            .then(initMailboxBindings)
            .then(callback);
    } else {
        // Fallback for Safari
        fetch(wasmModule)
            .then(response => response.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes))
            .then(initMailboxBindings)
            .then(callback);
    }
}

// Shared memory bindings for mailbox buffers for JS <--> WASM message passing
var wasmShared;
var wasmInbox;
var wasmOutbox;
var wasmMailboxMaxSize;
var wasmExchangeMessages;
var wasmInstanceReady = false;

// Callback to initialize shared memory mailbox bindings once WASM module is instantiated
function initMailboxBindings(result) {
    let exports = result.instance.exports;
    wasmShared = new Uint8Array(exports.memory.buffer);
    wasmInbox = exports.wasm_inbox_ptr();
    wasmOutbox = exports.wasm_outbox_ptr();
    wasmMailboxMaxSize = exports.wasm_mailbox_size();
    wasmExchangeMessages = exports.exchange_messages;
    wasmInstanceReady = true;
}

// UTF-8 string <--> byte buffer encoder and decoder
const utf8enc = new TextEncoder();
const utf8dec = new TextDecoder();

// Synchronous message passing function for exchanging UTF-8 strings across WebAssembly VM sandbox boundary
//   str: string to be sent from JS --> WASM
//   return: reply string from WASM --> JS
export function syncMessages(str) {
    if (!wasmInstanceReady) {
        throw "syncMessages cannot talk to ime-engine.wasm because the wasm instance is not ready";
    }
    let utf8Message = utf8enc.encode(str);
    let inboxMsgSize = 0;
    for (let i=0; i<utf8Message.length && i<wasmMailboxMaxSize; i++) {
        wasmShared[wasmInbox+i] = utf8Message[i];
        inboxMsgSize += 1;
    }
    let outboxMsgSize = wasmExchangeMessages(inboxMsgSize);
    return utf8dec.decode(wasmShared.subarray(wasmOutbox, wasmOutbox + outboxMsgSize));
}
