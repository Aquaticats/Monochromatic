# Targeted evidence: onchange 7.1.0

Status:
 agent-written during the targeted-evidence run of 2026-11-25;
 read-only against the candidate (nothing was installed, built, tested or run);
 incomplete,
 not approved by the owner,
 not final.

This file records evidence about one candidate for the watcher vet whose report is
`doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md`.
The design it serves is `doc/planning/garage-bisync-run-on-file-change.md`.
It records no score, no ranking and no recommendation,
and it does not edit the vet report.

## Purpose and scope

The vet report lists seven hard constraints and asks each surviving candidate for
source-level confirmation of them, plus the capabilities it records for every candidate,
plus a maintenance audit.
This file answers that for onchange 7.1.0 with primary evidence:
the npm registry metadata and published tarball,
the released tag's own source,
the source of each dependency whose behaviour a claim rests on
(Chokidar for watching, cross-spawn for spawning, tree-kill for killing, fsevents for macOS),
the npm and GitHub release history,
and the repository's issue and pull-request history.

Every claim below is either read from a file (cited as `path:line` plus an excerpt),
read from a registry or GitHub API response (cited with the exact command and the fields used),
or derived by arithmetic from those reads.
No behaviour in this file was measured at runtime,
and no code of the candidate or of its dependencies was executed.
The vet report's "External execution gate" and "Finalist validation" sections remain open,
so nothing here can stand in for them.

## Candidate identity and access

Candidate: onchange 7.1.0, the npm package `onchange`
(`https://www.npmjs.com/package/onchange`).
Repository: `Qard/onchange` (`https://github.com/Qard/onchange`),
found through the registry metadata's `repository` field,
which for 7.1.0 reads
`"repository":{"type":"git","url":"git+https://github.com/Qard/onchange.git"}`
(`curl --silent https://registry.npmjs.org/onchange`, field `versions.7.1.0.repository`).
Release tag: `v7.1.0`.
Access date for every URL and API query below: 2026-11-25,
in a session that ran from 18:23 to 18:40 -05:00.

The clone was made exactly as instructed:

```console
$ gh repo clone Qard/onchange "/home/user/temp/agent/onchange-source-for-garage-watcher-vet-2026-11-25" -- --depth 1 --branch v7.1.0
```

Clone path: `/home/user/temp/agent/onchange-source-for-garage-watcher-vet-2026-11-25`.
It is a shallow clone and is deleted when this run ends,
so every `path:line` below is relative to that clone root and can be re-created with the command above.

What the tag points at (`git rev-parse`, `git cat-file -t`, `git cat-file -p`):

- `v7.1.0` is an annotated tag object, `git cat-file -t v7.1.0` reports `tag`,
  `git cat-file -p v7.1.0` reads `object 4337dee2f71b37cdb669e20f2cbb80a6b181cd67`,
  `type commit`, `tag v7.1.0`,
  `tagger Blake Embrey <hello@blakeembrey.com> 1603328032 -0700`, message `7.1.0`.
- `git rev-parse v7.1.0^{commit}` and `git rev-parse HEAD` both give
  `4337dee2f71b37cdb669e20f2cbb80a6b181cd67`.
- That commit is dated `2020-10-21T17:53:51-07:00`,
  author `Blake Embrey <hello@blakeembrey.com>`, subject `7.1.0`,
  tree `cf19ecd0faf03b9fd424c36a5b9a8e88ee29c083`.
- The registry's `gitHead` for 7.1.0 is the same
  `4337dee2f71b37cdb669e20f2cbb80a6b181cd67`,
  and `time["7.1.0"]` is `2020-10-22T00:54:16.861Z`.
- `gh api repos/Qard/onchange/releases/tags/v7.1.0` reports
  `"published_at":"2020-10-22T00:55:29Z"` and `"assets":[]`.

Working tree at the tag: 13 tracked files (`git ls-files`):
`.gitignore`, `.travis.yml`, `LICENSE`, `README.md`, `package-lock.json`, `package.json`,
`src/bin.ts` (155 lines), `src/echo.ts` (3 lines), `src/index.ts` (295 lines),
`test/concurrent.js`, `test/queue.js`, `test/scripts/slow.js`, `tsconfig.json`.
There is no `.github/` directory at the tag,
so no release workflow, no CI workflow and no issue templates.

Version and licence fields read at the tag:
`package.json:2` `"version": "7.1.0"`,
`package.json:14` `"license": "MIT"`,
`package.json:11-13` `"bin": { "onchange": "dist/bin.js" }`,
`package.json:17-19` `"files": [ "dist/" ]`.
There is no `engines` field anywhere in `package.json`,
and no `os` or `cpu` field.

### Dependency sources used for backend and execution claims

Onchange delegates all watching to Chokidar, all spawning to cross-spawn,
all killing to tree-kill and all templating to `@blakeembrey/template`.
Those behaviours are therefore claimed from those packages' own source,
at the version a fresh install of 7.1.0 resolves today under the declared ranges:

- `chokidar` `^3.3.1` (`package.json:52`) resolves to 3.6.0,
  published 2024-02-06, sha1 `197c6cc669ef2a8dc5e7b4d97ee4e092c3eb0d5b`,
  verified by `sha1sum` of `https://registry.npmjs.org/chokidar/-/chokidar-3.6.0.tgz`
  (this run extracted the verified tarball; every `chokidar` citation below is to
  `dep-chokidar-3.6.0/package/...` in the scratch folder of this run).
  The repository's own `package-lock.json:227-230` pins 3.3.1
  (`"chokidar": { "version": "3.3.1" ... "integrity": "sha512-4QYCEWOcK3OJrxwvyyAOx...`),
  so the same claims were also checked against 3.3.1
  (sha1 `c84e5b3d18d9a4d77558fef466b1bf16bbeb3450`, also verified by `sha1sum`),
  and its line numbers are given beside 3.6.0's where they differ.
- `cross-spawn` `^7.0.1` (`package.json:53`) resolves to 7.0.6
  (sha1 `8a58fe78f00dcd70c370451759dfbfaf03e8ee9f`).
- `tree-kill` `^1.2.2` (`package.json:55`) resolves to 1.2.2,
  the latest published version, sha1 `4ca09a9092c88b73a7cdc5e8a01b507b0790a0cc`, verified.
- `@blakeembrey/template` `^1.0.0` (`package.json:50`) resolves to 1.2.0
  (sha1 `acd948eb29334e882019e8876b42ff6da1dcf528`, verified).
- `@blakeembrey/deque` `^1.0.5` (`package.json:49`) resolves to 1.0.5.
- `arg` `^4.1.3` (`package.json:51`) resolves to 4.1.3, the newest 4.x.
- `ignore` `^5.1.4` (`package.json:54`) resolves to 5.3.2, the newest 5.x.
- On macOS only, Chokidar's optional `fsevents` `~2.3.2` resolves to 2.3.3
  (sha1 `cac6407785d03675a2a5e1a5305c697b347d90d6`, verified).

`npm install` was not run, so the resolved versions above are read from the registry's
version lists and semver ranges, not from an installed tree.

## How to read a record

Each record gives the fields the governing skill's "Evidence records" section asks for:
candidate and exact version,
the claim and why it decides a gate,
the gate or overlay it bears on,
a status,
the primary evidence with an excerpt and access date,
corroboration or counterevidence where material,
and an outcome.
Records `OC-E01` to `OC-E23` decide hard constraints or the capabilities the vet report
records for every candidate; records `OC-M01` to `OC-M05` are the maintenance audit.
The `OC-` prefix exists only to keep record names unique beside the sibling files in this
folder, which use plain `E01`, `E02`, and so on.

Status words are the skill's: pass, fail, open, not applicable.
"Open" means this read-only run could not settle it,
never that the candidate failed.

## This repository's own verdicts on onchange (counterevidence search)

The prompt asked for this candidate's name to be searched across `doc/` and the package docs,
and for every verdict found to be recorded as counterevidence.
The search was
`rg --line-number --no-heading --hidden --glob '!.git' --glob '!node_modules' --word-regexp -i "onchange" .`
over the whole checkout at `main`, plus the same word-regexp search over `README.md`,
`AGENTS.md`, `CLAUDE.md`, `SECURITY.md`, `.github/`, `.claude/` and `.agents/`.

Every hit outside this vet and outside the vet report itself, across four files:

- `doc/audit/file-enforcer.md:544`, under the heading
  "Tier two: layer components a reconciler calls" and the sub-heading
  "Watchers, the A6 trigger layer",
  records the line `onchange` (`Qard/onchange`), "runs a command on glob-matched changes".
  Verdict: a layer component a reconciler calls, not a reconciler.
  That verdict is about file-enforcer's job (owning a reconcile loop over derived files),
  and it says nothing against running a command on changes,
  which is exactly this vet's job.
- `doc/audit/file-enforcer.md:264` lists `onchange` among tools absent from that audit's
  registry capture (`mise registry`), as evidence of a registry blind spot.
  Verdict: discovery-scope note, not a judgement of the tool.
- `doc/planning/garage-bisync-run-on-file-change.md:173` records the owner's round 5 answer
  that the third-party vet goes on with Watchman, onchange, modd and directory-watcher.
  Verdict: admission to this vet, not an assessment.

No handover, troubleshooting doc, decision record, package `README.md` or `package-paused/`
document carries a verdict on onchange.
The other two hits are unrelated JavaScript event names
(`package/music-player/design/support.js:319`, `package/music-player/design/candidates/support.js:319`).

So there is no repository precedent against this candidate,
and no repository precedent for it either,
beyond the file-enforcer tier-two placement,
which classes it as a trigger layer rather than a reconciler in a different job.

