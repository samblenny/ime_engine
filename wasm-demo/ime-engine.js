// Relative URL to ime-engine WASM module
const wasmModule = "ime-engine.wasm";

// Load ime-engine WASM module, bind shared memory for mailboxes, then invoke callback
export function loadIMEEngineWasm(callback) {
    var importObject = {
        js: { js_warn_wasm_panic: () =>
              {
                  // The WASM panic handler calls this in the statement before a
                  // `loop {}` that will will peg the cpu at 100% if allowed to run.
                  // Since rust stable does not offer way for no_std wasm modules to
                  // halt execution after error, throw js exception here instead.
                  console.error("the wasm module panicked");
                  throw "wasm panic";
              },
              js_log_trace: (traceCode) => {
                  console.log("wasm trace code:", traceCode);
              },
            },
    };
    if ("instantiateStreaming" in WebAssembly) {
        // The new, more efficient way
        WebAssembly.instantiateStreaming(fetch(wasmModule), importObject)
            .then(initMailboxBindings)
            .then(callback);
    } else {
        // Fallback for Safari
        fetch(wasmModule)
            .then(response => response.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes, importObject))
            .then(initMailboxBindings)
            .then(callback);
    }
}

// Shared memory bindings for mailbox buffers for JS <--> WASM message passing
var wasmShared;
var wasmQueryBuf;
var wasmReplyBuf;
var wasmBufferSize;
var wasmQuerySharedMemIPC;
var wasmInstanceReady = false;

// Callback to initialize shared memory mailbox bindings once WASM module is instantiated
function initMailboxBindings(result) {
    let exports = result.instance.exports;
    wasmShared = new Uint8Array(exports.memory.buffer);
    wasmQueryBuf = exports.wasm_query_buf_ptr();
    wasmReplyBuf = exports.wasm_reply_buf_ptr();
    wasmBufferSize = exports.wasm_buffer_size();
    wasmQuerySharedMemIPC = exports.query_shared_mem_ipc;
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
    let querySize = 0;
    for (let i=0; i<utf8Message.length && i<wasmBufferSize; i++) {
        wasmShared[wasmQueryBuf+i] = utf8Message[i];
        querySize += 1;
    }
    let replySize = wasmQuerySharedMemIPC(querySize);
    if (replySize == 0) {
        return "";
    }
    return utf8dec.decode(wasmShared.subarray(wasmReplyBuf, wasmReplyBuf + replySize));
}
