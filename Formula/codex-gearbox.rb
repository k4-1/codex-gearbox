class CodexGearbox < Formula
  desc "Plan-aware Codex model and effort router"
  homepage "https://github.com/k4-1/codex-gearbox"
  version "0.5.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.5.1/codex-gearbox-aarch64-apple-darwin"
      sha256 "e4c7a3f279f662510ee60e97dbe26b7a18a3ff80818f2ff97835aaf0abaf0514"
    else
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.5.1/codex-gearbox-x86_64-apple-darwin"
      sha256 "6bc0062ca44152099f1a3d5832290368d2526c4b198e4b1df82517c96f507abb"
    end
  end

  on_linux do
    url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.5.1/codex-gearbox-x86_64-unknown-linux-gnu"
    sha256 "a00a8ac6bb3a43df00f1a01ab6e28af6e21c081042ffe6ad2651d70c60a033ea"
  end

  def install
    binary = Dir[buildpath / "codex-gearbox-*"].first
    bin.install binary => "codex-gearbox"
    bin.install_symlink "codex-gearbox" => "gearbox-shift"
    bin.install_symlink "codex-gearbox" => "shift"
  end
end