## Hard-constraint records

### Licence and inspectable source

#### OC-E01 Licence text and file path

Candidate and version: onchange 7.1.0.
Claim: the licence is MIT, with the complete licence text in the repository at the released tag.
Gate: hard constraint "open source with inspectable source";
skill section "License and dependency surface".
Status: pass.
Primary evidence: `LICENSE:1-5` in the clone reads
`MIT License` / `Copyright (c) 2013 Stephen Belanger` /
`Permission is hereby granted, free of charge, to any person obtaining a copy`.
The file is 8 lines: the MIT text with the copyright line filled in.
`package.json:14` declares `"license": "MIT"`.
Access date: 2026-11-25.
Corroboration: `curl --silent https://registry.npmjs.org/onchange` reports
`license` `"MIT"` for 7.1.0, and `gh api repos/Qard/onchange` reports
`"license":"MIT"`.
Counterevidence: none found.
Outcome: pass. A permissive licence raises no compatibility question for a tool that is run,
not linked into this repository's code.

#### OC-E02 Released version has inspectable source at the tag

Candidate and version: onchange 7.1.0.
Claim: the source that produced the published package is readable at the release tag,
and the published package carries that same source.
Gate: hard constraint "open source with inspectable source";
skill section "Production source" and overlay "Native, Wasm, and prebuilt overlay".
Status: pass.
Primary evidence: the clone is at tag commit `4337dee`,
and contains every code path this file cites:
`src/index.ts` (the library: watch, queue, spawn, kill),
`src/bin.ts` (the CLI: argument parsing, exclusion file, startup),
`src/echo.ts` (a three-line stdin-to-stdout helper used by `--outpipe`).
`tsconfig.json:2-16` compiles `src/**/*` to `dist` with `"strict": true`,
`"target": "ES2018"`, `"declaration": true`, `"sourceMap": true`,
`"inlineSources": true`.
The published tarball is built from that source by its own manifest:
`package.json:23` `"build": "rimraf dist/ && tsc"` and
`package.json:26` `"prepare": "npm run build"`.
Corroboration, the strongest available here:
each shipped file carries an inline source map whose `sourcesContent` can be compared byte for
byte with the tag.
Downloaded `https://registry.npmjs.org/onchange/-/onchange-7.1.0.tgz` and compared:

- `dist/index.js.map` names `../src/index.ts`, and its `sourcesContent[0]` equals
  `src/index.ts` at the tag: `true`.
- `dist/bin.js.map` names `../src/bin.ts`, and its `sourcesContent[0]` equals
  `src/bin.ts` at the tag: `true`.
- `dist/echo.js.map` names `../src/echo.ts`, and its `sourcesContent[0]` equals
  `src/echo.ts` at the tag: `true`.

The tarball's `package.json` is identical to the tag's: a JSON-normalised `diff` of the two
printed no differences.
Access date: 2026-11-25.
Outcome: pass. The registry package and the tag carry the same source,
so a reader can audit exactly what runs.

### Provenance of what a user installs

Onchange ships no per-platform artifacts.
The same npm tarball is what a user installs on Linux x86-64, macOS arm64 and Windows x64,
and the JavaScript inside it is platform-neutral.

#### OC-E03 The registry tarball maps to the tag, and there is nothing else to install

Candidate and version: onchange 7.1.0.
Claim: for each of the three target platforms the installed artifact is the npm tarball
`onchange-7.1.0.tgz`, built from tag `v7.1.0`, with no separate download step.
Gate: hard constraint "open source with inspectable source" and overlay
"Native, Wasm, and prebuilt overlay" (release workflow, target matrix, source-to-artifact mapping).
Status: pass.
Primary evidence:

- `bin` is `dist/bin.js` (`package.json:11-13`), and the shipped file starts with
  `#!/usr/bin/env node` (`dist/bin.js:1`), so npm's bin shim runs it with the user's Node.
- `prepare` runs the build at publish time (`package.json:26`), and the tarball contains the
  built `dist/`, so a consumer's install runs no build.
- No `postinstall`, `preinstall` or `install` script exists in `package.json`,
  so a consumer's install runs no lifecycle script of this package;
  the only manifest script that builds anything is `prepare`, which npm runs when a package is
  published and when it is installed from a Git URL, not when it is installed from the registry
  tarball.
- The tarball holds exactly 12 files: `dist/bin.{js,d.ts,js.map}`,
  `dist/echo.{js,d.ts,js.map}`, `dist/index.{js,d.ts,js.map}`, `LICENSE`,
  `package.json` and `README.md`.
- The repository has no release workflow: there is no `.github/` directory at the tag,
  so the tag's whole CI surface is `.travis.yml:1-5`,
  `language: node_js` with `node_js: [10, stable]`, and that file contains no deploy step.
- `gh api repos/Qard/onchange/releases/tags/v7.1.0` reports `"assets":[]`,
  so there is no release binary, archive or checksum file to compare.
- The published 7.1.0 carries no build attestation:
  `versions.7.1.0.dist.attestations` is absent from the registry document,
  which is expected for a 2020 publish (npm provenance began in 2023).

Access date: 2026-11-25.
Corroboration: the `gitHead` in the registry equals the tag object's commit,
and the tarball's inline source maps equal the tag's sources (`OC-E02`).
Counterevidence: a reader who wants a CI-built artifact will not find one.
What exists is a maintainer-machine `npm publish` from the tag,
with the mapping proved afterwards by the shipped source maps,
not by a workflow file.
That is weaker than a workflow that builds the artifact from the tag in public CI,
and it is recorded as a caveat rather than a failure,
because the mapping itself is verified and the shipped code is plain JavaScript,
so no compiler flags or host functions are hidden in it.
Outcome: pass with the caveat above.

#### OC-E04 Install path on each of the three target platforms

Candidate and version: onchange 7.1.0.
Claim: on Linux x86-64, macOS arm64 and Windows x64 the user installs the same tarball through
npm, and the only platform-specific code that arrives is Chokidar's optional macOS native addon.
Gate: hard constraint "runs on Linux x86-64, macOS arm64 and Windows x64";
overlay "Native, Wasm, and prebuilt overlay".
Status: pass with the macOS caveat in `OC-E06`.
Primary evidence, per platform:

- Linux x86-64: `npm install onchange` writes the tarball's JavaScript and installs
  Chokidar 3.6.0 and its pure-JavaScript dependencies.
  `fsevents` declares `"os": ["darwin"]` (`dep-fsevents-2.3.3/package/package.json`),
  so npm skips the optional dependency everywhere else.
  No native module, no compilation step, no network fetch beyond the registry.
- Windows x64: the same tarball and the same dependency set, again with `fsevents` skipped by
  its `os` field. `bin` is a Node script, so npm's Windows shim runs it with `node`.
- macOS arm64: the same tarball, plus the optional `fsevents` 2.3.3,
  which is a prebuilt universal Mach-O binary (`fsevents.node`, x86_64 and arm64 slices)
  shipped inside the npm tarball; see `OC-E06`.
- Chokidar itself needs Node `>= 8.10.0`
  (`versions.3.6.0.engines`, `https://registry.npmjs.org/chokidar`),
  and onchange declares no `engines` of its own,
  so nothing in the dependency graph enforces a floor beyond Chokidar's.

Access date: 2026-11-25.
Outcome: pass. One tarball serves all three platforms,
with a single prebuilt native addon on macOS that the candidate's own repository does not build.

#### OC-E05 Checksums, signature and release assets

Candidate and version: onchange 7.1.0.
Claim: there is a checksum and a registry signature for the installed artifact, and no
maintainer signature.
Gate: overlay "Native, Wasm, and prebuilt overlay" (checksum or signature, release verification).
Status: pass for integrity; open for maintainer identity.
Primary evidence:

- `https://registry.npmjs.org/onchange` publishes
  `versions.7.1.0.dist.shasum` `a6f0f7733e4d47014b4cd70aa1ad36c2b4cf3804` and
  `dist.integrity` starting `sha512-ZJcqsPiWUAUpvmnJri5TPBooqJOPmC0ttN65juhN15Q8xA+Nbg3BaxBHXQ`.
- This run downloaded the tarball and hashed it locally:
  `sha1sum` gives `a6f0f7733e4d47014b4cd70aa1ad36c2b4cf3804` (equal to `shasum`),
  and base64 of the sha512 digest gives exactly the `integrity` string above
  (hex `64972ab0f896500529be69c9ae2e533c1a28a8938f982d2db4deb98ee84dd7943cc40f8d6e0dc16b10475d0e39122b2d28a94a1256f479d99b3d69ca48192f32`).
  The tarball is 11836 bytes, 12 files.
- `versions.7.1.0.dist.signatures[0].keyid` is
  `SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA`,
  the npm registry's own signing key, with an OpenPGP block in `dist["npm-signature"]`.
  That signature attests what the registry served, not who built it.
- There is no maintainer GPG signature, no `.sig` or `.asc` asset, no SBOM and,
  as `OC-E03` records, no attestation.

Access date: 2026-11-25.
Outcome: integrity is verifiable end to end, and identity is not.
Recorded as: pass on checksum, open on signature,
with the note that a registry signature plus a source-map match is the whole chain available
for this 2020 package.

#### OC-E06 The macOS native boundary is Chokidar's optional `fsevents` addon

Candidate and version: onchange 7.1.0, with Chokidar 3.6.0 and `fsevents` 2.3.3.
Claim: on macOS the watch backend is a prebuilt third-party native binary,
so the prebuilt-artifact question attaches to `fsevents` and not to onchange.
Gate: overlay "Native, Wasm, and prebuilt overlay"; skill section "License and dependency surface".
Status: pass, with the provenance boundary named.
Primary evidence:

