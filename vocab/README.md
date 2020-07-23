# Vocab Data Entry README

## Pinyin IME Conventions

Problem: Which keys do you press to cause an IME to produce text that you see
printed in a book?

This is how the macOS Pinyin IME works.

In the table, `SPC` means press the spacebar, `RET` means press the "return"
key, and `OPT-a` means press the "a" key while holding down the "option" key.

| To type | for      | using IME mode | press these keys.           | Result   |
|---------|----------|----------------|-----------------------------|----------|
| 汉字    | nǚ'ér    | Pinyin         | `n v e r SPC`               | 女儿     |
| ASCII   | nǚ'ér    | Pinyin         | `n v e r RET`               | nver     |
| ASCII   | kàn jiàn | Pinyin         | `k a n RET SPC j i a n RET` | kan jian |
| Pinyin  | lǜ       | ABC-Extended   | `l OPT-v v`                 | lǚ       |
| Pinyin  | ü        | ABC-Extended   | `OPT-u u`                   | ü        |
| Pinyin  | ā        | ABC-Extended   | `OPT-a a`                   | ē        |
| Pinyin  | á        | ABC-Extended   | `OPT-e a`                   | á        |
| Pinyin  | ǎ        | ABC-Extended   | `OPT-v a`                   | ǎ        |
| Pinyin  | à        | ABC-Extended   | ```OPT-` a```               | à        |


## Text Editors

The macOS Pinyin IME mostly works fine with emacs, but the ABC-Extended key
combos for diacritics conflict with emacs meta shortcuts. BBEdit works well
for typing Hanyu Pinyin with diacritics using the ABC-Extended IME mode.
