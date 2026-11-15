cask "multi-link" do
  version "@VERSION@"
  sha256 "@SHA256@"

  url @URL@
  name "multi-link"
  desc "Terminal dashboard for Claude Code and Codex sessions"
  homepage "https://github.com/jar-narvz/multi-link"

  depends_on arch: :arm64
  depends_on macos: :sequoia

  app "multi-link.app"
  binary "#{appdir}/multi-link.app/Contents/MacOS/multi-link"

  uninstall quit: "dev.multi-link.menubar"

  zap trash: "~/Library/Preferences/dev.multi-link.menubar.plist"

  caveats <<~EOS
    multi-link is not notarized. After a fresh install, open the app once,
    then click Open Anyway in System Settings > Privacy & Security.
    Until then, multi-link in Terminal is stopped with no message.
    Upgrades through brew normally keep that approval.

    Before uninstalling, untick Open at login in the app and run:
      multi-link hook remove
  EOS
end