- Chokidar declares `fsevents` as an optional dependency
  (`versions.3.6.0.optionalDependencies`, `"fsevents":"~2.3.2"`),
  and `fsevents` declares `"os": ["darwin"]`.
- The published `fsevents` 2.3.2 and 2.3.3 tarballs each contain
  `fsevents.node`, a `Mach-O universal binary with 2 architectures: [x86_64 ...] [arm64 ...]`
  (the `file` command's reading of both tarballs' `package/fsevents.node`).
- Both tarballs' `package.json` list `"files": ["fsevents.d.ts","fsevents.js","fsevents.node"]`
  and only `clean`, `build`, `test` and `prepublishOnly` scripts,
  so the binary is built by the fsevents project at its own publish
  (`"prepublishOnly": "npm run build"`, which runs `node-gyp`), not on the user's machine.
- `fsevents.js:9-11` throws `Module 'fsevents' is not compatible with platform ...`
  on any platform other than darwin, so the module is inert elsewhere even if present.
- Chokidar guards the module's use: `lib/fsevents-handler.js:197`
  `const canUse = () => fsevents && FSEventsWatchers.size < 128;`,
  so a module-level cap of 128 live streams decides whether a watcher instance uses FSEvents at
  all (`index.js:324-326`).

Access date: 2026-11-25.
Counterevidence, recorded because it is unresolved:
the registry's version document for `fsevents` 2.3.2 and 2.3.3 lists
`"install": "node-gyp rebuild"` among the scripts,
while the tarballs' own `package.json` (the manifest npm actually executes) has no `install`
script and the tarball contains no `binding.gyp`.
This run did not resolve which document governs;
it only records that the tarball would not compile even if that script ran.
Outcome: pass. The macOS artifact is a prebuilt, source-available, universal binary from a
separate project with its own archive, and the candidate's own install carries no native code.

### Watching backends in source for the three target systems

Onchange holds no watch code of its own:
`src/index.ts:180-186` builds the watcher with
`const watcher = chokidar.watch(matches, { cwd, ignored, ignoreInitial, usePolling, interval, awaitWriteFinish, });`
and everything else in the file reacts to that watcher's events (`src/index.ts:254-268`).
The backend is therefore Chokidar's choice, made in its constructor:
`index.js:389-393` `if (opts.useFsEvents) { this._fsEventsHandler = new FsEventsHandler(this); } else { this._nodeFsHandler = new NodeFsHandler(this); }`.

#### OC-E07 Linux x86-64 backend: inotify through `fs.watch`

Candidate and version: onchange 7.1.0 with Chokidar 3.6.0.
Claim: on Linux the watcher is Node's `fs.watch`, which libuv implements with inotify.
Gate: hard constraint "runs on Linux x86-64".
Status: pass.
Primary evidence:

- `index.js:321-326` sets the default and the fallback:
  `if (undef(opts, 'useFsEvents')) opts.useFsEvents = !opts.usePolling;` then
  `const canUseFsEvents = FsEventsHandler.canUse(); if (!canUseFsEvents) opts.useFsEvents = false;`.
  Without the darwin-only `fsevents` module, `canUse()` is false, so the Node handler is used.
- `index.js:328-331` `if (undef(opts, 'usePolling') && !opts.useFsEvents) { opts.usePolling = isMacos; }`,
  so on Linux polling stays off unless the user passes `--poll`.
- `lib/nodefs-handler.js:119` `return fs.watch(path, options, handleEvent);`
  (3.3.1: `lib/nodefs-handler.js:118`).
- Node's own documentation for `fs.watch` states the backend:
  "On Linux systems, this uses inotify(7)" (`https://nodejs.org/docs/latest-v22.x/api/fs.html`,
  section `fs.watch`, subsection "Availability").
- onchange exposes polling as a fallback: `src/index.ts:168-169`
  `const usePolling = options.poll !== undefined;` with
  `--poll`/`-p` from `src/bin.ts:40,62`, documented at `README.md:107-113`.

Access date: 2026-11-25.
Outcome: pass. Linux watching is inotify via libuv, with an opt-in polling fallback.

#### OC-E08 macOS arm64 backend: FSEvents through the `fsevents` addon

Candidate and version: onchange 7.1.0 with Chokidar 3.6.0 and `fsevents` 2.3.3.
Claim: on macOS with the optional addon installed, the watcher is an FSEvents stream per watched
path; without the addon it falls back to polling through `fs.watchFile`.
Gate: hard constraint "runs on macOS arm64".
Status: pass.
Primary evidence:

- `index.js:321-322` leaves `useFsEvents` on by default when polling is not requested,
  and `lib/fsevents-handler.js:197` returns true when the module loaded.
- The stream is created at `lib/fsevents-handler.js:93-96`:
  `const createFSEventsInstance = (path, callback) => { const stop = fsevents.watch(path, callback); ... }`,
  reached from `lib/fsevents-handler.js:150` `watcher: createFSEventsInstance(watchPath, (fullPath, flags) => { ... })`.
- `fsevents.js` (2.3.2 and 2.3.3) wraps the native addon whose README describes watching
  "changes to a given directory tree" (`dep-fsevents-2.3.2/package/README.md:6`).
- If the addon is missing, `index.js:328-331` forces polling on macOS
  (`opts.usePolling = isMacos`), and the Node handler then polls with `fs.watchFile`
  (`lib/nodefs-handler.js:265`), so the fallback is polling rather than a silent no-op.

Access date: 2026-11-25.
Outcome: pass. The macOS backend exists and is the platform's native event API.

#### OC-E09 Windows x64 backend: `fs.watch` through ReadDirectoryChangesW

Candidate and version: onchange 7.1.0 with Chokidar 3.6.0.
Claim: on Windows the same Node handler is used, and Node's `fs.watch` maps to
`ReadDirectoryChangesW`.
Gate: hard constraint "runs on Windows x64".
Status: pass.
Primary evidence:

- The handler choice at `index.js:389-393` is `useFsEvents`, which
  `index.js:326` forces to false when `fsevents` cannot load, and `fsevents` cannot load
  off darwin (`fsevents.js:9-11` throws).
- `index.js:328-331` sets polling on macOS only, so Windows uses event-driven `fs.watch`.
- `lib/nodefs-handler.js:119` `fs.watch(path, options, handleEvent)`, and Node's manual states
  "On Windows systems, this feature depends on ReadDirectoryChangesW"
  (`https://nodejs.org/docs/latest-v22.x/api/fs.html`, `fs.watch`, "Availability").
- Spawning is portable too: `src/index.ts:70-74` spawns through cross-spawn,
  which exists to resolve Windows command shims.
  `README.md:24` records the one usage caveat:
  "NOTE: Windows users may need to use double quotes rather than single quotes."

Access date: 2026-11-25.
Counterevidence: none found in source;
the Windows path is also the one with the least independent evidence in this repository
(the issue tracker has Windows problems recorded, see `OC-M02`, `#122` and `#133`).
Outcome: pass on the backend's existence and portability; unmeasured here.

### Recursion, including folders created after the watcher starts

Onchange passes the user's globs straight to Chokidar and sets no `depth`
(`src/index.ts:180-186`), so Chokidar's default applies:
`lib/nodefs-handler.js:560-561`
`const oDepth = this.fsw.options.depth; if ((oDepth == null || depth <= oDepth) && ...)`.
A `null` depth means no limit.

#### OC-E10 Linux and Windows: a folder created after start gets its own watch

Candidate and version: onchange 7.1.0 with Chokidar 3.6.0.
Claim: every directory under the watched root is watched, and a directory created later is
watched as soon as its parent's watch reports it.
Gate: hard constraint "watches every folder under the root recursively, including folders
created after it starts".
Status: pass, with the race noted below.
Primary evidence, in order:

- `lib/nodefs-handler.js:546-575` is `_handleDir`. It scans the directory
  (`:563` `await this._handleRead(dir, initialAdd, wh, target, dir, depth, throttler);`)
  and then installs a watch whose listener rescans that directory on every event:
  `:567-572` `closer = this._watchWithNodeFs(dir, (dirPath, stats) => { if (stats && stats.mtimeMs === 0) return; this._handleRead(dirPath, false, wh, target, dir, depth, throttler); });`
  (3.3.1: `lib/nodefs-handler.js:531-555`, listener at `:553-555`).
- `lib/nodefs-handler.js:456-472` is `_handleRead`. It reads one level
  (`:468-472` `let stream = this.fsw._readdirp(directory, { fileFilter: ..., directoryFilter: ..., depth: 0 })`)
  and, for each entry not seen before, recurses:
  `:492-498` `if (item === target || !target && !previous.has(item)) { ... this._addToNodeFs(path, initialAdd, wh, depth + 1); }`
  (3.3.1: `:440-482`, recursion at `:482`).
- `lib/nodefs-handler.js:587-650` is `_addToNodeFs`. For a directory it calls
  `:617` `closer = await this._handleDir(wh.watchPath, stats, initialAdd, depth, target, wh, targetPath);`
  (3.3.1: call at `:600`), which installs the watch described above.
- `lib/nodefs-handler.js:302-303` wires errors from each `fs.watch` to
  `this._boundHandleError = (error) => fsW._handleError(error);`.

So the chain for a folder created after start is:
the parent's `fs.watch` fires, `_handleRead` rescans the parent,
the new entry is unknown, `_addToNodeFs` is called, it stats the entry,
`_handleDir` installs a watch on it, and `_handleRead` scans it,
which recurses into any children it already has.

