# Vocab Data Entry README

## Pinyin IME Conventions

Problem: Which keys do you press to cause an IME to produce text that you see
printed in a book?

This is how the macOS Pinyin IME works.

In the table, `<space>` means press the spacebar, `<return>` means press the
"return" key, and `<option>-a` means press the "a" key while holding down the
"option" key.

| To type | for   | using IME mode | press these keys.   | Result |
|---------|-------|----------------|---------------------|--------|
| 汉字    | nǚ'ér | Pinyin         | `n v e r <space>`   | 女儿   |
| ASCII   | nǚ'ér | Pinyin         | `n v e r <return>`  | nver   |
| Pinyin  | lǜ    | ABC-Extended   | `l <option>-v v`    | lǚ     |
| Pinyin  | ü     | ABC-Extended   | `<option>-u u`      | ü      |
| Pinyin  | ā     | ABC-Extended   | `<option>-a a`      | ē      |
| Pinyin  | á     | ABC-Extended   | `<option>-e a`      | á      |
| Pinyin  | ǎ     | ABC-Extended   | `<option>-v a`      | ǎ      |
| Pinyin  | à     | ABC-Extended   | ```<option>-` a```  | à      |


## Text Editors

The macOS Pinyin IME mostly works fine with emacs, but the ABC-Extended key
combos for diacritics conflict with emacs meta shortcuts. BBEdit works well
for typing Hanyu Pinyin with diacritics using the ABC-Extended IME mode.
