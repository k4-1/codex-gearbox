class CodexGearbox < Formula
  desc "Plan-aware Codex model and effort router"
  homepage "https://github.com/k4-1/codex-gearbox"
  version "0.6.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.6.1/codex-gearbox-aarch64-apple-darwin"
      sha256 "d1102445c29c47eee6005b9928a5925750163ebaac666282933325955fa7aa5d"
    else
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.6.1/codex-gearbox-x86_64-apple-darwin"
      sha256 "e078f2edd44ea029374d4c88a1d017bd8b40a2cfb3065725ba07032f449cf7de"
    end
  end

  on_linux do
    url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.6.1/codex-gearbox-x86_64-unknown-linux-gnu"
    sha256 "8ca3b2918c804cd4e73240d58eca053974651dbd5d2485da79297b8c8cb990a5"
  end

  def install
    binary = Dir[buildpath / "codex-gearbox-*"].first
    bin.install binary => "codex-gearbox"
    bin.install_symlink "codex-gearbox" => "gearbox-shift"
    bin.install_symlink "codex-gearbox" => "shift"
  end
end