Limits worth recording beside the pass:

- There is a race between a directory's creation and the arrival of its watch:
  its first scan finishes before the watch is installed (`:563` before `:567`),
  so a write inside a brand-new directory in that gap can be missed.
  The source does not claim otherwise and no test at the tag covers it
  (`test/queue.js` and `test/concurrent.js` watch one existing file, `test/queue.js:21-27`).
- Chokidar rescans a directory at most once per second when the watch path has no glob
  (`lib/nodefs-handler.js:460-463`), but onchange always passes globs,
  so that throttle is bypassed here.
- One `fs.watch` exists per directory, so the watch budget for `~/Plain` is one per folder:
  1454 (`doc/planning/garage-bisync-run-on-file-change.md`, "Facts measured"),
  against `fs.inotify.max_user_watches = 524288` on this PC (same section).

Access date: 2026-11-25.
Outcome: pass. The mechanism that adds watches for newly created folders is present and is the
same one used for Windows; the miss window is recorded as a limit, not a failure.

#### OC-E11 macOS: a recursive FSEvents stream plus Chokidar's own tree walk

Candidate and version: onchange 7.1.0 with Chokidar 3.6.0 and `fsevents` 2.3.3.
Claim: on macOS the watched tree is scanned once at start and then covered by an FSEvents stream,
which reports events from the whole subtree, including directories created after the stream
starts.
Gate: hard constraint "watches every folder under the root recursively, including folders
created after it starts", macOS half.
Status: pass, on the addon's documented behaviour rather than on the kernel API's specification.
Primary evidence:

- `_addToFsEvents` scans the whole directory tree once, with Chokidar's own filters,
  emitting add events for everything it finds (`lib/fsevents-handler.js:446-496`,
  the readdirp at `:470-496`).
- It then installs one stream for the requested path:
  `lib/fsevents-handler.js:509-519`
  `if (opts.persistent && forceAdd !== true) { ... this.initWatch(realPath, path, wh, processPath); }`,
  through `initWatch` at `:427-436` and `_watchWithFsEvents` at `:304-362`,
  which creates the stream at `:93-96`
  `const createFSEventsInstance = (path, callback) => { const stop = fsevents.watch(path, callback); ... }`.
- The event's own path decides what happened, through the switch at
  `lib/fsevents-handler.js:341-350` on `FSEVENT_CREATED`, `FSEVENT_MODIFIED`, `FSEVENT_DELETED`
  and `FSEVENT_MOVED`, so a directory created under the root arrives as an event carrying a path,
  not as a new watch to install.
- Chokidar consolidates sibling watch paths onto a higher-level stream once ten or more of them
  share a prefix (`couldConsolidate` at `lib/fsevents-handler.js:182-194`, threshold 10 at `:81`).
- The addon's own description is "changes to a given directory tree"
  (`dep-fsevents-2.3.2/package/README.md:6`).
- The module-level cap of 128 live streams (`lib/fsevents-handler.js:197`) is not reached by a
  single watch root.

Access date: 2026-11-25.
Counterevidence: this run did not read the native addon's C source, which is outside the
npm tarball, so the recursive coverage is recorded from the module's README and Chokidar's
reliance on it, not from the kernel API's specification.
Outcome: pass with that caveat.

### Running the command

#### OC-E12 The command is an argv vector, never a shell string

Candidate and version: onchange 7.1.0.
Claim: `onchange '<globs>' -- <command> ...` runs the command as an executable plus arguments,
with no shell in between, so `systemctl --user start rclone-bisync-garage.service` runs as written.
Gate: hard constraint "runs an arbitrary command, here `systemctl --user start rclone-bisync-garage.service`";
capability "how it runs the command".
Status: pass.
Primary evidence:

- The CLI splits its own arguments from the command at the `--` token:
  `src/bin.ts:151-155` `const index = args.indexOf("--"); if (index) return [args.slice(0, index), args.slice(index + 1)]; return [args, []];`.
- The command vector is templated once per argument and spawned directly:
  `src/index.ts:144-146` `const command = options.command ? options.command.map((arg) => template<State>(arg)) : [];`
  then `src/index.ts:70-74`
  `this.childCommand = spawn(this.command[0], this.command.slice(1), { cwd, env, stdio, });`.
- `spawn` there is cross-spawn's (`src/index.ts:3`), which calls Node's `spawn` with
  `shell: false` semantics and only resolves the executable through `PATH`
  (and, on Windows, through command shims).
- The child's working directory is onchange's own (`src/index.ts:136`
  `const cwd = options.cwd ? resolve(options.cwd) : process.cwd();`,
  settable with `-c`/`--cwd`, `src/bin.ts:31,52`),
  its environment is onchange's environment unchanged (`src/index.ts:140`
  `const env = options.env || process.env;`),
  and its stdio is onchange's own stdin, stdout and stderr
  (`src/index.ts:62-67`), so the child can be interactive and its output is not captured.
- The exit is observed, not judged: `src/index.ts:76-81`
  `this.childCommand.on("exit", (code, signal) => { this.log(\`command ${exitMessage(code, signal)}\`); ... });`
  where `exitMessage` (`src/index.ts:286-288`) merely formats the code.
  Nothing compares the code to zero, so a failing command does not stop, restart or exit the
  watcher. The fact is recorded, not scored.

Access date: 2026-11-25.
Counterevidence, a usage constraint read from source:
`src/bin.ts:151-155` decides the split with
`const index = args.indexOf("--"); if (index) return [args.slice(0, index), args.slice(index + 1)];`.
An absent `--` gives `index` `-1`, which is truthy, so the branch is taken:
the last glob is dropped from the match list and the command list becomes `args.slice(0)`,
the whole argument list.
A `--` as the very first token gives `index` `0`, which is falsy, so no command is parsed at all.
A caller must therefore pass `--` with at least one glob before it, which is the documented form
(`README.md:22-26`).
Outcome: pass.

#### OC-E13 Changed paths never reach the environment or stdin, and are never quoted for a shell

Candidate and version: onchange 7.1.0.
Claim: the changed path reaches the command only where the user writes a `{{...}}` placeholder in
an argument, it is substituted literally into that argument, and because there is no shell
there is no quoting step at all.
Gate: capability "how changed paths are put into the command line, the environment or stdin,
and how they are quoted".
Status: pass, with one exception for `--outpipe`.
Primary evidence:

- The state object built per event is `src/index.ts:218-226`:
  `event`, `changed`, `file`, `fileExt`, `fileBase`, `fileBaseNoExt`, `fileDir`.
- Substitution happens only inside arguments the user wrote placeholders in:
  `src/index.ts:238-244` `new Job( log, command.map((arg) => arg(state)), outpipe?.(state) )`,
  where each `arg` is the template compiled at `src/index.ts:145`.
- The template library inserts values literally:
  `dep-template-1.2.0/package/dist/index.js:113-122`
  `return (data) => { let result = ""; for (const value of values) { result += typeof value === "string" ? value : value(data); } return result; };`
  There is no escape, quote or sanitise step.
- Environment: passed through untouched (`src/index.ts:140`), never augmented with the path.
  The only `process.env` occurrence in the shipped `dist/index.js` is the read at
  `dist/index.js:86` `const env = options.env || process.env;`, never an assignment.
- Stdin: the child inherits onchange's stdin (`src/index.ts:62-67`), so no path is written there.
- Quoting: onchange's only quoting helper, `src/index.ts:293-295`
  `return \`"${str.replace(/["\\\\$`!]/g, "\\\\$&")}"\`;`, is used only to build its own
  `echo.js` invocation (`src/index.ts:10-11`, `src/index.ts:275-283`), never for user arguments.
- Because no shell parses the argument vector, a path with spaces, parentheses, brackets or quotes
  is one argv element wherever the user puts `{{file}}` in its own argument,
  which is what a path-safe invocation needs.
  This is a reading of the code, not a measurement.
- The exception: `--outpipe` is run as a shell string.
  `src/index.ts:52` `this.childOutpipe = spawn(this.outpipe, { cwd, env, stdio, shell: true });`,
  and `src/index.ts:275-283` templates the path into that string before it reaches the shell.
  Any `--outpipe` use therefore puts file-name text through a shell unquoted.
  `--outpipe` is not needed for this design's command, and it is documented as optional
  (`README.md:115-123`); the risk is recorded, not exercised.
- The child's view of the path is relative to the watch cwd:
  Chokidar emits `sysPath.relative(opts.cwd, path)` when `cwd` is set
  (`index.js:562`), and onchange always sets it (`src/index.ts:181`).

Access date: 2026-11-25.
Outcome: pass for the command path this design needs, with `--outpipe` recorded as the one
shell-string surface.

### Busy behaviour and the queue

#### OC-E14 Default busy behaviour: exactly one of the four categories, and it is queue

Candidate and version: onchange 7.1.0.
Claim: with no options, a change that arrives while the command runs is queued,
and the next run starts when the running command exits.
Gate: hard constraint "queues one more run of its command natively when events arrive while the
command runs"; capability "runs the command to completion and never kills a run in progress".
Status: pass.
Primary evidence:

- Concurrency default: `src/index.ts:142` `const jobs = Math.max(options.jobs || 0, 1);`,
  documented as "default is `1`" at `README.md:83-89`.
- The two data structures: `src/index.ts:151-152`
  `const running = new Set<Job>(); const queue = new Deque<Job>();`.
