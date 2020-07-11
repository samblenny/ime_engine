"use strict";

var memBuf;
var msgBufPtr;
var utf8Decoder = new TextDecoder();
var x = document.querySelector("#x");

function unpack_message(msg_len) {
    let message = memBuf.subarray(msgBufPtr, msgBufPtr + msg_len);
    x.textContent = [x.textContent, utf8Decoder.decode(message)].join("\n");
};

var importObject = {js: {log: arg => console.log("log:", arg),}};

WebAssembly.instantiateStreaming(fetch('ime-engine.wasm'), importObject)
    .then(result => {
        console.log(result.instance);
        var exports = result.instance.exports;
        var request_message = exports.request_message;
        memBuf = new Uint8Array(exports.memory.buffer);
        msgBufPtr = exports.msg_buf_ptr();
        for (let n in [0,1,2,3]) {
            let msg_len = request_message(n);
            unpack_message(msg_len);
        }
    });
