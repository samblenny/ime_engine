# IME-Engine

IME-Engine provides an IME back-end (rust) and a vocabulary data entry
workflow (ruby) for developers who want to embed a lightweight, customizable
IME into other software.

**PLEASE NOTE: This is just a hobby project. I hope to implement the vision I
describe below in the not-too-distant future, but... no promises. For now,
please consider this vaporware. I will remove this note if I make sufficient
progress.**


## WASM Demo

To view the WASM demo for my current progress:

- hosted page: <https://samblenny.github.io/ime-engine/www/>
- code: [www/](www/) directory in this repo


## Non-Features

1. This is not a full IME.

2. This does not include AI stuff or a very large, general-purpose vocabulary.
   There is no auto-correct or fuzzy predictive completion.

3. This is not a rust crate of the sort where you just add the dependency and
   start making API calls. The workflow for customizing vocab lists uses ruby
   scripts to pre-compute arrays and generate rust source code. Ruby v2.3 or
   later should work on macOS and linux (windows not tested). No gems needed.

4. This does not use wasm-pack. I build the WASM module with cargo, rustc's
   wasm32-unknown-unknown build target, and hand-coded javascript bindings.


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

My phase 1 goal is to build a rust library for translating Hanyu Pinyin from
the HSK1 vocabulary list into Simplified Chinese characters. If that works, I
want to expand the vocabulary list to HSK3 or maybe HSK4. I might also try
adding Hangul, Hiragana, and Katakana.


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
