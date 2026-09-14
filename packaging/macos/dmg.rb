# frozen_string_literal: true

# native-packages supplies an owned copy, already signed when Apple credentials
# are configured. Keep it unchanged and package only this input.
require "fileutils"
require "tmpdir"

payload, output = ARGV
abort "usage: dmg.rb PAYLOAD OUTPUT.dmg" unless payload && output
abort "output already exists: #{output}" if File.exist?(output)
Dir.mktmpdir("fastpotify-dmg-") do |directory|
  FileUtils.cp_r(File.join(payload, "."), directory, preserve: true)
  app = File.join(directory, "Spotifast.app")
  legacy = File.join(directory, "Fastpotify.app")
  abort "expected only the signed Spotifast.app input" unless File.directory?(app) && !File.exist?(legacy)
  # Earlier updaters require a real Fastpotify.app at the image root. Keep an
  # unchanged, hidden copy for them; Finder presents Spotifast for new installs.
  abort "legacy update copy failed" unless system("ditto", app, legacy)
  abort "legacy update signature failed" unless system("codesign", "--verify", "--deep", "--strict", legacy)
  abort "cannot hide the compatibility bundle" unless system("chflags", "hidden", legacy)
  File.symlink("/Applications", File.join(directory, "Applications"))
  abort "DMG creation failed" unless system("hdiutil", "create", "-volname", "Spotifast",
    "-srcfolder", directory, "-format", "UDZO", output)
  abort "DMG verification failed" unless system("hdiutil", "verify", output)
end