- Dequeue, first in first out, one per free slot: `src/index.ts:192-212`
  `function dequeue() { if (queue.size === 0) return; if (running.size >= jobs) return; const job = queue.popLeft(); running.add(job); job.start(..., () => { running.delete(job); if (delay > 0) return setTimeout(dequeue, delay); return dequeue(); }); }`.
- Enqueue on every event: `src/index.ts:238-246` pushes a fresh `Job` and calls `dequeue()`.
- The event source is Chokidar's `all` event (`src/index.ts:254-258`),
  filtered only by `--filter` (`src/index.ts:150,255`).
- The run is considered finished at the child's `exit` event (`src/index.ts:76-81`).
  For this design's command that event is the `systemctl --user start` client exiting,
  which happens before the sync finishes
  (`doc/planning/garage-bisync-run-on-file-change.md`, "Owner answers, round 4": killing the
  client does not stop the sync).

So the default is one of the four categories named by the prompt, and it is queue:
serialised, FIFO, run to completion.

Two properties of that queue matter here and are recorded without scoring:

- It does not coalesce. One `Job` is queued per event (`src/index.ts:238-246`),
  so N changes during a run leave N pending runs,
  not one. For `~/Plain` that means a large batch, a branch switch or an rclone pull can queue
  hundreds or thousands of `systemctl --user start` invocations,
  each a separate child process. `--delay` inserts a wait between them
  (`src/index.ts:208-209`), which spaces the queue out but does not merge it.
  The queue is a `Deque` with no bound (`src/index.ts:152`).
- Whether each queued start turns into a new sync is systemd's decision, not onchange's:
  a `start` on a unit that is still active joins it.
  The design answers that with a trigger unit ordered after the sync unit
  (`doc/planning/garage-bisync-run-on-file-change.md`, "Owner answers, round 4"),
  which is outside this candidate.

Access date: 2026-11-25.
Corroboration: the maintainer states the same intent in `Qard/onchange#130`
(comment by `blakeembrey`, 2024-09-04):
"So version 5 switched to queue up tasks by default ... It shouldn't start a new process until
the first one is killed."
Outcome: pass.

#### OC-E15 Queue mode never stops a running command, and `--kill` is the only thing that does

Candidate and version: onchange 7.1.0.
Claim: without `--kill`, nothing in the run path terminates a running child;
with `--kill`, every event clears the queue and kills the running child and its process tree.
Gate: hard constraint "never interrupts a running command; each run finishes";
capability "runs the command to completion and never kills a run in progress".
Status: pass for the default; fail if `--kill` is passed.
Primary evidence:

- The only kill site is inside `enqueue`, guarded by the flag:
  `src/index.ts:229-232`
  `if (kill) { queue.clear(); // Remove pending ("killed") tasks. running.forEach((child) => child.kill(killSignal)); // Kill running tasks. }`.
- `kill` comes from `options.kill` (`src/index.ts:135` `const kill = !!options.kill;`),
  set only by `-k`/`--kill` (`src/bin.ts:25,38,60`).
- The kill is a process-tree kill through tree-kill:
  `src/index.ts:88-98` `treeKill(this.childOutpipe.pid, killSignal); ... treeKill(this.childCommand.pid, killSignal);`.
  tree-kill walks the tree with `ps -o pid --no-headers --ppid` on Linux (`dep-tree-kill-1.2.2/package/index.js:43-49`),
  `pgrep -P` on macOS (`:31-37`) and `taskkill /pid <pid> /T /F` on Windows (`:28-30`),
  then signals every pid it found (`:80-87` `process.kill(parseInt(pid, 10), signal)`).
- The permitted signals are narrowed to two: `src/bin.ts:82-89` rejects anything but
  `SIGINT` and `SIGKILL`, and the default is `SIGTERM` (`src/index.ts:143`)
  only for library callers who never use the CLI.
- Nothing else kills: there is no timeout, no memory cap and no restart path anywhere in
  `src/index.ts`; the child is never `detached`, and onchange installs no signal handler
  (the shipped `dist/bin.js` has none; the only `process.exit` calls are argument validation at
  `dist/bin.js:52,58,65,108`).
  `src/bin.ts:92-109` discards the close function `onchange(...)` returns,
  so the CLI's only shutdown is the default one for SIGINT and SIGTERM.

Access date: 2026-11-25.
Outcome: pass in the configuration this design would use (no `--kill`),
with a fail recorded for anyone who passes `--kill`.

#### OC-E16 The round 4 requirement, met natively

Candidate and version: onchange 7.1.0.
Claim: a change during a run starts one more run after the run ends,
with no wrapper.
Gate: hard requirement added in the design's round 3 and made binding in round 4,
"after a change that lands while a sync runs, one more sync starts after that run ends".
Status: pass, with the no-coalescing caveat `OC-E14` records.
Primary evidence: `OC-E14`'s source lines: the queue holds every event
(`src/index.ts:238-246`), concurrency defaults to one (`src/index.ts:142`),
and the completion callback starts the next queued job (`src/index.ts:206-210`).
Access date: 2026-11-25.
Outcome: pass natively. A wrapper is not needed to satisfy the requirement;
a wrapper may still be wanted to collapse bursts, which is a different question.

### Quiet period and debounce

#### OC-E17 There is no quiet period by default, and no option that is one

Candidate and version: onchange 7.1.0.
Claim: default onchange runs the command once per event as soon as the event arrives, with no
debounce; the two options that shape bursts are `--delay` (a pause between job starts,
default 0) and `--await-write-finish` (Chokidar's stability threshold,
off by default, 2000 ms when enabled).
Gate: capability "merges a burst of events into one run (a quiet period)".
Status: fail on merging, pass on the options existing.
Primary evidence:

- `src/index.ts:141` `const delay = Math.max(options.delay || 0, 0);`, documented as
  "set the amount of delay (in ms) between process changes" (`README.md:91-97`),
  and applied only after a job finishes (`src/index.ts:208-209`
  `if (delay > 0) return setTimeout(dequeue, delay);`).
  It therefore spaces queued runs, and does not merge events into one run.
- `src/index.ts:170-172`
  `const awaitWriteFinish = options.awaitWriteFinish ? { stabilityThreshold: options.awaitWriteFinish } : undefined;`
  is passed to Chokidar at `src/index.ts:186`.
  Chokidar only waits when that object exists (`index.js:594-620` `if (awf && (event === EV_ADD || event === EV_CHANGE) && this._readyEmitted)`),
  and when the threshold is missing it defaults to 2000 ms with a 100 ms poll
  (`index.js:368-370`), the numbers `README.md:99-105` quotes.
  Because onchange passes `stabilityThreshold` alone, the poll interval stays 100 ms.
  The wait applies per file and only after the initial scan, so it delays an event,
  it does not collapse a directory's worth of events into one run.
- Chokidar's own coalescing is small and fixed:
  `index.js:616-619` throttles repeat `change` events for the same path to one per 50 ms;
  `index.js:359` enables `atomic` by default when not polling and not using fsevents,
  and `index.js:576-592` folds an `unlink` followed by an `add` of the same path within 100 ms
  into a single `change`.
- Nothing in `src/index.ts` counts events, waits for silence, or deduplicates jobs
  in the queue: the queue is drained one job per event (`src/index.ts:238-246`).

Access date: 2026-11-25.
Counterevidence: the vet report's ledger entry says "`--delay` and Chokidar's
`awaitWriteFinish` shape bursts" and lists a quiet period only as a capability to record,
which agrees with what is recorded here.
Outcome: fail on the capability "merges a burst of events into one run":
no option provides it. The design has to accept one run per event, or add `--delay`,
or wrap the command.

### Path exclusion

#### OC-E18 Globs plus a gitignore file, with two defaults

Candidate and version: onchange 7.1.0.
Claim: paths can be excluded by repeatable glob, a gitignore-spec file can be supplied,
and the built-in excluded set is exactly `**/node_modules/**` and `**/.git/**`.
Gate: capability "excludes paths by pattern (for example rclone's `*.partial` download names)".
Status: pass.
Primary evidence:

- Repeatable glob option: `src/bin.ts:19` `"--exclude": [String]` with
  `src/bin.ts:33` `"-e": "--exclude"`; documented at `README.md:51-57`.
- Gitignore-spec file: `src/bin.ts:18` `"--exclude-path": String`, built at
  `src/bin.ts:111-135` with the `ignore` package:
  `const ignorer = ignore(); ignorer.add(readFileSync(excludePath, "utf8"));`
  and matched through `relative(cwd, path)` (`src/bin.ts:127-129`);
  documented at `README.md:66-73`.
  A missing file is fatal with a message (`src/bin.ts:132-134`).
- Default excludes: `src/index.ts:174-177`
  `if (options.defaultExclude !== false) { ignored.push("**/node_modules/**", "**/.git/**"); }`,
  with `--no-exclude` as the off switch (`src/bin.ts:26,66`, `README.md:59-64`).
- How exclusion is applied: the list goes to Chokidar's `ignored` (`src/index.ts:182`),
  which Chokidar compiles into one `anymatch` predicate
  (`index.js:765-779` `this._userIgnored = anymatch(list, undefined, ANYMATCH_OPTS);`)
  and applies before a path is registered (`lib/nodefs-handler.js:589,606`),
  so an excluded directory is never watched.
- `*.partial` is not excluded by default and is not in Chokidar's editor-file pattern
  (`lib/constants.js:42` `exports.DOT_RE = /\..*\.(sw[px])$|~$|\.subl.*\.tmp/;`),
  so rclone's temporary names would need an explicit `-e '*.partial'`.
- Hidden files are matched like any other path unless excluded:
  the default ignore list contains no dot rule, and `DOT_RE` covers editor swap, backup and
  Sublime temp names only.
  That last point is a correction of the vet report's ledger entry, which says only that
  "hidden files are matched unless excluded" and does not mention these editor names.

