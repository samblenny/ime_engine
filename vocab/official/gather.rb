#!/usr/bin/ruby
require "zlib"

IN_FILES = Dir.entries(".").sort.select { |f| /hsk.*txt/.match(f) }
OUT_ORIG = "hsk2012.txt"
OUT_FILE = "hsk2012.gz"

abort "Error: Where are the input files?" if IN_FILES.empty?
puts "Preparing to compress files as #{OUT_FILE}:\n  #{IN_FILES.join("\n  ")}"
print "Do you want to proceed? [y/N]: "
abort "ok, maybe another time..." if !["y", "Y"].include? gets.chomp

buf = ""
for f in IN_FILES
  buf << "--- #{f}\n"
  buf << IO.binread(f)
end
Zlib::GzipWriter.open(OUT_FILE) { |gz|
  gz.mtime = Time.now
  gz.orig_name = OUT_ORIG
  gz.write buf
}
puts "done"
