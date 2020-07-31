#!/usr/bin/ruby
require 'webrick'

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
