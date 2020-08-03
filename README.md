# ime-engine

![ime-engine WASM demo screenshot](demo-screenshot.jpg)

ime-engine provides an Input Method Editor back-end (rust) and a
vocabulary data entry workflow (ruby) for developers who want to embed a
lightweight, customizable IME into other software. Try the
[WebAssembly demo](https://samblenny.github.io/ime-engine/wasm-demo/).

## Features

### Language Support
- Now: Simplified Chinese with 1200 word HSK4 level vocabulary
- Soonish: 2500 word HSK5
- Maybe Later: 5000 word HSK6, Hangul, Hiragana, Katakana

### Design
- Small and efficient (.wasm file with 1200 word vocab is 30 KB)
- Sacrifice ease of typing for better privacy (no AI stuff)
- Builds no_std and does not use heap allocation
- Intended for plain text user interfaces (CLI or terminal-style web UI)


## Try the WebAssembly Demo

![ime-engine WASM demo screenshot](demo-screenshot.jpg)

Demo web page with terminal-style UI and WebAssembly (WASM) back-end:
- <https://samblenny.github.io/ime-engine/wasm-demo/>
- code: [wasm-demo/](wasm-demo/)

Things to try typing in demo terminal:
- `/help` + return
- `woxiangheguozhi`
- `woxiang he guozhi` (spaces after g and e)
- `woxiang1he1guozhi` (ones in the middle)
- `woxiangheguozhi11` (ones at the end)
- `woxiang2he2guozhi` (twos instead of ones)


## Try the CLI Demo

To build and run the CLI demo, invoke `make run` from repository's root
directory, like this:

```
$ make run
cargo run --quiet

woxiangheguozhi
我 (1想 2向 3像 4香 5响)  (1喝 2和 3河) 果汁

woxiang heguozhi
我想 (1喝 2和 3河) 果汁

woxiang he guozhi
我想喝果汁

woxianheguozhi11
我先喝果汁
```


## Non-Features

1. This is not intended for use with OS-level IME software like Rime, IBus,
   or Fcitx. Rather, you can use ime-engine to embed lightweight, stand-alone
   IME features directly into other software.

2. This does not include AI stuff or a very large vocabulary. There is no
   auto-correct. No spell-check. No fuzzy matching. No predictive completion.
   No key logging back to a server to train ML models. Typing with ime-engine
   requires motivation, patience, and accurate spelling.

3. This does not use wasm-pack. I build the WASM module using cargo, rustc with
   wasm32-unknown-unknown, and hand-coded javascript bindings.


## What problems does IME-Engine hope to solve?

1. **Embeddable, offline, no_std IME back-end:** Provide a small, fast, IME
   back-end suitable for including in language learning apps or other focused,
   special-purpose software. 

2. **Match IME spelling to textbook spelling:** Provide a way to avoid spelling
   confusion between IME and textbook. For vocabulary practice software,
   students can get stuck if the IME does not respond appropriately to
   correctly typed romanized spellings from the textbook.

3. **Custom vocabulary data entry workflow:** Provide methods and tools to
   assist with accurately typing and quality checking vocab lists.


## For Developers

### Repository Tour

| Directory | Description |
|---|---|
| /src | Rust source; `autogen_hsk.rs` has static arrays with vocab data generated by ruby script |
| /vocab | ruby scripts and TSV text files for vocab data entry |
| /wasm-demo | HTML/CSS/JS source + `webserver.rb` ruby script for local http server |


### Install Dev Tools

| Tool | Purpose |
|--|--|
| rustup | Get rustc, cargo, and wasm32-unknown-unknown |
| ruby v2.3+ | Local web server for WebAssembly Demo + Rust code generation from vocab lists |
| GNU make | Automate long arguments needed by cargo for multi-target build and test |

Tested July 2020 on macOS Mojave and Debian Stretch.

1. Install rustc with rustup. See <https://www.rust-lang.org/tools/install>
2. Configure PATH environment variable: add `export PATH="$PATH:$HOME/.cargo/bin"`
   to .bash_profile or whatever
3. Add WebAssembly compile target: `rustup target add wasm32-unknown-unknown`
4. Make sure you have a ruby interpreter v2.3 or later: `ruby --version`
   - For macOS Mojave or Catalina, default system ruby should work fine. For Big Sur,
     scripting language runtimes will not be installed by default
     (see [Catalina release notes](https://developer.apple.com/documentation/macos-release-notes/macos-catalina-10_15-release-notes#Scripting-Language-Runtimes)).
     Big Sur is still in beta, but Apple will probably provide a command
     line tools installer option.
   - Debian may need `sudo apt install ruby`


### Run Tests

From repository root directory:

```
make test
```


### Build and run WebAssembly demo

1. From repository root directory:
   ```
   make wasm
   cd wasm-demo
   ruby webserver.rb
   ```
2. Load http://localhost:8000 in browser
3. Stop `webserver.rb` with control-c when done


### Install Emacs rust-mode (optional)

1. Install rustfmt: `rustup component add rustfmt`
1. Download rust-mode: https://github.com/rust-lang/rust-mode/releases/tag/0.4.0
2. Install rust-mode files
   ```
   cd ~/.emacs.d/lisp
   unzip ~/Downloads/rust-mode-0.4.0.zip
   ```
3. Edit .emacs
   ```
   (add-to-list 'load-path "~/.emacs.d/lisp/rust-mode-0.4.0")
   (autoload 'rust-mode "rust-mode" nil t)
   (add-to-list 'auto-mode-alist '("\\.rs\\'" . rust-mode))
   ```

To use rustfmt to format an emacs buffer: `C-c C-f`


### Customize Vocab List

1. Read `vocab/autogen-hsk.rb`. There is an array near the top to set which .tsv
   files contain vocab words. Comments describe how the .tsv fields are used.
2. On macOS, BBEdit works well for editing .tsv files. It helps to set 36 pt font
   and 12 character tab width.
3. To re-generate the vocab data static arrays in `src/autogen_hsk.rs`:
   ```
   cd vocab/
   ruby autogen-hsk.rb
   ```
   By default, the script just checks the contents of the .tsv files for duplicates
   and other problems. To update `autogen_hsk.rs`, you must answer `y` at prompt.


### Roadmap

- [x] Phase 1: Proof of concept WASM demo page with terminal-style UI and
      working IME for 1200 word HSK4 vocabulary.
- [x] Phase 2: Tests, build workflow, and documentation for using ime-engine as
      a library for building with and without no_std.
- [ ] Phase 3: Bigger vocabulary. At least HSK5 (2500). Maybe HSK6 (5000).
- [ ] Phase 4: Korean and Japanese (maybe). Perhaps add support for Hangul,
      Hiragana, and Katakana. Probably not Kanji though.