Access date: 2026-11-25.
Outcome: pass.

### When the Linux inotify watch limit runs out

#### OC-E19 An error event that is dropped unless `--verbose`, partial silent loss, no fallback

Candidate and version: onchange 7.1.0 with Chokidar 3.6.0.
Claim: when `fs.watch` fails for a directory, Chokidar emits an `error` event,
onchange's handler writes it only when `--verbose` was passed,
and the watcher keeps running with that directory's own events unwatched.
Gate: capability "what it does when the inotify watch limit or the macOS and Windows
equivalents run out".
Status: fail, meaning the behaviour is silent loss, not a diagnostic and not a fallback.
Primary evidence, in the order the failure travels:

- `lib/nodefs-handler.js:105-123` wraps the watch:
  `try { return fs.watch(path, options, handleEvent); } catch (error) { errHandler(error); }`,
  and `lib/nodefs-handler.js:174-187` routes a later asynchronous failure of the same watch
  (`watcher.on(EV_ERROR, async (error) => { ... broadcastErr(error); })`) to the same handler.
- The handler is bound at `lib/nodefs-handler.js:302-303`
  `this._boundHandleError = (error) => fsW._handleError(error);` and passed at `:333`
  `errHandler: this._boundHandleError`.
- Chokidar decides what to report: `index.js:642-650`
  `_handleError(error) { const code = error && error.code; if (error && code !== 'ENOENT' && code !== 'ENOTDIR' && (!this.options.ignorePermissionErrors || (code !== 'EPERM' && code !== 'EACCES'))) { this.emit(EV_ERROR, error); } return error || this.closed; }`.
  `ENOSPC` is not in the suppressed list, so it is emitted.
  Chokidar's README documents the same failure from the user's side:
  `README.md:281-285` `Error: watch /home/ ENOSPC` with the advice to raise
  `fs.inotify.max_user_watches`.
- Onchange's whole error policy is one line:
  `src/index.ts:268` `watcher.on("error", (error: Error) => log(\`watcher error: ${error}\`));`
  and `log` is a no-op unless `--verbose` (`src/index.ts:155-159`
  `const log = options.verbose ? function log(message) { stdout.write(...) } : function () {};`).

So with default flags: no message, no exit, no retry, no switch to polling,
and the affected directory is simply not watched while its parent and children may still be.
The directory's scan still ran (`lib/nodefs-handler.js:563` precedes the watch at `:567`),
so files and subdirectories already inside it are registered and watched individually,
which makes the loss partial rather than total: events that only the failing directory's own
watch would have reported are the ones lost.
A reader who wants a diagnostic must pass `--verbose`, and even then onchange continues.

How likely this is here: `~/Plain` holds 1454 folders
(`doc/planning/garage-bisync-run-on-file-change.md`, "Facts measured"),
one watch per folder (`OC-E10`), against
`fs.inotify.max_user_watches = 524288` and `fs.inotify.max_user_instances = 1024`
on this PC (same section).
The ratio is roughly one watch per folder used out of 524288 available,
so exhaustion is not a realistic risk for this tree as measured.
One qualification read from source: Chokidar briefly takes a second watch on a file whose inode
changed, via `lib/nodefs-handler.js:372-375` `this.fsw._addPathCloser(path, this._watchWithNodeFs(file, listener))`,
so the count can exceed the directory count transiently.

Access date: 2026-11-25.
Counterevidence: the vet report's ledger entry says "watch-limit exhaustion is not documented",
which is true of onchange's own README and false of Chokidar's, whose README names `ENOSPC`;
the behaviour itself is undocumented in onchange and silent as read here.
Outcome: fail on the capability as a silent-loss behaviour.
Recorded as evidence for later scoring, not as an exit, because exhaustion is not plausible
at the measured watch count.

### Errors, exit codes and lifecycle

#### OC-E20 A failing command is logged and otherwise ignored

Candidate and version: onchange 7.1.0.
Claim: the candidate never exits and never restarts because its command failed;
a non-zero child status is only formatted into a `--verbose` log line.
Gate: capability record only; it decides how a service wrapper must watch the watcher.
Status: no gate; recorded.
Primary evidence: `src/index.ts:76-81` (exit handler), `src/index.ts:286-288`
(`exitMessage`), `src/index.ts:155-159` (log only under `--verbose`).
There is no `process.exitCode` assignment outside argument validation
(`dist/bin.js:52,58,65,108`).
Access date: 2026-11-25.
Outcome: recorded. A supervisor, not this candidate, has to notice that the sync is failing.
In this design the unit itself already reports failure through its own
`rclone-bisync-garage-alert@.service` drop-in
(`doc/planning/garage-bisync-run-on-file-change.md`, "Purpose").

#### OC-E21 No run at startup by default; `--add` would run once per pre-existing file

Candidate and version: onchange 7.1.0.
Claim: the initial scan of an existing tree produces no runs by default,
`--initial` produces one, and `--add` produces one run per pre-existing file.
Gate: capability record only.
Status: no gate; recorded, because `--add` on this tree would be pathological.
Primary evidence:

- `src/index.ts:167` `const ignoreInitial = options.add !== true;`, passed at `:183`,
  so Chokidar's initial `add` events are suppressed unless `--add` was given.
- `src/index.ts:250-251` `if (initial) enqueue("", "");` runs the command once at startup,
  before `ready`; `--initial`/`-i` documented at `README.md:41-47`.
- With `--add`, Chokidar emits one `add` per existing file after `ready`,
  and onchange enqueues one job per event (`src/index.ts:254-258`),
  so on this tree that is 10332 queued runs (`doc/planning/garage-bisync-run-on-file-change.md`,
  "Facts measured").
  Recorded as a usage constraint, not measured.

Access date: 2026-11-25.
Outcome: recorded.

#### OC-E22 No declared Node version range

Candidate and version: onchange 7.1.0.
Claim: the package declares no `engines`, so npm installs it on any Node,
while its dependencies require at least Node 8.10.
Gate: capability record only, information for the later platforms.
Status: no gate; recorded.
Primary evidence: no `engines` key in `package.json` (the whole file was read; keys are
`name`, `version`, `description`, `author`, `keywords`, `bin`, `license`, `main`, `typings`,
`files`, `scripts`, `repository`, `bugs`, `homepage`, `husky`, `lint-staged`, `publishConfig`,
`snyk`, `dependencies`, `devDependencies`);
Chokidar 3.6.0 declares `"engines": {"node": ">= 8.10.0"}`.
Access date: 2026-11-25.
Outcome: recorded.

### Security-relevant surfaces

#### OC-E23 No sockets, no network, no telemetry, no privileged operation; two shell surfaces

Candidate and version: onchange 7.1.0.
Claim: the candidate opens no listener and no connection, checks for no update, sends no
telemetry, and needs no privilege; the two child-process surfaces that could be abused are
`--outpipe` (a shell string) and `--kill` (a process-tree kill), both opt-in.
Gate: skill section "Production source" (credentials, network, filesystem, escaping, serialization)
and overlay "Human-auditability overlay" (concentration of process execution).
Status: pass, with the surfaces named.
Primary evidence:

- Every module the shipped code loads:
  `require(".")`, `require("arg")`, `require("fs")`, `require("ignore")`, `require("path")`
  in `dist/bin.js`; `require("@blakeembrey/deque")`, `require("@blakeembrey/template")`,
  `require("chokidar")`, `require("cross-spawn")`, `require("path")`, `require("tree-kill")`
  in `dist/index.js`.
  No `http`, `https`, `net`, `dgram`, `tls`, `dns`, `child_process.exec` or `eval` appears in
  either shipped file, and no `new Function` appears.
- Child processes: exactly two spawn sites, `src/index.ts:52` (`--outpipe`, with `shell: true`)
  and `src/index.ts:70-74` (the command, argv).
- Kill path: `src/index.ts:91,96` call tree-kill, which shells out to `ps` on Linux,
  `pgrep` on macOS and `taskkill /pid <pid> /T /F` on Windows
  (`dep-tree-kill-1.2.2/package/index.js:27-50`).
  It runs only under `--kill`.
- Filesystem reads: the tree being watched, and the optional `--exclude-path` file
  (`src/bin.ts:122-129`). No write of any kind: `grep -n -E "writeFile|mkdir|unlink|rename|rmdir|appendFile"`
  over `src/*.ts` returns no match at all, and the only stream writes are to stdout
  (`src/index.ts:157`, `src/echo.ts:3`).
- Network: none, so no update check and no telemetry;
  `package.json:47` `"snyk": true` is a marker for Snyk's own repository integration,
  not a runtime hook, and no `@snyk/protect` dependency exists.
- Privileges: none requested; no `sudo`, no setuid, no system service, no bus call.
  The candidate never mentions systemd; `systemctl` is only ever the user's command.
- Listener: none. The one socket in the tree is a test fixture,
  `test/scripts/slow.js:1` `require("net").createServer().listen();`,
  and the published tarball does not contain `test/` (`package.json:17-19` `"files": ["dist/"]`).
- Stdin and stdout are inherited by the child (`src/index.ts:62-67`), so anything on the
  watcher's stdin reaches the child; in a user unit stdin is not a terminal and stays empty.
- Shell-metacharacter risk from file names exists only through `--outpipe`
  (`src/index.ts:52` plus templating at `:275-283`), recorded in `OC-E13`.
  The command path itself cannot be injected through a file name,
  because it is spawned as argv with no shell.

