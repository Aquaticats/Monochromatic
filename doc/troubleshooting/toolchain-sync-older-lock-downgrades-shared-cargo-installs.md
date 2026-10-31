# A toolchain sync in a worktree with an older lock downgrades the shared `cargo-binstall` installs and breaks git in every checkout

## Symptom

On 2026-10-31 (EDT 2026-10-30 23:54),
every git command in the main checkout began failing through the git wrapper with:

```json
{"schemaVersion":1,"sequence":0,"type":"engine-failure","code":"config-invalid","message":"/var/home/user/Monochromatic/cli-git.config.jsonc: Configuration key policies.security/forbidden-strings[1].rulesFile must be a string, found an array.","hint":"Hint: if you just edited cli-git.config.jsonc, fix the key the message names or revert the edit; an unknown key, a wrong type or an unknown policy id fails before any policy runs."}
```

`git status`,
`git commit`,
and `git reflog` all failed the same way,
for every session working in the main checkout.
The message points at `cli-git.config.jsonc`,
but that file was correct:
commit `ad9fe75d05` had switched `rulesFile` to an array on purpose,
and a commit at 23:52:52 had still gone through.

## Root cause

The configuration was fine;
the wrapper binary had been replaced by an older one.

1.  The git on `PATH` in the main checkout is a link chain into the shared cargo install directory:
    `.cache/cli-git-bin/git -> /home/user/.cargo/bin/cli-git-native`,
    and `.cache/toolchain-bin/cli-git-native` links to the same file.
    Every worktree's links resolve to that one binary.
2.  The toolchain sync installs `cargo-binstall`-route tools with the exact locked version and no install location,
    `package/file-enforcer/core/src/toolchain/providers.cargo.ts:50-55`:

    ```ts
    // package/file-enforcer/core/src/toolchain/providers.cargo.ts
    const args = [
      'binstall',
      '--no-confirm',
      '--version',
      entry.version,
    ];
    ```

    and the Rust port does the same in `package/mpm-shadow/cli/src/toolchain/install_cargo.rs:41`.
    `cargo binstall --help` (cargo-binstall 1.26.1,
    the locked version) states where that goes:
    "By default, binaries are installed to the global location `$CARGO_HOME/bin`".
3.  At 23:53:45 to 23:54:08 the translation session's worktree setup script (`finish-setup.ts`) ran a sync in
    `/var/home/user/worktrees/tr-g245`,
    whose `toolchain.lock.json` pins `monochromatic-git-policy-cli` 0.4.0 (main pins 0.4.4).
    Measured afterwards:
    `/home/user/.cargo/bin/cli-git-native` ctime 23:54:02,
    `cargo-dylint` 23:53:45,
    `dylint-link` 23:53:56,
    `~/.cargo/.crates.toml` rewritten at 23:54:02 and listing
    `monochromatic-git-policy-cli 0.4.0`,
    `cargo-dylint 6.3.0` and `dylint-link 6.3.0`,
    and `tr-g245/.cache/cli-git-bin/git` relinked at 23:54:08.
    The binary's mtime read 2026-10-29 12:45:57,
    the 0.4.0 archive's time,
    which is why mtime alone made the file look untouched;
    ctime showed the replacement.
4.  Wrapper 0.4.0 predates the `rulesFile` array,
    so it rejects main's configuration before any policy runs.

The sync already knows it is replacing a shared install and says so,
`package/file-enforcer/core/src/toolchain/sync.ts:143-144`:

```ts
// package/file-enforcer/core/src/toolchain/sync.ts
    `${id} is installed at ${observed} but this tree's lock pins ${locked}; installing ${locked} replaces it `
      + 'for every tree that shares this install, and a tree whose lock pins the other version replaces it back on its next sync',
