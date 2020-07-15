import { loadIMEEngineWasm, syncMessages } from './ime-engine.js';

// Look up the hanzi for a list of pinyin search keys
function wasmDemo() {
    // Text output goes to pre#stdout
    let preStdout = document.querySelector("#stdout");

    // Send search keys to the WASM module and log the response text in pre#stdout
    let lines = [];
    let pinyinSearchKeys = ["he", "hao", "na", "ta", "chu", "dianshi", "jintian", "shui", "mianbao"];
    for (const messageToWASM of pinyinSearchKeys) {
        let replyToJS = syncMessages(messageToWASM);
        lines.push("  <js> " + messageToWASM);
        lines.push("<wasm> " + replyToJS);
    }
    preStdout.textContent = lines.join("\n");
}

loadIMEEngineWasm(wasmDemo);
