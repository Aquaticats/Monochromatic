# Renaming the Git wrapper program and its command word

Status: decided by the owner; step 1 is being built.
The work waited for the four lint branches of the crate (`refactor/git-policy-cli-lint-*`) to merge,
because the command word is read in `invocation/management*.rs`, which one of them was editing.
They merged on 2026-11-05; step 1 is built on `feat/git-policy-cli-program-rename`
(worktree `.claude/worktrees/git-policy-cli-program-rename`, from `a7e32e118e`, the gated suppression branch).
The proposed handling of the settings file and data folders is adopted as written, owner veto open.
Steps 2 and 3 (release, lock) wait for the owner's answer on when the one release of each crate happens
(`doc/handover/questions-for-the-owner.md`).

## What the owner decided

2026-11-04, after `AGENTS.md` VGW ("a word that fits any code (`native`, `core`, `helpers`) never stands alone")
named the program `cli-git-native` as a vague name:

- "Rename it too." (the installed program `cli-git-native`)
- Program name, through the question tool, verbatim note: "follow the actual crate's name."
  So the program becomes `monochromatic-git-policy-cli`, the crate's name.
- The command word, through the question tool: "B: rename the word too".
  So `git cli-git <command>` becomes `git monochromatic-git-policy-cli <command>`
  (`doctor`, `check`, `fix`, `migrate-to-subvolume`, `convert-worktree-to-subvolume`, `service run`, and the rest).
- The old spelling, through the question tool: "A: still works, with a one-line notice (Recommended)":
  `git cli-git ...` keeps working in the new release and prints one line naming the new spelling.
- Package folders, through the question tool: "A: exempt package folders (Recommended)";
  VGW now ends "Package folders are exempt."

## What changes

- `package/git-policy/cli/Cargo.toml`: the `[[bin]]` name (the file is managed by file-enforcer, `CLAUDE.md` WC2:
  edit its source and run `file-enforcer write`).
- The command word in the management grammar (`invocation/management_arguments.rs` and its users), the help texts,
  the service mode the service unit starts (`... service run`), and every test that types the word.
- The old word: accepted, with one line on standard error naming the new spelling, then the same behaviour.
- The crate's tasks (`mise.toml`: `--bin`, `target/release/<name>`), its tests that name the built file
  (`CARGO_BIN_EXE_...`, `release_bins_tests.rs`, launcher fixtures), README, SPEC and the e2e TypeScript.
- Other packages:
  `package/git-executable/core` (`self-shim.ts` lists the names it must never pick as the real Git: add the new name,
  keep the old one for older installs),
  and the references in `mpm-shadow`, `crate-release`, `mutation-test`, `pnpm-shadow` and `dependency-catalog`
  fixtures (each read before changing: some are comments, some are behaviour).
- Workflows: `cargo-publish.yml` (build `--bin`, the archive layout cargo-binstall reads, the `--help` check),
  `npm-release.yml`, `catalog-specifiers.yml`.
- The root `mise.toml` task that runs `cli-git-native cli-git fix ...`.
- `toolchain.plan.jsonc` and `toolchain.lock.json` (the `bins` list and the postinstall `sourceBin` that links the
  program as `git` under `.cache/cli-git-bin`): only after the release that carries the new name is published,
  because the lock must name a file the archive holds.

## The settings file and the data the wrapper keeps

The word `cli-git` also names the settings file (`cli-git.config.jsonc`), data folders (`cli-git/discarded`,
`cli-git/notices`), log files (`cli-git-audit.jsonl`), the link folder `.cache/cli-git-bin`, the service units and the
`cli-git:` prefix of messages: 162 string literals in 65 files.
Told that renaming the settings file or the data folders would leave existing ones behind, the owner answered,
2026-11-04: "rename them."

Proposed handling, following the owner's choice for the command word (the old spelling keeps working with a notice);
open to veto:

- The settings file becomes `monochromatic-git-policy-cli.config.jsonc`.
  In this repository the committed file is renamed in the same change.
  In a repository that still has only the old file, the wrapper reads it and prints one line naming the new name;
  it never renames a committed file by itself.
  When both exist, the new one is read and the line names the old one as ignored.
- The data folders and files the wrapper keeps inside the Git directory (saved copies, notices, logs, records)
  take the new name.
  The first time the wrapper needs one and finds only the old name, it renames it in place (one atomic rename in the
  same folder), so nothing is copied and nothing is left behind.
  When both exist, the new one is used and the old one is left untouched and reported, never deleted
  (owner: "never discard user data").

Asked about the link folder `.cache/cli-git-bin`, the service unit names and the `cli-git:` message prefix,
the owner picked "A: rename them too (Recommended)".
The new units replace the old ones when the old helper is replaced; a helper still running under an old unit keeps
landing commits until then.

## Order

1. One branch: program file, command word with the old word accepted, tests, docs, workflows,
   `git-executable` and the other packages; gated.
2. Release the crate; the archive holds `monochromatic-git-policy-cli`.
3. Lock the release: `toolchain.plan.jsonc`, `toolchain.lock.json`, the root `mise.toml` task; install;
   check that `git` resolves to the new file and that a commit, a push and `git cli-git doctor` (old word, with the
   notice) work, and that the running background helper started by the old program still lands commits.

## Older installs

Worktrees still on an older lock keep `cli-git-native` and the old word; the new release accepts the old word,
and `git-executable` keeps the old name in its list.
A background helper started by an older program keeps running from its versioned install folder until it is replaced.