Access date: 2026-11-25.
Outcome: pass, with `--outpipe` and `--kill` named as the only opt-in process-execution
surfaces and neither needed by this design.

## Maintenance audit

The skill's "Maintenance audit" section asks for issues and pull requests updated in the last 12
months, release cadence, maintainer concentration, and whether maintenance reaches a release.
Reference date for "last 12 months": 2025-11-25, one year before this run.

### OC-M01 Repository and tracker state

Candidate and version: onchange 7.1.0.
Claim: the repository is neither archived nor disabled, and its whole tracker holds five open
items, all untouched since 2024-09-04.
Gate: maintenance audit, repository state and tracker volume.
Status: no gate; recorded.
Primary evidence: `gh api repos/Qard/onchange` reports
`"archived":false`, `"disabled":false`, `"open_issues_count":5`, `"stargazers_count":826`,
`"forks_count":49`, `"subscribers_count":10`, `"created_at":"2014-01-18T08:50:47Z"`,
`"pushed_at":"2026-06-18T20:53:10Z"`, `"default_branch":"master"`,
`"license":"MIT"`.
Access date: 2026-11-25.
Outcome: recorded: a live repository with a very small open backlog.

### OC-M02 The ten most recently updated issues

Candidate and version: onchange 7.1.0.
Claim: in the last 12 months no issue was created and none was touched;
the nearest activity is a closed report from 2025-10-16.
Gate: maintenance audit, issues created or updated in the last 12 months.
Status: no gate; recorded.
Primary evidence: `gh api "repos/Qard/onchange/issues?state=all&sort=updated&direction=desc&per_page=40"`
returns, in update order,
`#132` (a PR, see `OC-M03`),
`#133` (created and updated 2025-10-16, closed 2025-10-18),
`#130` (open, updated 2024-09-04),
`#126` (a PR, open, updated 2024-09-04),
`#131` (a PR), `#4` (closed, updated 2024-07-29),
`#122` (closed, updated 2023-12-20), `#129` (closed, updated 2023-12-05),
`#124` (closed, updated 2023-11-30), `#49` (open, updated 2022-10-30).
So the ten most recently updated items span 2022 to 2026, and only one of them
(`#132`) falls inside the last 12 months.
Issues created inside the last 12 months: none.
Issue `#133`, the closest, is 13 months before this run.

Maintainer comments, counted separately from maintainer actions
(`gh api repos/Qard/onchange/issues/<n>/comments`):

- `#133` two comments, both by the reporter, the second withdrawing the report
  ("S-sorry, it turned out this issue wasn't package-dependent.")
  and linking `npm/cli#4225`. No maintainer comment.
- `#130` one comment, `blakeembrey`, 2024-09-04, explaining that version 5 queues by default
  and that dropping a pending run "might be considered a bug".
  This is the only maintainer statement this run found about the busy behaviour.
- `#4` five comments, two by `Qard` and three by reporters.
- `#122` seven comments, including two by `blakeembrey`; the report is a PowerShell failure.
- `#129` four comments, two by `blakeembrey`.
- `#124` one comment, `blakeembrey`.
- `#49` three comments, `blakeembrey` and two reporters; still open since 2017.
- `#128` no comments.
- `#79` seven comments, three by `blakeembrey`; a Docker report, closed.
- `#127` one comment, `blakeembrey`.

Maintainer actions (labels, assignment, closure) over the 15 most recently updated items:
no item has an assignee; labels exist on three (`#49` and `#79` `help wanted`,
`#127` `question`); everything else is unlabelled.
Closures happen, and almost always without a comment.

Stale unanswered bugs and compatibility failures found in that sample:

- `#49` "Way to stop listening on changes", open since 2017-04-07, last touched 2022-10-30.
- `#118` "Feature request: Run once, then exit", open since 2021-03-16, five comments
  including maintainer replies, never implemented.
- `#121` "Feature request: incremental initial run", open since 2022-01-10, no reply.
- `#70` "Onchange does not detect changes when used in a directory with parenthesis"
  (2018-10-10, closed 2020-04-28):
  the maintainer's first reply blames Chokidar
  ("This seems like it'd be an issue with `chokidar`"), and the close reads
  "Closing and leaving it up to chokidar to resolve."
  A path-with-parentheses failure was therefore closed without a fix or a reproduction.
- `#133` "Failed if installed path contains ampersand" (2025-10-16, closed 2025-10-18),
  withdrawn as an npm bug, not an onchange bug.
- `#122` "Doesn't work in PowerShell" (2022-01-20, closed 2023-12-20) and `#4`
  "How do I pass multiple commands with &&" are the Windows-adjacent reports;
  neither records a fix shipped in a release.

Access date: 2026-11-25.
Counterevidence for the maintainers' side: they do answer,
`blakeembrey` replied on `#130` in 2024, and `#129`, `#124`, `#127`, `#79` each got a reply.
Outcome: recorded. The tracker is a low-activity, low-volume one where reports are answered
sporadically and often closed without a shipped change.

### OC-M03 The ten most recently updated pull requests

Candidate and version: onchange 7.1.0.
Claim: one pull request was updated inside the last 12 months, it is a dependency security
upgrade, it was merged the day it was touched after 586 days open, and it is unreleased.
Gate: maintenance audit, pull requests updated in the last 12 months, review practice,
merge latency, unreviewed external fixes.
Status: no gate; recorded.
Primary evidence: `gh api "repos/Qard/onchange/pulls?state=all&sort=updated&direction=desc&per_page=12"`
returns, in update order:

- `#132` "[Snyk] Security upgrade cross-spawn from 7.0.3 to 7.0.5",
  opened 2024-11-09, merged 2026-06-18 (586 days), author `Qard` (a maintainer account),
  bot-generated. Reviews: one, by `jebitese`, `APPROVED`.
- `#126` "add explanation of chokidar's resource consumption", open since 2022-05-07,
  last touched 2024-09-04, author `fotoflo` (external), one maintainer comment
  (`blakeembrey`), no review, never merged.
- `#131` "[Snyk] Security upgrade @blakeembrey/template from 1.0.0 to 1.2.0",
  opened and merged 2024-09-04 (same day), author `Qard`, no review recorded.
- `#115` "Update help text", opened 2020-11-03, merged 2020-11-08 (5 days), external author
  `cjbarth`, one maintainer comment, no review.
- `#114` "feat: add more replacements", opened 2020-08-25, merged 2020-10-21 (57 days),
  external author `forivall`.
- `#113` `@types/node` update, opened and merged 2020-05-13, bot.
- `#112` "Fix `--exclude-path` feature", opened 2020-04-28, merged 2020-04-30 (2 days),
  external author `simnalamburt`.
- `#110` "Refactor package using TypeScript and `arg`", opened and merged 2020-04-25, maintainer.
- `#107` Snyk minimist upgrade, 2020, bot, merged within days.
- `#109` stdio fix, opened and merged 2020-04-21 (2 minutes), maintainer.
- `#106` Snyk minimist upgrade, 2020, bot, merged within days.
- `#102` tree-kill update, opened 2020-01-13, merged 2020-02-19 (37 days), external author.

So: inside the last 12 months, exactly one pull request was touched (`#132`, a bot-generated
security upgrade), it was merged, and it was reviewed by one non-maintainer account.
External code contributions either wait for months or never merge (`#126`, open since 2022).
There is no review recorded on any of the other eleven.

Access date: 2026-11-25.
Outcome: recorded. Maintenance exists at the level of dependency security upgrades,
not at the level of features or bug fixes.

### OC-M04 Release cadence, and how far the default branch runs ahead of the tag

Candidate and version: onchange 7.1.0.
Claim: the last publish was 2020-10-22, five commits sit unreleased on `master`, and none of the
maintenance work above reached a release.
Gate: maintenance audit, latest publish and whether maintenance merges reach a release.
Status: no gate; recorded.
Primary evidence:

- `https://registry.npmjs.org/onchange` lists 39 versions from 0.0.1 to 7.1.0,
  with `dist-tags.latest` still `7.1.0` and `time["7.1.0"]` `2020-10-22T00:54:16.861Z`.
  The document's `time.modified` is `2023-04-05T02:08:55.970Z`, also in the past.
- GitHub releases: the newest is `v7.1.0`, `published_at` `2020-10-22T00:55:29Z`.
- `gh api "repos/Qard/onchange/compare/v7.1.0...master"` reports `ahead_by: 5`:
  `332e3d69` "Update bin prompt to `{{file}}` (#115)", 2020-11-08;
  `2d1f2f16` "Create SECURITY.md", 2023-12-12;
  `504e1554` "Security upgrade @blakeembrey/template from 1.0.0 to 1.2.0 (#131)", 2024-09-04;
  `a9dce673` "Upgrade all dependencies", 2024-09-04;
  `5fec925c` "Upgrade cross-spawn from 7.0.3 to 7.0.5 (#132)", 2026-06-18.
  Master's `package.json` still says `"version": "7.1.0"`,
  so no release was cut for any of them.
- What those five commits change: `SECURITY.md` (+5 lines), `package-lock.json`,
  `package.json` (`arg` ^5.0.2, `chokidar` ^3.6.0, `cross-spawn` ^7.0.5,
  `@blakeembrey/template` ^1.2.0, build scripts moved to `ts-scripts`),
  `src/bin.ts` (help text `{{changed}}` to `{{file}}`),
  `src/index.ts` (trailing-comma formatting and `?.pid` guards in `kill()`),
  `test/concurrent.js`, `test/scripts/slow.js`.
  The queue, spawn and error handling this file cites are unchanged on `master`.
