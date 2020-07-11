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
  return words
end

def get_tsv_words(filename)
  words = []
  for row in File.read(filename).lines
    hanzi, _pinyin = row.chomp.split("\t")
    words << hanzi
  end
  return words
end

puts "Checking archive of offical word lists..."
counts = {hsk1: 150, hsk2: 300, hsk3: 600, hsk4: 1200, hsk5: 2500, hsk6: 5000}
for key in counts.keys.sort
  words = get_official_words(key.to_s)
  puts " #{key}: #{words.size} #{if words.size == counts[key] then "ok" else "err" end}"
end

puts "Comparing TSV data entry with offical word lists..."
checks = { hsk1: "hsk1.tsv" }
for archive_key, tsv_filename in checks.each_pair
  tsv_lines = get_tsv_words(tsv_filename)
  tsv_words = Set.new(tsv_lines)
  archive_words = Set.new(get_official_words(archive_key.to_s))
  puts " #{tsv_filename}:"
  puts "  #{tsv_lines.size} lines, #{tsv_words.size} unique words"
  if archive_words == tsv_words
    puts "  TSV word list matches archive list"
  else
    missing = archive_words - tsv_words
    extra = tsv_words - archive_words
    puts "  Missing words (not in TSV file):\n   #{missing.to_a.join("\n   ")}" if !missing.empty?
    puts "  Extra words (not in official list):\n   #{extra.to_a.join("\n   ")}" if !extra.empty?
  end
end
