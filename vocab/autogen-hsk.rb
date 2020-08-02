#!/usr/bin/ruby
# coding: utf-8
require 'erb'
require 'set'

RUST_FILE = "../src/autogen_hsk.rs"
WORD_FILES = [
  "hsk1.tsv",
  "hsk1-extra.tsv",
  "hsk2.tsv",
  "hsk2-extra.tsv",
  "hsk3.tsv",
  "hsk3-extra.tsv",
  "hsk4.tsv",
  "hsk4-extra.tsv",
]

# Returns array: [[ciyu, pinyin], [ciyu, pinyin], ...] (see note 2)
# Notes:
# 1. The `select {..."#"..."\t"}` filters out blank lines and comments
# 2. Most input lines are like "词语\tpinyin", which yields two element arrays
#    like [ciyu, pinyin]. But, some tsv input lines include part of speech and
#    meaning like, "过\tguo\tv.\tto spend, to pass", to help verify data entry
#    for ciyu with multiple meanings for the same pronunciation. The longer
#    lines yield 4 element arrays like [ciyu, pinyin, partofspeech, meaning].
def read_tsv(file)
  File.read(file).lines
    .select { |n| !n.start_with?("#") && n.include?("\t") }
    .map { |n| n.chomp.split("\t") }
end

# Make a set with each unique character used in pinyin from <file>.
def char_set(file)
  # See comment below about ruby multiple assignment semantics
  Set.new(read_tsv(file).map { |_, pinyin| pinyin.downcase.chars }.flatten)
end

# Normalize pinyin to lowercase ASCII (remove diacritics/whitespace/punctuation).
TR_FROM = " 'abcdefghijklmnopqrstuwxyzàáèéìíòóùúüāēěīōūǎǐǒǔǚ"
TR_TO   = " 'abcdefghijklmnopqrstuwxyzaaeeiioouuvaeeiouaiouv"
ELIDE   = " '"
def normalize(pinyin)
  n = pinyin.downcase.delete(ELIDE).tr(TR_FROM, TR_TO)
  abort "Error: normalize(#{pinyin}) gave #{n} (non-ascii). Check TR_FROM & TR_TO." if !n.ascii_only?
  return n
end

# Check integrity and coverage of the character transposition table.
# The map/reduce uses set algebra to build a sorted string of unique characters from all the files.
detected = WORD_FILES.map {|wf| char_set(wf)}.reduce {|a,b| a+b}.to_a.sort.join("")
if detected != TR_FROM
  warn "Error: Characters used in word file pinyin do not match TR_FROM"
  warn " detected: \"#{detected}\""
  warn " TR_FROM:  \"#{TR_FROM}\""
  abort "You need to update TR_FROM and TR_TO so pinyin will properly normalized to ASCII"
end
abort "Error: Check for TR_FROM/TR_TO length mismatch" if TR_FROM.size != TR_TO.size


