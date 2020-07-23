#!/usr/bin/ruby
require "zlib"
require "set"

ARCHIVE = "official/hsk2012.gz"

def get_official_words(key)
  buf = Zlib::GzipReader.open(ARCHIVE) { |gz| gz.read }
  target_key = Regexp.compile(/^--- #{key}/)
  other_keys = Regexp.compile(/^--- /)
  words = []
  skip = true
  for line in buf.lines
    if target_key.match line
      skip = false
    elsif other_keys.match line
      break if !skip
      skip = true
    else
      words << line.chomp if !skip
    end
  end
  return Set.new(words)
end

def get_tsv_words(filename)
  words = []
  for row in File.read(filename).lines
    hanzi, _pinyin = row.chomp.split("\t")
    words << hanzi
  end
  return words
end

# Verify contents of the official words archive file using set algebra
# to calculate how many new words occur in each level
puts "Checking archive of offical word lists..."
levels = [:hsk1, :hsk2, :hsk3, :hsk4, :hsk5, :hsk6]
words = levels.map {|k| [k, get_official_words(k.to_s)]}.to_h
new_words = {hsk1: words[:hsk1],
             hsk2: words[:hsk2] - words[:hsk1],
             hsk3: words[:hsk3] - words[:hsk2],
             hsk4: words[:hsk4] - words[:hsk3],
             hsk5: words[:hsk5] - words[:hsk4],
             hsk6: words[:hsk6] - words[:hsk5],
            }
for k in levels
  puts " #{k}: #{words[k].size} words (#{new_words[k].size} new)"
end

# Use set algebra to find differences between TSV data and official word list
def compare_tsv_to_official(tsv_filename, official_words)
  tsv_lines = get_tsv_words(tsv_filename)
  tsv_words = Set.new(tsv_lines)
  puts "\n #{tsv_filename}:"
  puts "  #{tsv_lines.size} lines, #{tsv_words.size} unique words"
  if official_words == tsv_words
    puts "  TSV word list matches new words for this level's official test list"
  else
    missing = official_words - tsv_words
    extra = tsv_words - official_words
    puts "  Missing words (not in TSV file):\n   #{missing.to_a.join("\n   ")}" if !missing.empty?
    puts "  Extra words (not in official test list):\n   #{extra.to_a.join("\n   ")}" if !extra.empty?
  end
end

puts "\nComparing TSV data entry with offical word lists..."
compare_tsv_to_official("hsk1.tsv", new_words[:hsk1])
compare_tsv_to_official("hsk2.tsv", new_words[:hsk2])
compare_tsv_to_official("hsk3.tsv", new_words[:hsk3])
compare_tsv_to_official("hsk4.tsv", new_words[:hsk4])
