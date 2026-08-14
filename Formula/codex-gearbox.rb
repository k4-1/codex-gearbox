class CodexGearbox < Formula
  desc "Plan-aware Codex model and effort router"
  homepage "https://github.com/k4-1/codex-gearbox"
  version "0.6.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.6.0/codex-gearbox-aarch64-apple-darwin"
      sha256 "d376e2b2e018f14119f9f39b4df50a6c81379fdb2815f0e02378dcb441060cfd"
    else
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.6.0/codex-gearbox-x86_64-apple-darwin"
      sha256 "24d21238904685a7982a90caac820ec5a715b3b6308c8bc3ee38e6b73e6b845d"
    end
  end

  on_linux do
    url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.6.0/codex-gearbox-x86_64-unknown-linux-gnu"
    sha256 "3fbbf544033dd1dc0c9c0cf02a79774a48c8ad31b88e3b70e5860a93adf8f9bb"
  end

  def install
    binary = Dir[buildpath / "codex-gearbox-*"].first
    bin.install binary => "codex-gearbox"
    bin.install_symlink "codex-gearbox" => "gearbox-shift"
    bin.install_symlink "codex-gearbox" => "shift"
  end
end