# Merge ciyu values for duplicate pinyin search keys
# example: ["he", "he"] and ["喝", "和"] get turned into ["he"] and ["喝\t和"]
# Notes (subtle ruby semantics):
# 1. read_tsv() can return arrays of >=2 elements like [ciyu, pinyin, ...]
# 2. The `for ciyu, pinyin in read_tsv()` below uses ruby multiple assignment
#    to assign just the first two elements of the arrays from read_tsv(). It
#    works like a slice. Additional array elements for part of speech and
#    meaning, if present, will be ignored. The extra fields help with using
#    grep on the .tsv files to check for duplicate entries.
merged_ciyu = []
merged_pinyin = []
pinyin_ciyu_test_data = []
ciyu_choice_max = 1;
first_index_of = {}
pinyin_size_max = 0;
pinyin_char_count = 0;
pinyin_key_count = 0;
i = 0
for wf in WORD_FILES
  for ciyu, pinyin in read_tsv(wf)
    normalized_pinyin = normalize(pinyin)
    # First, save unprocessed (pinyin, 词语) pairs for generating rust test data
    pinyin_ciyu_test_data << [normalized_pinyin, ciyu]
    # Proceed with merging homophones for generating rust query lookup data
    if first_index_of[normalized_pinyin]
      if merged_ciyu[first_index_of[normalized_pinyin]].include?(ciyu)
        # If you see this warning, use grep to check for duplicate entries.
        # Verify the part of speech. For some words like 过, 等, and 省, the
        # official word list has separate entries for different meanings of the
        # same word.
        warn "Likely duplicate word: #{wf}:  #{ciyu}:#{pinyin}  ==>  try:  grep #{ciyu} *.tsv"
      end
      # Conditionally append hanzi for duplicate pinyin search key
      # 1. Skip ciyu like 过 guò with same hanzi spelling, same pinyin
      #    spelling, but different part of speech
      # 2. For the rest, append the new hanzi to the list of choices
      if !merged_ciyu[first_index_of[normalized_pinyin]].include?(ciyu)
        merged_ciyu[first_index_of[normalized_pinyin]] << ciyu
        ciyu_choice_max = [ciyu_choice_max, merged_ciyu[first_index_of[normalized_pinyin]].size].max
      end
    else
      # First instance of search key ==> Add new entries
      merged_ciyu[i] = [ciyu]
      merged_pinyin[i] = normalized_pinyin
      first_index_of[normalized_pinyin] = i
      i += 1
      pinyin_char_count += normalized_pinyin.size
      pinyin_key_count += 1;
    end
    # Is this the longest pinyin phrase so far?
    pinyin_size_max = [pinyin_size_max, normalized_pinyin.size].max
  end
end

# Sort the merged vocab lists in pinyin order
merged_pinyin, merged_ciyu = merged_pinyin.zip(merged_ciyu).sort.transpose

# Print statistics
avg_pinyin_key_len = Float(pinyin_char_count) / pinyin_key_count
puts "\nUnique pinyin search keys: #{pinyin_key_count}"
puts "Average characters per pinyin search key: #{avg_pinyin_key_len.round(1)}"

# Ask about updating the Rust array source code
puts "\nPreparing to generate rust source code..."
print "This will overwrite #{RUST_FILE}\nDo you want to continue? [y/N] "
abort "no changes made" if !["y", "Y"].include? gets.chomp

# Generate rust source code with ciyu and pinyin arrays
File.open(RUST_FILE, "w") { |rf|
  TEMPLATE = <<~RUST
    #![allow(dead_code)]
    // This file is automatically generated. DO NOT MAKE EDITS HERE!
    // To make changes, see ../vocab/autogen-hsk.rb

    // Longest homophone choice size (choices joined by "\\t")
    pub const CIYU_CHOICE_MAX: usize = <%= ciyu_choice_max %>;

    pub static CIYU: &[&'static str] = &[
    <% merged_ciyu.each do |h| %>    &"<%= h.join("\t") %>",
    <% end %>];

    // The longest phrase in the list below has PINYIN_SIZE_MAX characters.
    // No need to consider longer slices when checking for match.
    // Note: UTF-8 means both (chars == bytes) or (chars != bytes) possible.
    pub const PINYIN_SIZE_MAX: usize = <%= pinyin_size_max %>;

    pub static PINYIN: &[&'static str] = &[
    <% merged_pinyin.each do |p| %>    &"<%= p %>",
    <% end %>];

    // Tuples are (normalized_pinyin, 词语) from early in vocab file code
    // generation precompute pipeline. These correspond closely to lines of
    // vocab .tsv files prior to any sorting or merging of duplicates.
    #[cfg(test)]
    pub static PINYIN_CIYU_TEST_DATA: &[(&'static str, &'static str)] = &[
    <% pinyin_ciyu_test_data.each do |np, cy| %>    (&"<%= np %>", &"<%= cy %>"),
    <% end %>];
    RUST
  rf.puts ERB.new(TEMPLATE).result(binding)
}
