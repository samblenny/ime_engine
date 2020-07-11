#!/usr/bin/ruby
require "zlib"

ARCHIVE = "hsk2012.gz"

if !File.exist? ARCHIVE
  abort "Error: The archive file is missing: #{ARCHIVE}"
end
buf = Zlib::GzipReader.open(ARCHIVE) { |gz| gz.read }
keys = buf.lines
         .select { |line| line.start_with? "--- " }
         .map { |line| line.gsub(/--- (.*)/, '\1').chomp }
puts "Records in #{ARCHIVE} ...\n #{keys.join("\n ")}"


