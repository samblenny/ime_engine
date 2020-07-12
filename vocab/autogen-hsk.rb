#!/usr/bin/ruby
# coding: utf-8
require 'erb'
require 'set'

RUST_FILE = "../src/autogen_hsk.rs"
HSK1 = {src: "hsk1.tsv", qc: "hsk1-QC-do-not-edit.tsv"}

# Returns array: [[hanzi, pinyin], [hanzi, pinyin], ...]
def read_tsv(file)
  File.read(file).lines.map { |n| n.chomp.split("\t") }
end

# Make a string of each unique character used in pinyin from <file>.
# Result should match TR_FROM (see below)
def detect_chars(file)
  Set.new(read_tsv(file).map { |_, pinyin| pinyin.downcase.chars }
            .flatten).to_a.sort.join("")
end

# Normalize pinyin to lowercase ASCII (remove diacritics/whitespace/punctuation).
TR_FROM = " 'abcdefghijklmnopqrstuwxyzàáèéìíòóùúāēěīōūǎǐǒǔǚ"
TR_TO   = " 'abcdefghijklmnopqrstuwxyzaaeeiioouuaeeiouaiouv"
ELIDE   = " '"
def normalize(pinyin)
  n = pinyin.downcase.delete(ELIDE).tr(TR_FROM, TR_TO)
  abort "Error: normalize(#{pinyin}) gave #{n} (non-ascii). Check TR_FROM & TR_TO." if !n.ascii_only?
  return n
end

# Check integrity and coverage of the character transposition table
detected = detect_chars(HSK1[:src])
if detected != TR_FROM
  warn "Error: Characters used in pinyin of #{HSK1[:src]} do not match TR_FROM"
  warn " detected: \"#{detected}\""
  warn " TR_FROM:  \"#{TR_FROM}\""
  abort "You need to update TR_FROM and TR_TO so pinyin will properly normalized to ASCII"
end
abort "Error: Check for TR_FROM/TR_TO length mismatch" if TR_FROM.size != TR_TO.size

# Generate a quality check TSV file for manually checking the normalized pinyin
print "This will overwrite #{HSK1[:qc]} and #{RUST_FILE}\nProceed? [y/N]: "
abort "no changes made" if !["y", "Y"].include? gets.chomp
File.open(HSK1[:qc], "w") { |qc|
  for hanzi, pinyin in read_tsv(HSK1[:src])
    qc.puts "#{hanzi}\t#{pinyin}\t#{normalize(pinyin)}"
  end
}

# Generate rust source code with hanzi and pinyin arrays
File.open(RUST_FILE, "w") { |rf|
  hanzi, pinyin = read_tsv(HSK1[:src]).transpose
  pinyin.map! { |word| normalize(word) }
  TEMPLATE = <<~RUST
    pub const HANZI: &[&'static str] = &[
    <% hanzi.each do |h| %>    &"<%= h %>",
    <% end %>];

    pub const PINYIN: &[&'static str] = &[
    <% pinyin.each do |p| %>    &"<%= p %>",
    <% end %>];
    RUST
  rf.puts ERB.new(TEMPLATE).result(binding)
}
