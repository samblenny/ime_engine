"use strict";
import { loadIMEEngineWasm, syncMessages } from './ime-engine.js';

// HTML UI elements for a chat window
const chatLog = document.querySelector("#log");
const suggest = document.querySelector("#suggest");
const compose = document.querySelector("#compose");

// Scrollback buffer for chat history
const historySize = 100;
const historyBuffer = [];

// Append plain text to chat log and scroll so it is visible.
// This is uses `Node.textContent = ...` to escape user input (anti-XSS precaution).
function chatLogPlainText(unsafeMessage) {
    let p = document.createElement('p');
    p.textContent = unsafeMessage;
    chatLog.insertAdjacentElement('beforeend', p);
    chatLog.scrollTop = chatLog.scrollHeight;
}

// Append HTML to chat log and scroll so it is visible.
// CAUTION: No XSS protection! Unsafe for user input.
//          Only use this with safe HTML from internal functions.
function chatLogSafeHTML(html) {
    chatLog.insertAdjacentHTML('beforeend', `<div class="cmd">${html}</div>`);
    chatLog.scrollTop = chatLog.scrollHeight;
}

function showHelp() {
    // To test VoiceOver on macOS, use Safari. Chrome glitches hard. VoiceOver on iOS works for bilingual.
    // The "<span>/</span>help..." stuff causes VoiceOver to say "slash help" instead of just "help".
    chatLogSafeHTML('<h2>Help</h2>\n' +
                    '<p>Pinyin Tips:<br>\n' +
                    '<ul><li> Use lowercase. </li>\n' +
                    '<li> Omit tone marks. For á, type <strong>a</strong> </li>\n' +
                    '<li> Umlaut is special. For ü, type <strong>v</strong> </li>\n' +
                    '<li> For choices like (1喝 2和 3河), pick with numbers or space </li>\n' +
                    '<li> Send with return or enter. </li> </ul>\n' +
                    '<p> Example: <br>\n' +
                    '&nbsp; "woxiang he guozhi", plus return, makes "<span lang="zh-CN">我想喝果汁</span>" </p>\n' +
                    '<p> Slash Commands: <span>/</span>help <span>/</span>about <span>/</span>clear </p>');
}

function showAbout() {
    const GHDHref = 'https://github.com/samblenny/ime-engine/tree/master/wasm-demo/';
    const GHDLinkText = 'samblenny/ime-engine/wasm-demo';
    const GHRHref = 'https://github.com/samblenny/ime-engine/tree/master/README.md';
    const GHRLinkText = 'samblenny/ime-engine/README.md';
    const WKHref = 'https://en.wikipedia.org/wiki/Input_method';
    const WKLinkText = 'Input Method Editor';
    // Look closely: This uses both template string (`...`) and regular ('...')
    chatLogSafeHTML('<h2>About</h2>' +
                    '<p>How does this work?<br>' +
                    `&nbsp; demo source code on GitHub: <a href="${GHDHref}">${GHDLinkText}</a><br>` +
                    `&nbsp; project README on GitHub: <a href="${GHRHref}">${GHRLinkText}</a></p>` +
                    '<p>What is an IME?<br>' +
                    `&nbsp; Wikipedia's <a href="${WKHref}">${WKLinkText}</a> article</p>`);
}

function clearLog() {
    while (chatLog.firstChild) {
        chatLog.firstChild.remove();
    }
}

// Register event handlers to enable chat mode UI
function enableChatMode() {
    // Update the suggestion box for edit event
    compose.addEventListener('input', (e) => {
        // TODO: better fix (vs toLowerCase) for when iOS auto-capitalizes input text
        const jsToWasm = compose.value.toLowerCase();
        if (['/help', '/about', '/clear'].includes(jsToWasm)) {
            suggest.textContent = "[waiting for return or enter]";
        } else {
            const wasmToJs = syncMessages(jsToWasm);
            suggest.textContent = wasmToJs;
            //console.log(wasmToJs);
        }
    });
    // Update chat log for Enter/Return (send)
    compose.addEventListener('keydown', (e) => {
        if (!e.repeat && e.key == "Enter" && compose.value.trim() != "") {
            suggest.textContent = "";
            // TODO: better fix (vs toLowerCase) for when iOS auto-capitalizes input text
            const jsToWasm = compose.value.toLowerCase();
            const wasmToJs = syncMessages(jsToWasm);
            const rawInput = compose.value;
            compose.value = "";
            if ('/help' === jsToWasm) {
                showHelp();
            } else if ('/about' === jsToWasm) {
                showAbout();
            } else if ('/clear' === jsToWasm) {
                clearLog();
            } else if (wasmToJs === "") {
                // No match for the pinyin ==> use the input string
                chatLogPlainText(rawInput.trim());
            } else {
                // Pinyin match ==> use result string from wasm
                chatLogPlainText(wasmToJs);
            }
        }
    });
}

// Show a welcome message, then enable chat mode
function wasmDemo() {
    // To test VoiceOver on macOS, use Safari. VoiceOver on iOS works for bilingual.
    // The "<span>/</span>help..." stuff causes VoiceOver to say "slash help" instead of just "help".
    chatLogSafeHTML("<p>This terminal has a built in Simplified Chinese IME. " +
                    "You can type pinyin phrases, other text, or <span>/</span>commands.<br> " +
                    "Try <strong>wo xiang he guozhi</strong> or <strong>woxiangheguozhi11</strong> " +
                    "(...one one) <br> " +
                    "Try <strong><span>/</span>help</strong> or <strong><span>/</span>about</strong>.</p>");
    enableChatMode();
}

loadIMEEngineWasm(wasmDemo);
