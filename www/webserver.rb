#!/usr/bin/ruby
# Serve the current working directory on a local web server.
# Use correct mime type for .wasm files (ruby didn't include this until v2.7)
require 'webrick'

# Cargo doesn't include a way to do a post-build copy yet, so do this...
print "Overwrite debug.wasm with a fresh copy from ../target? [y/N]: "
if ["Y", "y"].include? gets.chomp
  if File.exist? "../target/wasm32-unknown-unknown/debug/wasm.wasm"
    puts "getting a fresh copy of debug.wasm"
    `cp ../target/wasm32-unknown-unknown/debug/wasm.wasm debug.wasm`
  else
    abort "Can't the find the .wasm file in ../target. Try a `cargo build`."
  end
else
  if File.exist? "debug.wasm"
    puts "Using existing copy of debug.wasm"
  else
    abort "Debug.wasm is missing. Try a `cargo build`."
  end
end

more_mime_types = {"wasm" => "application/wasm"}
config = {
  :Port => 8000,
  :DocumentRoot => Dir.pwd,
  :DoNotReverseLookup => true,
  :MimeTypes => WEBrick::HTTPUtils::DefaultMimeTypes.merge(more_mime_types),
}
server = WEBrick::HTTPServer.new(config)
trap 'INT' do server.shutdown end

# Start a web server on http://localhost:8000/
server.mount('/', WEBrick::HTTPServlet::FileHandler, config[:DocumentRoot], {})
server.start
