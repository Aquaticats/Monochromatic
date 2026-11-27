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
 The fake `cargo-shadow`: its `update` moves zbus in the two crates' locks, as the real update did on 2026-11-07, and its
 `install` reconciles the slint version each lock holds with the floor its manifest names.

 The refusal text of the frozen form is copied from the producer, not invented: the `frozen install:` line is the
 `eprintln!` in `package/cargo-shadow/cli/src/install.rs`, and the rest is the `Display` of `InstallError::Stale` in
 `package/cargo-shadow/cli/src/install_error.rs` (`Frozen install failed:`, one `- <root>` line per stale root, then the two
 closing lines). `FIXTURE_CARGO_SHADOW_SKIPS_LOCK_WRITE=1` makes the plain install report success and leave
 `package/a/broken-crate`'s lock stale, which is the stand-in for an install that does not do what it claims: the fixture
 stands in for a fault, it does not reproduce a defect of the real `cargo-shadow`.
 */
export const FAKE_CARGO_SHADOW: string = `#!/usr/bin/env node
const { readFileSync, writeFileSync } = require('node:fs');

const SKIPPED_ROOT = 'package/a/broken-crate';
const CRATES = ['package/a/broken-crate', 'package/c/red-crate'];

const ceilingOf = (manifest) => /slint = \\{ version = ">=([^"]+)"/.exec(manifest)?.[1];
const lockedSlintOf = (lock) => /name = "slint"\\nversion = "([^"]+)"/.exec(lock)?.[1];
const parts = (version) => version.split('.').map(Number);
const satisfies = (locked, floor) => {
  const [lockMajor, lockMinor, lockPatch] = parts(locked);
  const [floorMajor, floorMinor, floorPatch] = parts(floor);
  if (lockMajor !== floorMajor) return lockMajor > floorMajor;
  if (lockMinor !== floorMinor) return lockMinor > floorMinor;
  return lockPatch >= floorPatch;
};

const verb = process.argv[2];
if (verb === 'update') {
  for (const crate of CRATES) {
    const lock = crate + '/Cargo.lock';
    writeFileSync(lock, readFileSync(lock, 'utf8').replace('version = "5.19.0"', 'version = "6.2.1"'));
  }
}
if (verb === 'install') {
  const frozen = process.argv.includes('--frozen-lockfile');
  const stale = [];
  for (const crate of CRATES) {
    const floor = ceilingOf(readFileSync(crate + '/Cargo.toml', 'utf8'));
    const lock = readFileSync(crate + '/Cargo.lock', 'utf8');
    const locked = lockedSlintOf(lock);
    if (frozen) {
      if (!satisfies(locked, floor)) stale.push(crate);
      continue;
    }
    if (process.env.FIXTURE_CARGO_SHADOW_SKIPS_LOCK_WRITE === '1' && crate === SKIPPED_ROOT) continue;
    writeFileSync(crate + '/Cargo.lock', lock.replace('name = "slint"\\nversion = "' + locked, 'name = "slint"\\nversion = "' + floor));
  }
  if (frozen && stale.length > 0) {
    process.stderr.write('cargo-shadow: frozen install: ' + stale.length + ' stale lockfiles, nothing written\\n');
    process.stderr.write('Frozen install failed: ' + stale.length + ' Cargo.lock file(s) are present but need an update to satisfy their manifests.\\n');
    for (const crate of stale) process.stderr.write('- ' + crate + '\\n');
    process.stderr.write('Nothing was written. Frozen is the default in CI, as in pnpm; \`--locked\` or \`--frozen\` on the cargo command asks for it too.\\n');
    process.stderr.write('To update them, run \`cargo check\` (or \`cargo-shadow install --no-frozen-lockfile\`) outside CI and commit the Cargo.lock changes.\\n');
    process.exit(1);
  }
}
`;

/**
 The Rust catalog of the fixture repository: one default floor and the exact pin the Slint testing backend carries.
 */
export const RUST_CATALOG: string = '[catalog]\nslint = ">=1.18.0"\n\n[catalogs.pinned]\ni-slint-backend-testing = "=1.18.0"\n';

/**
 One crate manifest: the generated catalog table `sync:files` writes and the named-catalog selection beside it.

 @param name - the crate's package name

 @returns the manifest text

 @example
 ```ts
 crateManifest({ name: 'broken-crate', },);
 ```
 */
export function crateManifest({
  name,
}: {
  readonly name: string;
},): string {
  return `[package]\nname = "${name}"\nversion = "0.0.0"\n\n[workspace.dependencies]\nslint = { version = ">=1.18.0", default-features = false }\n\n[package.metadata.rust-catalog.catalogs]\ni-slint-backend-testing = "pinned"\n`;
}

/**
 The workspace catalog the fixture's fake `catalog:tighten` raises.
 */
export const PNPM_WORKSPACE: string = 'catalog:\n  oxlint: \'>=1.93\'\n';

/**
 The lock text that records the catalog value before the raise, and the version the fake `deps:update` moves.
 */
export const PNPM_LOCK: string =
  'lockfileVersion: \'9.0\'\n\ncatalogs:\n  default:\n    oxlint:\n      specifier: \'>=1.93\'\n      version: 1.93.4\n';

