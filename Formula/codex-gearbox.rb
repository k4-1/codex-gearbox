class CodexGearbox < Formula
  desc "Plan-aware Codex model and effort router"
  homepage "https://github.com/k4-1/codex-gearbox"
  version "0.8.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.8.0/codex-gearbox-aarch64-apple-darwin"
      sha256 "787fc7c7a34ebe5516b93dde1381d3421793fb3497562f05f6225b73f6007217"
    else
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.8.0/codex-gearbox-x86_64-apple-darwin"
      sha256 "8661a8cceb47d49685ba6f2e9c55bc258c37bdfef780e7dd095ef4708083fa50"
    end
  end

  on_linux do
    url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.8.0/codex-gearbox-x86_64-unknown-linux-gnu"
    sha256 "d85731975b8908696c1e5b212569f543aef016bfd67e6d3acc7f6476aa825558"
  end

  def install
    binary = Dir[buildpath / "codex-gearbox-*"].first
    bin.install binary => "codex-gearbox"
    bin.install_symlink "codex-gearbox" => "gearbox-shift"
    bin.install_symlink "codex-gearbox" => "shift"
  end
end
