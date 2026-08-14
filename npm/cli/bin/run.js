"use strict";

const { spawnSync } = require("node:child_process");

const packages = {
  "darwin-arm64": "codex-gearbox-darwin-arm64",
  "darwin-x64": "codex-gearbox-darwin-x64",
  "linux-x64": "codex-gearbox-linux-x64",
  "win32-x64": "codex-gearbox-win32-x64"
};

module.exports = function run(command) {
  const packageName = packages[`${process.platform}-${process.arch}`];
  if (!packageName) {
    console.error(`codex-gearbox does not support ${process.platform}-${process.arch}`);
    process.exit(1);
  }

  let binary;
  try {
    binary = require.resolve(`${packageName}/bin/${command}${process.platform === "win32" ? ".exe" : ""}`);
  } catch {
    console.error(`Missing ${packageName}; reinstall codex-gearbox.`);
    process.exit(1);
  }

  const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
  if (result.error) throw result.error;
  process.exit(result.status ?? 1);
};
