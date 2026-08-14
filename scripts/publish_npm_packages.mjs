#!/usr/bin/env node
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const [tag, mode] = process.argv.slice(2);
if (!/^codex-gearbox-v\d+\.\d+\.\d+$/.test(tag ?? "")) {
  throw new Error("usage: publish_npm_packages.mjs codex-gearbox-vX.Y.Z [--dry-run]");
}

const version = tag.slice("codex-gearbox-v".length);
const root = fileURLToPath(new URL("..", import.meta.url));
const output = mkdtempSync(join(tmpdir(), "codex-gearbox-npm-"));
const assets = join(output, "assets");
mkdirSync(assets);

const platforms = [
  { name: "darwin-arm64", os: "darwin", cpu: "arm64", target: "aarch64-apple-darwin" },
  { name: "darwin-x64", os: "darwin", cpu: "x64", target: "x86_64-apple-darwin" },
  { name: "linux-x64", os: "linux", cpu: "x64", libc: "glibc", target: "x86_64-unknown-linux-gnu" },
  { name: "win32-x64", os: "win32", cpu: "x64", target: "x86_64-pc-windows-msvc", extension: ".exe" }
];
const commands = ["codex-gearbox", "gearbox-shift", "shift"];

function exec(command, args) {
  execFileSync(command, args, { stdio: "inherit" });
}

function packageJson(name, extra = {}) {
  return {
    name,
    version,
    description: "Plan-aware Codex model and effort router",
    license: "MIT",
    repository: "https://github.com/k4-1/codex-gearbox.git",
    ...extra
  };
}

function publish(directory, name) {
  exec("npm", ["pack", "--dry-run", directory]);
  try {
    execFileSync("npm", ["view", `${name}@${version}`, "version"], { stdio: "ignore" });
    return;
  } catch {
    if (mode === "--dry-run") return;
    exec("npm", ["publish", directory, "--access", "public"]);
  }
}

try {
  exec("gh", ["release", "download", tag, "--dir", assets]);

  for (const platform of platforms) {
    const directory = join(output, platform.name, "bin");
    mkdirSync(directory, { recursive: true });
    for (const command of commands) {
      const file = `${command}-${platform.target}${platform.extension ?? ""}`;
      const source = join(assets, file);
      if (!existsSync(source)) throw new Error(`release asset is missing: ${file}`);
      cpSync(source, join(directory, `${command}${platform.extension ?? ""}`));
    }
    const packageName = `codex-gearbox-${platform.name}`;
    writeFileSync(join(output, platform.name, "package.json"), `${JSON.stringify(packageJson(packageName, {
      os: [platform.os], cpu: [platform.cpu], ...(platform.libc ? { libc: [platform.libc] } : {}), files: ["bin"]
    }), null, 2)}\n`);
    publish(join(output, platform.name), packageName);
  }

  const cli = join(output, "cli");
  cpSync(join(root, "npm", "cli"), cli, { recursive: true });
  const manifestPath = join(cli, "package.json");
  const manifest = JSON.parse(readFileSync(manifestPath));
  manifest.version = version;
  manifest.optionalDependencies = Object.fromEntries(platforms.map(({ name }) => [`codex-gearbox-${name}`, version]));
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  publish(cli, "codex-gearbox");
} finally {
  rmSync(output, { recursive: true, force: true });
}
