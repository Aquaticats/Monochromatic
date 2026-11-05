# Renaming the Git wrapper program and its command word

Status: step 1 of "Order" is built on the branch `feat/git-policy-cli-program-rename`, unmerged and unreleased
(see "Step 1 as built"); steps 2 and 3 wait for the owner.
The decisions below are the owner's and are unchanged.

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

## Step 1 as built

Recorded 2026-11-04T22:36:12-05:00 on the branch `feat/git-policy-cli-program-rename`; the sections above stay as decided.

### Mechanism

- One pair per renamed name lives in `package/git-policy/cli/src/native/foundation/wrapper_names_old_and_new.rs`
  (program, command word, settings file, message prefix, data folder, four logs, merge records, ignored-state copy records,
  service unit); no other file spells a pair.
- Where the wrapper first learns the Git directories of a command (`GitFacts` in `repo_read/repository_facts.rs`, once per
  process), `foundation/kept_names_in_the_git_directories.rs` moves each earlier name to the current one with
  `renameat2(RENAME_NOREPLACE)` (`foundation/earlier_name_moved_to_current_in_place.rs`): no lock, nothing copied, a lost race
  or a crash between the look and the rename leaves a state the next run finishes.
  The audit log's numbered older copies move with it.
  Readers that visit another worktree's Git directory (`path_for_reading`) never move anything.
- The settings file: `LoadedConfig.earlier_name_notice`, said once per process on standard error.
  An invalid earlier file is an error that names its path, without the notice.
- The earlier command word: `EARLIER_COMMAND_WORD_NOTICE` in front of the same action; `EARLIER_SERVICE_WORDS` starts the
  service mode too, so a unit file an older program wrote still starts a helper.

### Where the plan was wrong or incomplete

- The plan's units section says a running old helper "keeps landing commits until replaced".
  A new wrapper never connects to it: the new socket is `$XDG_RUNTIME_DIR/monochromatic-git-policy-cli/<instance>.sock`.
  Both helpers can exist for one worktree until the old one idles out; nothing stops or removes the old units or the old
  template unit files (older installs in other worktrees need them), and the new journal folder
  `monochromatic-git-policy-cli/service/<instance>` under the state directory starts empty.
- A socket path holds 107 bytes, and the program folder name is 21 bytes longer than `cli-git`:
  a runtime directory longer than 41 bytes no longer fits, so the service fixtures and the e2e scenarios use shorter runtime
  directory names.
- After the settings file is renamed in the repository (the last commit of the branch), a worktree still on the old lock runs
  the old wrapper, which reads only `cli-git.config.jsonc`: it sees no configuration and runs no policy until the lock moves.
  The `catalog-specifiers` workflow copes by choosing the program, word and settings name of the pinned release.
- The link folder `.cache/cli-git-bin` is hard-coded in `package/mpm-shadow/cli/src/toolchain/scope.rs` and in
  `.config/mise/conf.d/20-env.toml`, so its rename belongs to step 3 with the lock.

### Not renamed, on purpose

`--cli-git-keep-going`; the Git configuration keys `hook.cli-git-lfs-pre-push.enabled` and `user.cli-git-test`;
the references `refs/cli-git/kept/`, `refs/cli-git/trash`, `refs/cli-git/auto-push/` and `refs/worktree/cli-git-service/`
(references are Git's retention state; a rename would need packed-references handling);
the temporary file and folder prefixes `.cli-git-*` and `cli-git-*` (including `.cli-git-worktree-copy-`, whose names the
copy records hold), the index lock marker `cli-git-service` (a reader of an older marker must keep working), the conversion and
migration journals, `.claude/cli-git-trash`, `cli-git-transactions` (an obsolete directory the startup step only reports),
thread names, the journal field `CLI_GIT_AT_MS`, the test variables `CLI_GIT_TEST_ONLY_*`, and the mutation and e2e tool
prefixes `cli-git-mutpar` and `localhost/cli-git-concurrent-e2e`.

### Edits that wait for the release (step 3)

- `toolchain.plan.jsonc` and `toolchain.lock.json`, entry `cargo:monochromatic-git-policy-cli`:
  `bins` becomes `["monochromatic-git-policy-cli"]`, `options.postinstall.link.sourceBin` the same,
  and the comment above the entry names the new program.
  `package/git-policy/cli/bin/native-release-bins.unit.test.ts` stops accepting the earlier executable from then on
  (remove `earlierExecutables`).
- The root `mise.toml` task `changeset:version`: `monochromatic-git-policy-cli monochromatic-git-policy-cli fix --all --policy
  mono/dependent-version-bump`, and its comment; then `npm-release.yml` drops the `cli-git-native` link it makes.
- With the link folder (owner: rename it): `.cache/cli-git-bin` becomes `.cache/monochromatic-git-policy-cli-bin` in the plan,
  the lock, `.config/mise/conf.d/20-env.toml`, `package/mpm-shadow/cli/src/toolchain/scope.rs` and
  `old_copy_places.rs` with their tests, and the READMEs that name it.
