#!/usr/bin/env python3
"""Write the Homebrew and Scoop manifests for a GitHub release."""

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOWNLOAD = "https://github.com/k4-1/codex-gearbox/releases/download"


def asset(assets, name):
    for item in assets:
        if item["name"] == name and item.get("digest", "").startswith("sha256:"):
            return item["digest"].removeprefix("sha256:")
    raise SystemExit(f"missing SHA-256 digest for {name}")


def url(tag, name):
    return f"{DOWNLOAD}/{tag}/{name}"


def main():
    release = json.load(open(sys.argv[1]))
    tag = release["tagName"]
    version = tag.removeprefix("codex-gearbox-v")
    if version == tag:
        raise SystemExit(f"unsupported release tag {tag}")
    assets = release["assets"]
    mac_arm = "codex-gearbox-aarch64-apple-darwin"
    mac_intel = "codex-gearbox-x86_64-apple-darwin"
    linux = "codex-gearbox-x86_64-unknown-linux-gnu"
    windows = "codex-gearbox-x86_64-pc-windows-msvc.exe"
    hashes = {name: asset(assets, name) for name in (mac_arm, mac_intel, linux, windows)}

    (ROOT / "Formula" / "codex-gearbox.rb").write_text(
        f'''class CodexGearbox < Formula
  desc "Plan-aware Codex model and effort router"
  homepage "https://github.com/k4-1/codex-gearbox"
  version "{version}"

  on_macos do
    if Hardware::CPU.arm?
      url "{url(tag, mac_arm)}"
      sha256 "{hashes[mac_arm]}"
    else
      url "{url(tag, mac_intel)}"
      sha256 "{hashes[mac_intel]}"
    end
  end

  on_linux do
    url "{url(tag, linux)}"
    sha256 "{hashes[linux]}"
  end

  def install
    binary = Dir[buildpath / "codex-gearbox-*"].first
    bin.install binary => "codex-gearbox"
    bin.install_symlink "codex-gearbox" => "gearbox-shift"
    bin.install_symlink "codex-gearbox" => "shift"
  end
end
'''
    )
    scoop = {
        "version": version,
        "description": "Plan-aware Codex model and effort router",
        "homepage": "https://github.com/k4-1/codex-gearbox",
        "license": "MIT",
        "architecture": {
            "64bit": {
                "url": url(tag, windows),
                "hash": hashes[windows],
            }
        },
        "bin": [
            [windows, "codex-gearbox"],
            [windows, "gearbox-shift"],
            [windows, "shift"],
        ],
    }
    (ROOT / "bucket" / "codex-gearbox.json").write_text(json.dumps(scoop, indent=2) + "\n")


if __name__ == "__main__":
    main()
