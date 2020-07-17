import { loadIMEEngineWasm, syncMessages } from './ime-engine.js';

// HTML UI elements for a chat window
const chatLog = document.querySelector("#log");
const suggest = document.querySelector("#suggest");
const compose = document.querySelector("#compose");

// Scrollback buffer for chat history
const historySize = 100;
const historyBuffer = [];

// Update the chat log
function logMessages(jsToWasm, wasmToJs) {
    historyBuffer.push("<pinyin> " + jsToWasm);
    historyBuffer.push(" <hanzi> " + wasmToJs);
    // Discard oldest messages if buffer is full
    while (historyBuffer.length > historySize) { historyBuffer.shift(); }
    // Update UI, scroll to bottom to show new messages
    chatLog.textContent = historyBuffer.join("\n");
    chatLog.scrollTop = chatLog.scrollHeight;
}

// Register event handlers to enable chat mode UI
function enableChatMode() {
    // Update the suggestion box for edit event
    compose.addEventListener('input', (e) => {
        // TODO: better fix (vs toLowerCase) for when iOS auto-capitalizes input text
        const jsToWasm = compose.value.toLowerCase();
        suggest.textContent = (jsToWasm != "") ? syncMessages(jsToWasm) : "";
    });
    // Update chat log for Enter/Return (send)
    compose.addEventListener('keydown', (e) => {
        if (!e.repeat && e.key == "Enter" && compose.value != "") {
            suggest.textContent = "";
            // TODO: better fix (vs toLowerCase) for when iOS auto-capitalizes input text
            const jsToWasm = compose.value.toLowerCase();
            const wasmToJs = syncMessages(jsToWasm);
            compose.value = "";
            logMessages(jsToWasm, wasmToJs);
        }
    });
}

// Look up hanzi for a list of pinyin examples, then enable chat mode
function wasmDemo() {
    // Send pinyin search keys to the WASM module and log the hanzi responses
    let lines = [];
    let pinyinSearchKeys = ["hao", "na", "ta", "nin", "mai", "jintian", "shui", "mianbao"];
    for (const jsToWasm of pinyinSearchKeys) {
        const wasmToJs = syncMessages(jsToWasm);
        logMessages(jsToWasm, wasmToJs);
    }
    enableChatMode();
}

loadIMEEngineWasm(wasmDemo);
