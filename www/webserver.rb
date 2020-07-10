#!/usr/bin/ruby
# Serve the current working directory on a local web server.
# Use correct mime type for .wasm files (ruby didn't include this until v2.7)
require 'webrick'

# Cargo doesn't include a way to do a post-build copy yet, so do this...
print "Overwrite debug.wasm with fresh build from ../target? [y/N]: "
if ["Y", "y"].include? gets.chomp
  if File.exist? "../target/wasm32-unknown-unknown/debug/wasm.wasm"
    `cp ../target/wasm32-unknown-unknown/debug/wasm.wasm debug.wasm`
  else
    abort "Can't the find the .wasm file in ../target. Try a `cargo build`."
  end
elsif !File.exist? "debug.wasm"
    abort "Debug.wasm is missing. Try a `cargo build`."
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
