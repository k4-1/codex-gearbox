class CodexGearbox < Formula
  desc "Plan-aware Codex model and effort router"
  homepage "https://github.com/k4-1/codex-gearbox"
  version "0.4.2"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.4.2/codex-gearbox-aarch64-apple-darwin"
      sha256 "0c3ae45f5fbeff5900e5d4a5ac539468a1a4fcca05573ec3ef24041fda44687a"
    else
      url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.4.2/codex-gearbox-x86_64-apple-darwin"
      sha256 "8dcccf754b9dae23c2a6a628d6c0eb099e61dc01514dbfcc5af51ba10d7d386c"
    end
  end

  on_linux do
    url "https://github.com/k4-1/codex-gearbox/releases/download/codex-gearbox-v0.4.2/codex-gearbox-x86_64-unknown-linux-gnu"
    sha256 "dff1d18f6e79d6e08b26e518e7c26ad0ce8532b175a9b5da519c5ca9e9161c4c"
  end

  def install
    binary = Dir[buildpath / "codex-gearbox-*"].first
    bin.install binary => "codex-gearbox"
    bin.install_symlink "codex-gearbox" => "gearbox-shift"
    bin.install_symlink "codex-gearbox" => "shift"
  end
end
