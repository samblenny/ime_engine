# ime-engine

ime-engine provides an Input Method Editor back-end (rust) and a
vocabulary data entry workflow (ruby) for developers who want to embed a
lightweight, customizable IME into other software.

Key Features:
- Small and efficient
- Builds no_std and does not use heap allocation
- Designed for plain text user interfaces (CLI or terminal-style web UI)

Currently Supported Languages:
- Simplified Chinese with 1200 word HSK4 level vocabulary


## Live Demo

You can try the IME on a demo web page built with WebAssembly (WASM):
- hosted page: <https://samblenny.github.io/ime-engine/wasm-demo/>
- code: [wasm-demo/](wasm-demo/)


## Non-Features

1. This is not intended for use with OS-level IME software like Rime, Ibus,
   or Fcitx. Rather, you can use ime-engine to embed lightweight, stand-alone
   IME features directly into other software.

2. This does not include AI stuff or a very large vocabulary. There is no
   auto-correct. No spell-check. No fuzzy predictive completion.

3. This does not use wasm-pack. I build the WASM module using cargo, rustc with
   wasm32-unknown-unknown, and hand-coded javascript bindings.


## What problems does IME-Engine hope to solve?

1. **Embeddable, offline, no_std IME back-end:** Provide a small, fast, IME
   back-end suitable for including in language learning apps or other focused,
   special-purpose software. By not depending on the rust standard library, and
   by not depending on a network communication, ime-engine can be compiled for
   platforms like WASM.

   Main intended use case: Provide IME back-end for SRS vocab practice web app
   designed for offline use (PWA) on phone or tablet.

2. **Match IME spelling to textbook spelling:** Provide a way to avoid spelling
   confusion between IME and textbook. For vocabulary practice software,
   students can get stuck if the IME does not respond appropriately to
   correctly typed romanized spellings from the textbook.

3. **Custom vocabulary data entry workflow:** Provide methods and tools to
   assist with accurately typing and quality checking vocab lists.


## Roadmap

- [x] Phase 1: Proof of concept WASM demo page with terminal-style UI and
      working IME for 1200 word HSK4 vocabulary.
- [ ] Phase 2: Tests, build workflow, and documentation for using ime-engine as
      a library for building with and without no_std.
- [ ] Phase 3: Bigger vocabulary. At least HSK5 (2500). Maybe HSK6 (5000).
- [ ] Phase 4: Korean and Japanese (maybe). Perhaps add support for Hangul,
      hiragana, and katakana. Probably not Kanji though.


## Glossary

**IME:** An input method editor (IME) is software for composing text in one
writing system by using a keyboard intended for some other writing system. For
example, you could use a pinyin or hangul IME to compose text in 汉字 or 한글
using a US-QWERTY keyboard.

**HSK:** Hanyu Shuiping Kaoshi is the official PRC Chinese language proficiency
test administered by Hanban (Confucius Institute). Hanban's HSK textbooks
include vocabulary tables of pinyin spellings for all the 汉字 on the HSK
vocabulary lists.

**PWA:** Progressive Web Application (PWA) refers to a method for using open
web standards to provide functionality similar to a phone app. Installing a PWA
is similar to making a bookmark---you do not need to go through an app store.

**Romanization:** Text from a writing system that does not use the Latin
alphabet can be *romanized* by following some system of rules for transliterating
that writing into words spelled with the Latin alphabet.

**SRS:** Spaced Repetition (SRS) is a system developed by Dr. Piotr Wozniak for
efficiently learning and remembering large vocabulary lists.

**WASM:** WebAssembly (WASM) is an open web technology for using compiled
programs in situations that previously would require using javascript. Compared
to javascript, WASM programs run faster, provide additional safety guarantees,
and can be written in languages such as rust.