/**
 The fake `pnpm`: it records the catalog the way pnpm 12 does, and refuses the pair that does not match the way its
 `--frozen-lockfile` does.

 `FIXTURE_PNPM_SKIPS_LOCKFILE_WRITE=1` makes the plain install leave the lock as it was, which is the state a run that
 stopped before the tail leaves behind; the frozen install must then fail, exactly as `ERR_PNPM_LOCKFILE_CONFIG_MISMATCH`
 refuses on 2026-11-23 (`doc/troubleshooting/pnpm-frozen-install-validation-drift.md`).
 */
export const FAKE_PNPM: string = `#!/usr/bin/env node
const { existsSync, readFileSync, writeFileSync } = require('node:fs');
const workspace = 'pnpm-workspace.yaml';
const lock = 'pnpm-lock.yaml';
if (!existsSync(workspace) || !existsSync(lock)) { process.exit(0); }
const wanted = /^\\s+[^:]+: '(>=.+?)'$/m.exec(readFileSync(workspace, 'utf8'))?.[1];
const recorded = /specifier: '(>=.+?)'/.exec(readFileSync(lock, 'utf8'))?.[1];
if (process.argv.includes('--frozen-lockfile')) {
  if (wanted !== recorded) {
    process.stderr.write('Error: ERR_PNPM_LOCKFILE_CONFIG_MISMATCH\\n\\n  x installing dependencies\\n  ' +
      'The current "catalogs" configuration does not match the value found in the lockfile: ' +
      workspace + ' has "' + wanted + '", ' + lock + ' has "' + recorded + '"\\n');
    process.exit(1);
  }
  process.exit(0);
}
if (process.env.FIXTURE_PNPM_SKIPS_LOCKFILE_WRITE === '1') { process.exit(0); }
writeFileSync(lock, readFileSync(lock, 'utf8').replace("specifier: '" + recorded, "specifier: '" + wanted));
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
 A Cargo lock text with zbus at 5.19.0 and slint at the floor the manifest names before any raise.
 */
export const CARGO_LOCK: string =
  '[[package]]\nname = "zbus"\nversion = "5.19.0"\n\n[[package]]\nname = "serde"\nversion = "1.0.200"\n\n[[package]]\nname = "slint"\nversion = "1.18.0"\n';

/**
 The fake `catalog:tighten` for the Rust catalog: it raises the slint floor to the highest release a consuming crate's lock
 holds, the way `package/dependency-catalog/cli-tighten` raises it (`cargo/run.ts`). It never touches a named-catalog row,
 because that tool skips them by design.
 */
export const TIGHTEN_CARGO_SCRIPT: string = `import { readFileSync, writeFileSync } from 'node:fs';
const catalog = readFileSync('rust-catalog.toml', 'utf8');
writeFileSync('rust-catalog.toml', catalog.replace('slint = ">=1.18.0"', 'slint = ">=1.20.0"'));
`;

/**
 The fake `file-enforcer write` (`mise run sync:files`): renders the Rust catalog's slint floor into every crate manifest,
 which is the only one this fixture's catalog holds.

 It reads and writes the same lines file-enforcer's `rust-catalog` rule owns, and nothing else.
 */
export const SYNC_SCRIPT: string = `import { readFileSync, writeFileSync } from 'node:fs';
const CRATES = ['package/a/broken-crate', 'package/c/red-crate'];
const floor = /slint = ">=([^"]+)"/.exec(readFileSync('rust-catalog.toml', 'utf8'))?.[1];
for (const crate of CRATES) {
  const manifest = crate + '/Cargo.toml';
  writeFileSync(manifest, readFileSync(manifest, 'utf8').replace(/slint = \\{ version = ">=[^"]+"/, 'slint = { version = ">=' + floor + '"'));
}
`;

/**
 The fake `file-enforcer check` (`mise run check:files`): reports every manifest whose rendered floor differs from the
 catalog's, in the shape the real check prints.

 The line shape is the one the real tool printed on `main` for issue 6398 (`drift-content | <path> | ... differs from what
 rule rust-catalog generates (content)`), quoted in GitHub issue 6398.
 */
export const CHECK_SCRIPT: string = `import { readFileSync } from 'node:fs';
const CRATES = ['package/a/broken-crate', 'package/c/red-crate'];
const floor = /slint = ">=([^"]+)"/.exec(readFileSync('rust-catalog.toml', 'utf8'))?.[1];
const drifted = [];
for (const crate of CRATES) {
  const rendered = /slint = \\{ version = ">=([^"]+)"/.exec(readFileSync(crate + '/Cargo.toml', 'utf8'))?.[1];
  if (rendered !== floor) drifted.push('drift-content | ' + crate + '/Cargo.toml | ... differs from what rule rust-catalog generates (content)');
}
if (drifted.length > 0) {
  for (const line of drifted) process.stdout.write(line + '\\n');
  process.exit(1);
}
`;

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
run = "node steps/move.ts pnpm-lock.yaml 1.93.4 1.94.0"
[tasks."catalog:tighten"]
run = ['node steps/move.ts pnpm-workspace.yaml ">=1.93" ">=1.94"', 'node steps/tighten-cargo.ts']
[tasks."sync:files"]
run = "node steps/sync.ts"
[tasks."check:files"]
run = "node steps/check.ts"
`;
