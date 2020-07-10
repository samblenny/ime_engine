# IME-Engine

IME-Engine is a toolkit for developers who want to embed a lightweight,
customizable IME into other software, such as a language learning web app.

**PLEASE NOTE: This is just a hobby project. I hope to implement the vision I
describe below in the not-too-distant future, but... no promises. For now, please
consider this vaporware. I'll remove this note if I make sufficient progress.**


## Non-Features

1. This is not a full IME. Among other things, you'll need to provide your own
   UI.

2. This is not a rust crate of the sort where you can just import it and use
   all the functionality through API calls. The workflow for customizing vocab
   lists includes source code generation. I'm talking about stuff like
   pre-computing hash tables and bloom filters that get packed into array
   literals.


## What problems does IME-Engine hope to solve?

1. **Embeddable IME for web apps:** Provide a small, simple, fast IME back-end
   that can be packaged as a WASM module for use in an SRS vocabulary web app
   with fill-in-the-blank questions.

2. **Match IME spelling to textbook spelling:** Provide a way to avoid spelling
   confusion between IME and textbook. For vocabulary practice software,
   students can get stuck if the IME does not respond appropriately to
   correctly typed romanized spellings from the textbook.


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

**Romanization:** Text from a writing system that does not use the Latin
alphabet can be *romanized* by following some system of rules for transliterating
that writing into words spelled with the Latin alphabet.

**SRS:** Spaced Repetition (SRS) is a system developed by Dr. Piotr Wozniak for
efficiently learning and remembering large vocabulary lists.
