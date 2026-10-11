#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const { spawnSync } = require("node:child_process");

const PLATFORM_PACKAGES = {
  "darwin arm64": "@llmpulse/cli-darwin-arm64",
  "darwin x64": "@llmpulse/cli-darwin-x64",
  "linux x64": "@llmpulse/cli-linux-x64",
  "win32 x64": "@llmpulse/cli-win32-x64",
};
const RELEASES = "https://github.com/LLM-Pulse/llmpulse-cli/releases";

const platformPackage = PLATFORM_PACKAGES[`${process.platform} ${process.arch}`];
if (!platformPackage) {
  console.error(`llmpulse: no prebuilt binary for ${process.platform} ${process.arch}. Build from source: ${RELEASES}`);
  process.exit(1);
}

const executable = process.platform === "win32" ? "llmpulse.exe" : "llmpulse";
let binary;
try {
  binary = require.resolve(`${platformPackage}/bin/${executable}`);
} catch {
  console.error(`llmpulse: ${platformPackage} is not installed. Reinstall @llmpulse/cli without --omit=optional or --no-optional.`);
  process.exit(1);
}

if (process.platform !== "win32") {
  try {
    fs.accessSync(binary, fs.constants.X_OK);
  } catch {
    fs.chmodSync(binary, 0o755);
  }
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
if (result.error) {
  console.error(`llmpulse: ${result.error.message}`);
  process.exit(1);
}
if (result.signal) {
  process.kill(process.pid, result.signal);
}
process.exit(result.status ?? 1);
