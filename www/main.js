"use strict";

WebAssembly.instantiateStreaming(fetch('ime-engine.wasm'))
    .then(result => {
        let exports = result.instance.exports;
        let buffer = new Uint8Array(exports.memory.buffer);
        let inbox = exports.js_inbox_ptr();
        let utf8dec = new TextDecoder();
        let x = document.querySelector("#x");
        for (let n=0; n<151; n++) {
            let size = exports.keystroke_event(n);
            let message = utf8dec.decode(buffer.subarray(inbox, inbox + size));
            x.textContent = [x.textContent, message].join("\n");
        }
    });
