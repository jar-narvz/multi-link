#!/bin/sh
set -eu
# Local audit URL: --url 'http://127.0.0.1:<port>/<name>-#{version}.dmg'
root=$(CDPATH='' cd -- "$(dirname "$0")/.." && pwd)
exec ruby - "$root/packaging/homebrew/multi-link.rb" "$@" <<'RUBY'
template = ARGV.shift
fail_input = lambda do |message|
  warn "render-cask: #{message}"
  exit 1
end
fail_input.call('usage: tools/render-cask.sh <version> <sha256> [--url <url>]') unless [2, 4].include?(ARGV.length)
version, sha, flag, url = ARGV
fail_input.call('invalid version') unless version.match?(/\A[0-9]+\.[0-9]+(\.[0-9]+)?\z/)
fail_input.call('sha256 must be exactly 64 lowercase hex characters') unless sha.match?(/\A[0-9a-f]{64}\z/)
if ARGV.length == 4
  fail_input.call('expected --url') unless flag == '--url'
  fail_input.call('URL must not contain double quotes, backslashes or whitespace') if url.match?(/["\\]|[[:space:]]/)
  fail_input.call('URL only permits #{version} interpolation') if url.gsub('#{version}', '').include?('#{')
  fail_input.call('URL must start with https:// or http://127.0.0.1') unless url.match?(%r{\A(?:https://|http://127\.0\.0\.1(?=[:/]|\z))})
end
# Preserve only the allowed version interpolation; dump escapes other Ruby syntax.
url_literal = url ? url.dump.gsub('\#{version}', '#{version}') : '"https://github.com/jar-narvz/multi-link/releases/download/v#{version}/multi-link-#{version}-arm64.dmg"'
values = { '@VERSION@' => version, '@SHA256@' => sha, '@URL@' => url_literal }
print File.read(template).gsub(/@VERSION@|@SHA256@|@URL@/) { |token| values.fetch(token) }
RUBY
