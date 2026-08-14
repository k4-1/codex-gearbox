class CodexGearbox < Formula
  desc "Plan-aware Codex model and effort router"
  homepage "https://github.com/k4-1/codex-gearbox"
  version "0.5.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.5.0/codex-gearbox-aarch64-apple-darwin"
      sha256 "674af0c6cafbbc8a38723f05e96d739e3b1026d1e1353ba8f39ffde0d8d38c77"
    else
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.5.0/codex-gearbox-x86_64-apple-darwin"
      sha256 "0bcfb760f9bd21681f3ac8533db2904fd170d132ee915a8098dfd272dc26ff28"
    end
  end

  on_linux do
    url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.5.0/codex-gearbox-x86_64-unknown-linux-gnu"
    sha256 "8d6b4e8f97df6658ffaf8311223fb9ca79aa79d5ce545a22365749a1aae8c91f"
  end

  def install
    binary = Dir[buildpath / "codex-gearbox-*"].first
    bin.install binary => "codex-gearbox"
    bin.install_symlink "codex-gearbox" => "gearbox-shift"
    bin.install_symlink "codex-gearbox" => "shift"
  end
end
