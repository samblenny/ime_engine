"use strict";

WebAssembly.instantiateStreaming(fetch('ime-engine.wasm'))
    .then(result => {
        let exports = result.instance.exports;
        let wasmShared = new Uint8Array(exports.memory.buffer);
        let wasmInbox = exports.wasm_inbox_ptr();
        let wasmOutbox = exports.wasm_outbox_ptr();
        let mailboxMaxSize = exports.wasm_mailbox_size();
        let utf8dec = new TextDecoder();
        let utf8enc = new TextEncoder();
        let x = document.querySelector("#x");
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
        let lines = [];
        let txMessages = ["he", "hao", "na", "shi", "dianshi", "jintian", "shui", "mianbao"];
        for (const txMsg of txMessages) {
            let rxMsg = send(txMsg);
            lines.push(txMsg + "\t" + rxMsg);
        }
        x.textContent = lines.join("\n");
    });