```

(the same text in `package/mpm-shadow/cli/src/toolchain/sync.rs:141-142`).
It warns and installs anyway.

### Readings that were wrong

- "The config edit broke git":
  the error names `cli-git.config.jsonc`,
  and another session first read it that way.
  The commit at 23:52:52,
  after the array landed,
  disproved it.
- "My merge at 23:52:15 re-ran a sync":
  the only `post-merge` hook is Git LFS's (`.git/hooks/post-merge`),
  and the replacement happened about 90 seconds after the merge,
  in another worktree.

## Verification

Measured on this host 2026-10-31 with `stat --format='%z ctime | %y mtime | %n'` on the files in step 3,
`rg 'git-policy|dylint' ~/.cargo/.crates.toml`,
and `rg --after-context=4 '"cargo:monochromatic-git-policy-cli"' <tree>/toolchain.lock.json` in both trees.

Works:

- A sync from a tree whose lock matches main's
  (no `isAtLockedVersion` warnings,
  `0 installed`).
- After the repair:
  `~/.cargo/.crates.toml` lists `monochromatic-git-policy-cli 0.4.4`,
  and `git status` in the main checkout succeeds.

Fails:

- A sync from any tree whose lock pins a different version of a `cargo-binstall`-route tool:
  it rewrites `~/.cargo/bin/<binary>` for every tree.
  Breakage is immediate when that tool is the git wrapper and the main checkout's configuration needs the newer one.

## Verified workarounds

### Repair: sync from the main checkout

`mise run toolchain:sync` in the main checkout,
which needs the owner's approval because it writes the real toolchain installs.
On 2026-10-31 it logged six `isAtLockedVersion` warnings and `toolchain sync: 6 installed, 57 unchanged, 0 failed`,
restoring `cargo-dylint` and `dylint-link` 6.3.2,
`monochromatic-git-policy-cli` 0.4.4,
`superhtml` 0.7.1,
`npm:socket` 1.12.2,
and `npm:wrangler` 4.156.0.
Tradeoff:
it pushes main's versions onto every other tree,
including the one that caused the downgrade,
whose old git configuration then runs under the newer wrapper.

### Prevention: refuse a sync whose lock differs from main's

The translation session's setup script now refuses to sync unless the tree's `toolchain.lock.json` matches the main checkout's byte for byte.
Tradeoff:
a tree on an older commit runs without syncing,
so it uses whatever the shared installs hold,
which is main's versions,
not its own lock's.

## What does not work

- Reverting `cli-git.config.jsonc` to a string `rulesFile`:
  it undoes a deliberate change and leaves the shared binary at whatever the last syncing tree pinned.
- Each worktree keeping its own `.cache/cli-git-bin` link:
  the links are per tree,
  but they all point at the one `~/.cargo/bin` file.

## Structural fix (chosen 2026-10-31, before the file-enforcer cutover)

`cargo binstall --help` (1.26.1) offers per-install locations:
`--install-path <PATH>` ("Install prebuilt binaries in a custom location")
and `--root <ROOT>` with `--no-track` for a cargo root without metadata.
Installing each `cargo-binstall`-route tool under a version-keyed directory in the toolchain root
(for example `<toolchain root>/cargo/<crate>/<version>`)
and linking each tree's `.cache/toolchain-bin` to its own lock's version would let trees with different locks coexist,
as the other routes under `/home/user/.local/share/monochromatic-dev/toolchain` already do.
`mpm-shadow` owns the toolchain after the file-enforcer cutover,
so the change belongs in `package/mpm-shadow/cli/src/toolchain/install_cargo.rs`;
the owner chose to make it before the cutover,
and the decision record `doc/decision/file-enforcer-rust-rewrite.md` ("Toolchain (mpm-shadow)") carries it.
Tools this repository publishes,
the git wrapper among them,
get no per-version copies:
the owner ruled "No older repo-maintained tools should ever exist when newer is present.",
so a tree with an older lock runs the newest installed release instead of downgrading it.

## Upstream filing decision

Nothing to file.

1.  Upstream's fault:
    no.
    `cargo-binstall` installs to `$CARGO_HOME/bin` by documented default and offers `--install-path` and `--root`;
    the shared location comes from this repository's call.
2.  to 6.
    Not reached:
    the fix belongs to this repository.

`.out-of-scope/` was not consulted,
because no upstream artifact is drafted.
