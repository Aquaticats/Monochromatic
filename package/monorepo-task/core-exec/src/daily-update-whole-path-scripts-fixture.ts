/**
 Contents of the stand-in programs and of the files of the whole-path repository.

 @module
 */

/**
 The fake `gh`: records its calls and answers like an empty tracker.
 */
export const FAKE_GH: string = `#!/usr/bin/env node
const { appendFileSync, readFileSync } = require('node:fs');
const stdin = process.stdin.isTTY ? '' : readFileSync(0, 'utf8');
const names = ['GIT_ASKPASS', 'GIT_TERMINAL_PROMPT', 'GH_PROMPT_DISABLED'];
const environment = Object.fromEntries(names.map((name) => [name, process.env[name] === undefined ? '(unset)' : process.env[name]]));
appendFileSync(process.env.FIXTURE_GH_LOG, JSON.stringify({ args: process.argv.slice(2), stdin, environment }) + '\\n');
const exitCode = Number(process.env.FIXTURE_GH_EXIT ?? '0');
if (exitCode !== 0) { process.stderr.write('error connecting to api.github.com\\n'); process.exit(exitCode); }
const verb = process.argv[3];
if (verb === 'list') process.stdout.write('[]');
if (verb === 'create') process.stdout.write('https://github.com/example/fixture/issues/1\\n');
`;

/**
 The fake `cargo-shadow`: its `update` moves zbus in the two crates' locks, as the real update did on 2026-11-07.
 */
export const FAKE_CARGO_SHADOW: string = `#!/usr/bin/env node
const { readFileSync, writeFileSync } = require('node:fs');
if (process.argv[2] === 'update') {
  for (const lock of ['package/a/broken-crate/Cargo.lock', 'package/c/red-crate/Cargo.lock']) {
    writeFileSync(lock, readFileSync(lock, 'utf8').replace('version = "5.19.0"', 'version = "6.2.1"'));
  }
}
`;

/**
 The fake `pnpm`: nothing to install in the fixture.
 */
export const FAKE_PNPM: string = `#!/usr/bin/env node
`;

/**
 Script that replaces one text with another in a file, standing in for an update that moves a lock.
 */
export const MOVE_SCRIPT: string = `import { readFileSync, writeFileSync } from 'node:fs';
const [file, from, to] = process.argv.slice(2);
writeFileSync(file, readFileSync(file, 'utf8').replace(from, to));
`;

/**
 Script that stands in for a crate's Clippy: it fails when the lock names zbus 6, or always when asked.
 */
export const CLIPPY_SCRIPT: string = `import { readFileSync } from 'node:fs';
const [code, always] = process.argv.slice(2);
const lock = readFileSync('Cargo.lock', 'utf8');
if (always === 'always' || lock.includes('version = "6.2.1"')) {
  console.log('warning: unused import');
  console.log('');
  console.log('error[' + code + ']: failed to resolve: could not find \`blocking\` in \`zbus\`');
  console.log(' --> src/session.rs:12:11');
  process.exit(101);
}
`;

/**
 A Cargo lock text with zbus at 5.19.0.
 */
export const CARGO_LOCK: string = '[[package]]\nname = "zbus"\nversion = "5.19.0"\n\n[[package]]\nname = "serde"\nversion = "1.0.200"\n';

/**
 Root `mise.toml`: the update's task names, each a script that moves a lock or does nothing.
 */
export const ROOT_MISE: string = `monorepo_root = true

[monorepo]
config_roots = ["package/*/*"]

[settings]
experimental = true
task.monorepo_depth = 3

[tasks."toolchain:update"]
run = "node steps/move.ts toolchain.lock.json nightly-1 nightly-2"
[tasks."toolchain:install"]
run = "node -e 0"
[tasks."toolchain:update:global"]
run = "node steps/move.ts $FIXTURE_GLOBAL_LOCK nightly-1 nightly-2"
[tasks."toolchain:install:global"]
run = "node -e 0"
[tasks."deps:update"]
run = "node -e 0"
[tasks."catalog:tighten"]
run = "node -e 0"
[tasks."sync:files"]
run = "node -e 0"
[tasks."check:files"]
run = "node -e 0"
`;