- Consequence for an installer: `npm install onchange` gets the 2020 code plus today's
  resolution of its ranges, which is not the same as master.
  The two security-relevant upgrades are already covered by the ranges as resolved today
  (`cross-spawn` ^7.0.1 resolves to 7.0.6, `@blakeembrey/template` ^1.0.0 resolves to 1.2.0),
  so a fresh install is not left on the vulnerable versions that PRs `#131` and `#132` fixed.

Access date: 2026-11-25.
Outcome: recorded. Release cadence is zero for six years;
maintenance merges do not reach a release.

### OC-M05 Maintainer concentration and disclosure channel

Candidate and version: onchange 7.1.0.
Claim: two people hold the project, and the security contact is a third-party platform.
Gate: maintenance audit, maintainer concentration and security reporting.
Status: no gate; recorded.
Primary evidence:

- `gh api "repos/Qard/onchange/contributors?per_page=100"` gives 23 entries:
  `blakeembrey` 87, `Qard` 21, `greenkeeper[bot]` 8, `snyk-bot` 4, then 19 people with 1 or 2
  commits each (`aegatlin` 2, `eush77` 2, the rest 1).
  Two accounts therefore carry 108 of the 141 listed contributions;
  `Qard` is Stephen Belanger, the copyright holder in `LICENSE:3`, and `blakeembrey` is the
  author of every commit that shaped the current source.
- The security policy is one screen and names an external party:
  `SECURITY.md` on `master` reads
  "To report a security vulnerability, please use the
  [Tidelift security contact](https://tidelift.com/security).
  Tidelift will coordinate the fix and disclosure."
  It was added 2023-12-12 and is not in any release.
- No security report or advisory appears in the tracker sample this run read.
  The repository's own advisories were not queried, so that is an absence in the sample,
  not a clean record.
  Dependency risk is handled by Snyk bot pull requests, four of which appear in `OC-M03`.

Access date: 2026-11-25.
Outcome: recorded. Contribution and decision-making are concentrated in two accounts,
and the published disclosure route is Tidelift's, not the tracker.

## Summary: each hard constraint as pass, fail or open

The hard constraints are the vet report's "Hard constraints" list,
plus the round 3 requirement it folds into that list.

Open source with inspectable source: pass, by `OC-E01` (MIT text at `LICENSE:1-8`) and
`OC-E02` (source at the tag, and the published package's inline source maps equal to it).

Runs an arbitrary command, here `systemctl --user start rclone-bisync-garage.service`: pass, by
`OC-E12` (argv through cross-spawn, no shell), with the `--` parsing constraint it records.

Runs on Linux x86-64, macOS arm64 and Windows x64: pass, by `OC-E04` (one npm tarball for all
three, no native code except Chokidar's optional macOS addon), `OC-E07` (inotify through
`fs.watch`), `OC-E08` (FSEvents through `fsevents`) and `OC-E09`
(`ReadDirectoryChangesW` through `fs.watch`).

Watches every folder under the root recursively, including folders created after it starts:
pass, by `OC-E10` for Linux and Windows (the parent's watch rescans, the new entry reaches
`_addToNodeFs`, `_handleDir` installs a watch and scans it) and `OC-E11` for macOS (a
recursive FSEvents stream over a one-time tree walk). The creation-to-watch race and the
unread macOS addon source are recorded as limits in those records,
and nothing here was measured at runtime.

After a change that lands while a sync runs, one more sync starts after that run ends: pass
natively, by `OC-E16` and `OC-E14`; no wrapper is needed for that requirement.

Never interrupts a running command; each run finishes: pass in the configuration this design
would use, by `OC-E15` (nothing kills unless `--kill` is passed);
the same record marks it failed for `--kill`.

Queues one more run of its command natively when events arrive while the command runs: pass,
by `OC-E14`, with the correction that it queues one run per event rather than one pending run,
since there is no coalescing and no bound on the queue.

Three further items the prompt asked to confirm or refute, which the vet report records as
capabilities rather than gates:

Quiet period or debounce and its default: fail on the capability, by `OC-E17`.
There is no quiet period; `--delay` (default 0) only pauses between queued job starts,
`--await-write-finish` defers individual Chokidar events (2000 ms threshold, 100 ms poll) and
does not merge a burst into one run, and Chokidar's own folding is 50 ms per path for `change`
and 100 ms for an `unlink` followed by `add`.

Path exclusion: pass, by `OC-E18`: repeatable `-e/--exclude` globs, `--exclude-path` for a
gitignore-spec file, `--no-exclude` to clear the built-in pair, and a built-in pair of exactly
`**/node_modules/**` and `**/.git/**`. `*.partial` is not among them.

What happens when the Linux inotify watch limit runs out: fail, by `OC-E19`.
Chokidar emits `ENOSPC` as an `error` event, onchange's single handler writes it only under
`--verbose`, and the watcher keeps running with that directory's own events unwatched:
no diagnostic by default, no retry, no polling fallback, no exit.
At the measured 1454 folders against `max_user_watches = 524288`, exhaustion is not plausible
for this tree.

Security-relevant surfaces: pass, by `OC-E23`: no socket, no connection, no update check, no
telemetry, no privilege, no filesystem write; the opt-in process-execution surfaces are
`--outpipe` (a shell string with unquoted path substitution) and `--kill` (a process-tree kill),
and this design needs neither.

## What this run did not do, and what stays open

- Nothing from the candidate or its dependencies was installed, built, tested or run,
  as instructed. No behaviour in this file is measured:
  the queue's timing, the burst behaviour on a real tree, the watch count on `~/Plain`,
  and every macOS and Windows claim are source readings.
- `~/Plain` was not listed and not touched, so the prompt's description of its file names
  (Japanese and Chinese characters, spaces, parentheses, brackets, quotes) is context from the
  brief, not something this run verified.
  The only recorded evidence about such names is the closed issue `#70` (parentheses in the
  run directory) and `#133` (an ampersand in the install path, withdrawn as an npm bug).
  Whether a path with brackets or quotes breaks glob matching is therefore open:
  it was reported once in 2018, closed unfixed with the cause attributed to Chokidar,
  and a read of Chokidar's glob path (`index.js:792-797`, `anymatch` over the tested string)
  suggests names are tested as strings rather than parsed as patterns,
  but that is inference, not a test.
- The candidate's own tests were read, not executed:
  `test/queue.js` (asserts that with the default jobs of 1 the output holds one line per
  completed run, and later all runs) and `test/concurrent.js` (same with `jobs: 10`).
  They cover the queue and nothing else: no exclusion test, no error test, no
  new-directory test, no platform matrix, and `.travis.yml` runs them on Node 10 and stable.
- The vet report's "External execution gate", "Finalist validation", scoring and recommendation
  remain untouched and open. Nothing here decides whether onchange should be used.
- The clone at `/home/user/temp/agent/onchange-source-for-garage-watcher-vet-2026-11-25`
  and the scratch folder
  `/home/user/temp/agent/watcher-targeted-evidence-onchange-scratch-2026-11-25`,
  which holds the verified tarballs of onchange, Chokidar 3.3.1 and 3.6.0, tree-kill,
  `@blakeembrey/template`, `fsevents` 2.3.2 and 2.3.3, and the registry documents,
  are deleted when this run ends,
  so every `path:line` above is meant to be read against a fresh clone and fresh downloads made
  with the commands recorded here.

## Where this file departs from the prompt or the ledger entry

- The prompt describes the watcher's command as "`systemctl --user start` of a small trigger unit"
  and the ledger entry as command `systemctl --user start rclone-bisync-garage.service`.
  Both forms are argv-only commands with no shell metacharacters, so `OC-E12`'s findings cover
  either; the record cites the service form because that is what the vet report names.
- The vet report's ledger entry says "`--delay` and Chokidar's `awaitWriteFinish` shape bursts".
  That is accurate as far as it goes, and the correction is in `OC-E17`:
  neither merges a burst into one run, `--delay` runs after a job finishes rather than before,
  and the burst-shaping that does exist inside Chokidar is a 50 ms same-path `change` throttle
  plus a 100 ms `unlink`-then-`add` fold.
- The vet report's ledger entry says "watch-limit exhaustion is not documented".
  It is not documented in onchange's README, but Chokidar's README documents `ENOSPC` by name,
  and onchange's handling of it is a silent drop unless `--verbose` (`OC-E19`).
  That is more specific than "not documented", and it is a fail on the capability.
- The vet report's ledger entry says "hidden files are matched unless excluded".
  That holds, and `OC-E18` adds what the ledger omits: editor swap, backup (`~`) and Sublime
  temp names are ignored by Chokidar whenever `atomic` is on, which is the default on Linux and
  Windows and off on macOS with fsevents.
- The vet report's ledger entry records the round 3 requirement as "met natively ... No wrapper
  needed", citing `src/index.ts:151-224` for the queue. The behaviour is confirmed by
  `OC-E14` and `OC-E16` on wider line ranges, and two properties the ledger does not state are
  recorded there: one pending job per event with no coalescing, and an unbounded queue.
- The prompt asked for "the release workflow file and lines that build each artifact from the tag,
  or the registry package and how it maps to the tag". There is no release workflow at this tag
  and no release artifact, so the record answers with the second branch (`OC-E03`):
  one registry tarball, its `prepare`-time build, and the inline source maps that tie the
  shipped JavaScript to the tag's TypeScript byte for byte.
- The prompt asked for "busy behaviour by default and with the option that queues".
  There is no option that turns queueing on, because queueing is the default and `--jobs` only
  raises concurrency (`OC-E14`); the option that changes the category is `--kill`, which drops
  pending runs and kills the running one (`OC-E15`).
