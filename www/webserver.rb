#!/usr/bin/ruby
require 'webrick'

WASM_SRC = "../target/wasm32-unknown-unknown/debug/ime_engine.wasm"
WASM_DEST = "ime-engine.wasm" # Note: '_' becomes '-'

# Cargo does not support post-build copy yet, so...
print "Copy #{WASM_DEST} from build dir? [y/N]: "
if ["Y", "y"].include? gets.chomp
  abort "Cannot find #{WASM_SRC}; try `cargo build`" if !File.exist? WASM_SRC
  `cp #{WASM_SRC} #{WASM_DEST}`
end
abort "#{WASM_DEST} is missing; try `cargo build`" if !File.exist? WASM_DEST

# Serve the current working directory on http://localhost:8000
# Use correct mime type for .wasm files (ruby only added wasm type in v2.7)
config = {DocumentRoot: Dir.pwd,
          MimeTypes: WEBrick::HTTPUtils::DefaultMimeTypes.merge({"wasm" => "application/wasm"}),
          DoNotReverseLookup: true,
          Port: 8000}
server = WEBrick::HTTPServer.new(config)
trap 'INT' do server.shutdown end
server.mount('/', WEBrick::HTTPServlet::FileHandler, config[:DocumentRoot], {})
server.start
