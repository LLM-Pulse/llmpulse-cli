#!/usr/bin/env node
// Builds the npm packages for one CLI release from its GitHub release archives:
//   node npm/build.mjs <version> <archives dir> <output dir>
// Each platform binary is its own package, selected by os/cpu through the
// launcher's optionalDependencies, so installs need no postinstall script
// (pnpm and bun skip those by default).
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const [version, archives, output] = process.argv.slice(2);
if (!version || !archives || !output) {
  console.error("usage: node npm/build.mjs <version> <archives dir> <output dir>");
  process.exit(1);
}

const repository = { type: "git", url: "git+https://github.com/LLM-Pulse/llmpulse-cli.git" };
const common = {
  version,
  license: "MIT",
  author: "LLM Pulse <info@llmpulse.ai>",
  homepage: "https://github.com/LLM-Pulse/llmpulse-cli#readme",
  bugs: { url: "https://github.com/LLM-Pulse/llmpulse-cli/issues" },
  repository,
  publishConfig: { access: "public" },
};
const platforms = [
  { suffix: "darwin-arm64", os: "darwin", cpu: "arm64", asset: "llmpulse-macos-aarch64", binary: "llmpulse" },
  { suffix: "darwin-x64", os: "darwin", cpu: "x64", asset: "llmpulse-macos-x86_64", binary: "llmpulse" },
  { suffix: "linux-x64", os: "linux", cpu: "x64", libc: "glibc", asset: "llmpulse-linux-x86_64", binary: "llmpulse" },
  { suffix: "win32-x64", os: "win32", cpu: "x64", asset: "llmpulse-windows-x86_64", binary: "llmpulse.exe" },
];

const write = (file, contents) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, contents);
};
const json = (value) => `${JSON.stringify(value, null, 2)}\n`;
const license = path.join(here, "..", "LICENSE");

fs.rmSync(output, { recursive: true, force: true });
const scratch = fs.mkdtempSync(path.join(output + "-extract-"));

for (const platform of platforms) {
  const name = `@llmpulse/cli-${platform.suffix}`;
  const dir = path.join(output, `cli-${platform.suffix}`);
  const zip = platform.os === "win32";
  const archive = path.join(archives, `${platform.asset}.${zip ? "zip" : "tar.gz"}`);
  const into = path.join(scratch, platform.suffix);
  fs.mkdirSync(into, { recursive: true });
  execFileSync(zip ? "unzip" : "tar", zip ? ["-q", archive, "-d", into] : ["-xzf", archive, "-C", into]);
  const binary = path.join(dir, "bin", platform.binary);
  fs.mkdirSync(path.dirname(binary), { recursive: true });
  fs.copyFileSync(path.join(into, platform.asset, platform.binary), binary);
  fs.chmodSync(binary, 0o755);
  fs.copyFileSync(license, path.join(dir, "LICENSE"));
  write(path.join(dir, "README.md"), `# ${name}\n\nThe ${platform.os} ${platform.cpu} binary of [@llmpulse/cli](https://www.npmjs.com/package/@llmpulse/cli). Install that package instead of this one.\n`);
  write(path.join(dir, "package.json"), json({
    name,
    description: `The ${platform.os} ${platform.cpu} binary of the LLM Pulse CLI.`,
    ...common,
    os: [platform.os],
    cpu: [platform.cpu],
    ...(platform.libc ? { libc: [platform.libc] } : {}),
    files: ["bin", "LICENSE", "README.md"],
    preferUnplugged: true,
  }));
}

const launcher = path.join(output, "cli");
fs.mkdirSync(path.join(launcher, "bin"), { recursive: true });
fs.copyFileSync(path.join(here, "launcher.js"), path.join(launcher, "bin", "llmpulse.js"));
fs.chmodSync(path.join(launcher, "bin", "llmpulse.js"), 0o755);
fs.copyFileSync(path.join(here, "README.md"), path.join(launcher, "README.md"));
fs.copyFileSync(license, path.join(launcher, "LICENSE"));
write(path.join(launcher, "package.json"), json({
  name: "@llmpulse/cli",
  description: "Official CLI for the LLM Pulse API: AI visibility metrics, mentions, citations, sentiment and share of voice from your terminal.",
  ...common,
  keywords: ["llmpulse", "llm-pulse", "cli", "ai-visibility", "geo", "seo", "chatgpt", "perplexity"],
  bin: { llmpulse: "bin/llmpulse.js" },
  files: ["bin", "LICENSE", "README.md"],
  engines: { node: ">=18" },
  optionalDependencies: Object.fromEntries(platforms.map((p) => [`@llmpulse/cli-${p.suffix}`, version])),
}));

fs.rmSync(scratch, { recursive: true, force: true });
console.log([...platforms.map((p) => `cli-${p.suffix}`), "cli"].map((dir) => path.join(output, dir)).join("\n"));
