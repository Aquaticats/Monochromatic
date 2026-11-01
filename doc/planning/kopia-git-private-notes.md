# Planning: kopia-git private notes versioning

Status:
 draft for teammate review.
 Not built.
 Authored 2026-07-13.
 Kopia primitives empirically verified against Kopia 0.23.1 on throwaway repositories.

## Goal

Give scattered,
 private,
 gitignored working notes a tidy,
 durable,
 versioned home
without ever exposing them to a public host and without forcing them out of their
in-tree locations.

The trigger was repository tidiness:
 roughly seven `*.local.md` process notes (merged issue drafts,
 PR reply drafts)
 pile up in the repository root because that is the only place they will not get lost.
They are gitignored (`*.local.*`,
`.gitignore` line 11),
 cannot be published,
 and currently have no home other than root clutter.

## Requirements

The consolidated target,
 after several rounds of refinement:

- Notes are free-form markdown,
   currently gitignored under `*.local.*`.
- The public GitHub repository must never see them and must carry no trace of them
  (no submodule,
   no branch,
   no tracked ignore rule that names them).
- Off-site encrypted backup:
  no cloud host may store note contents as plaintext;
  preferably filenames and commit messages are hidden too.
- Real version history:
  browse past versions,
   diff them,
   restore any of them.
- Delete a spent note from the working area but recover it later,
   never lose it forever.
- Readable plaintext in the working tree (hard requirement):
  notes stay grep-able markdown,
   not ciphertext blobs to decrypt on demand.
- Co-located (hard requirement):
  a note may sit next to the code it annotates
  (`package/foo/design.local.md`),
  not forced into a single collector directory.
- Reuse existing tooling where practical.

Two constraints were relaxed during discussion:
 hiding filenames and commit messages moved from hard to preferred,
 and the literal git history requirement was accepted as "browsable versioned history",
 not git's commit DAG,
 branches,
 or line-level blame.

## Key insight: most of this already exists

The local Kopia setup already satisfies most of the requirement set:

- The Kopia repository uses a filesystem backend (2.2 TB),
  client-side encryption `AES256-GCM-HMAC-SHA256`,
   hash `BLAKE2B-256-128`.
- Kopia encrypts everything client-side,
   including filenames and directory structure;
  the storage backend sees only content-addressed encrypted blobs.
- This repository is already a Kopia snapshot source,
  and `.kopiaignore` does not exclude `*.local.*`,
  so the notes are already encrypted,
   versioned,
   and captured today.
- The filesystem backend auto-syncs to pCloud,
  so off-site durability is already handled and GitHub is not needed at all.

That last point dissolves the hardest part of every git-layer alternative:
 there is no untrusted git host to encrypt against,
 because the notes never travel through GitHub.
pCloud only ever receives Kopia's client-side ciphertext with metadata hidden.

The two genuine gaps are:

- Retention:
  the global policy prunes old snapshots
  (keep 10 latest,
   7 daily,
   4 weekly,
   24 monthly,
   3 annual),
  so "never lose a spent note" is not satisfied yet.
- Interface:
  Kopia's snapshot CLI is not a git-like commit and history experience.

## Chosen approach: kopia-git

Build a thin,
 git-flavored CLI over Kopia snapshots,
 scoped to the private notes,
 with storage,
 encryption,
 and off-site sync left to the existing Kopia to filesystem to pCloud pipeline.

The name `kopia-git` is a working title and is contested:
 there is no git or GitHub involved,
 so the "git" may mislead a future reader.
Alternatives under consideration:
 `kopia-vcs`,
 `notekeep`.

## Verified Kopia behavior

All of the following were tested on isolated throwaway repositories
 (separate `--config-file`,
 temporary paths),
 never against the real Kopia repository.

- Commit message:
  `kopia snapshot create <path> --description="msg"` accepts a free-form description.
  `--tags` and `--pin` also exist.
- Keep forever:
  `--pin=keep-forever` on create,
  or `kopia snapshot pin <id> --add=keep-forever`,
  exempts a snapshot from pruning.
  Confirmed a pinned snapshot survives retention.
- Log:
  `kopia snapshot list <source>` lists snapshots with timestamps,
   IDs,
   and descriptions.
- Diff:
  `kopia diff <id1> <id2>` works but is file-level,
   not line-level
  (it reports "changed `./a.md` size 12 to 19",
   "removed `./b.md`"),
  so a wrapper must restore both versions and run a real line diff for hunks.
- Restore and recover:
  `kopia snapshot restore <id> <dir>` restores a snapshot.
  A file deleted from the working tree,
   then snapshotted,
   was restored from an earlier snapshot.
  Delete-and-recover is proven,
   not assumed.
- Distinct source identity:
  `kopia snapshot create <path> --override-source='notes@host:/monochromatic-notes'`
  records a snapshot under a virtual source identity,
  separate from the whole-repository backup source,
  so the notes get their own history,
   retention,
   and pins with no path collision.

### Negative result: scattered notes cannot use an ignore whitelist

An attempt to keep notes scattered in place by snapshotting the repository root
 with a gitignore-style whitelist
 (`*` to exclude everything,
 `!*/` to re-descend directories,
 `!*.local.md` to re-include notes)
 captured nothing,
 under both rule orderings tried.

Kopia normalizes and reorders ignore rules
 (both orderings displayed identically as `!*.local.md`,
`!*/`,
`*`),
 so gitignore precedence tricks that depend on order do not hold.
The exclude-everything rule wins and the snapshot is empty.
Conclusion:
 do not build on ignore-rule whitelisting.

## Design: scattered notes via invisible staging

Because whitelisting fails but `--override-source` works,
 scattered co-located notes are supported through staging,
 not ignore rules.

- Notes live wherever the author writes them,
  anywhere matching the note pattern.
  They never move.
- On `kopia-git commit`,
  the wrapper gathers notes (`rg --files -g '<pattern>'`),
  hardlinks them into an ephemeral staging tree in a cache directory outside the repository,
  preserving repository-relative paths,
  and snapshots that clean notes-only tree with
  `--override-source='notes@<host>:/monochromatic-notes' --description=<msg> --pin=keep-forever`.
- The staging tree contains only notes,
  so there is no whitelist and no risk of sweeping in `.env.local.json` or code.
- The staging tree rebuilds each commit,
  so a note deleted from the repository is absent from the new snapshot
  but retained in earlier pinned snapshots.
- Restore maps a staged path back to the note's real location in the repository.
- Hardlinks require the same filesystem;
  the wrapper falls back to copy for tiny notes when they differ.

The collector directory still exists,
 but only as an invisible cache detail,
 never as clutter in the working tree.

### Command surface for v1

The requested v1 scope is the full surface:

- `init`:
  gather existing scattered `*.local.md` notes and configure the Kopia source and retention.
- `commit`:
  stage,
   snapshot with description,
   pin keep-forever.
- `log`:
  `kopia snapshot list` for the virtual notes source.
- `diff`:
  file-level `kopia diff`,
   plus a restore-both-and-line-diff path for hunks.
- `restore`:
  restore a snapshot or a single note,
   mapping staged paths back to real locations.
- `pin`:
  explicit pin management.

### Package shape

Per repository conventions:

- A Node CLI package under `package/<name>/cli/`,
  bin with a `#!/usr/bin/env node` shebang.
- `mise.toml` task definitions mirroring a sibling CLI package.
- TSDoc on declarations,
   tests covering each command path,
   and a `README.md`
  before the package is considered complete.
- No bash scripts;
  wrapper logic lives in the package bin.

## Open questions for review

- Name:
  `kopia-git` versus `kopia-vcs` versus `notekeep`.
  The working title implies git,
   which is not involved.
- Note pattern precision:
  `*.local.md` cleanly separates markdown notes from config and secrets at the root today
  (`.env.local.json`,
  `mise.local.toml`,
  `forbidden-strings.append.local.txt` are not markdown),
  but a dedicated suffix would be safer against a future non-note `*.local.md`.
- History granularity:
  Kopia snapshot history is confirmed acceptable,
  but note that `kopia diff` is file-level;
  is the restore-both-and-line-diff path sufficient for review comfort?
- Log source filtering:
  `kopia snapshot list` on an overridden source did not match by default
  and needed `--all`;
  the wrapper must query the virtual source consistently.
- Coexistence:
  the notes source and the existing whole-repository backup source both derive from the same tree;
  staging with `--override-source` keeps them separate,
  but multi-machine use and pCloud sync conflict handling are unexplored.
- Key durability:
  Kopia's repository password and encryption are the single point of total loss;
  confirm the password is backed up out of band.

## Rejected alternatives

Each was ruled out for a concrete reason.
Recorded so the decision is not relitigated.

- Jujutsu (`jj`):
  no encryption of any kind;
  does not run git clean and smudge filters,
  so `git-crypt` under `jj` snapshots plaintext (a data exposure hazard);
  the operation log is not a durable keep-forever store.
  Since `jj` 0.30 it shells out to real `git`,
   so `gcrypt::` remotes do work under it,
  but `jj` adds nothing to this problem.
- `git-crypt`:
  encrypts contents but leaks filenames and commit messages;
  introduces a new key.
- `git-agecrypt`:
  reuses an age key and avoids churn with a blake3 cache,
  but it is effectively unmaintained since 2024,
  has no releases,
  and its own README recommends against using it over sops.
- `transcrypt`:
  deterministic (no churn) and actively maintained,
  but uses a symmetric password (no age reuse) and leaks filenames and commit messages.
- `git-remote-gcrypt`:
  readable working tree,
   hides filenames and messages,
   real git,
   off the shelf;
  but needs a new GPG key and the gcrypt dependency,
  and does not reuse the existing Kopia stack.
- `gocryptfs` reverse mode:
  readable source files,
   hides filenames,
   deterministic;
  but FUSE,
   its own key,
   and the git-over-mount workflow is unverified.
- `sops` as an in-tree solution:
  reuses the existing age key,
  but for prose it is whole-file binary encryption;
  the idiomatic workflow leaves ciphertext in the working tree (fails readable-in-tree),
  and the clean and smudge filter workflow churns because sops re-encrypts non-deterministically.
- Per-file `age` and a plain private companion repository:
  fail the metadata or plaintext constraints.

## Next steps

- Resolve the open questions,
   especially the name and the note pattern.
- Scaffold the package and implement `init`,
  `commit`,
  `log`,
  `restore`,
   then `diff` and `pin`.
- Record the final decision as an entry under `doc/decision/`.

## References

- Kopia:
  [kopia-docs][]
- git-remote-gcrypt:
  [gcrypt][]
- git-crypt:
  [git-crypt][]
- git-agecrypt:
  [git-agecrypt][]
- transcrypt:
  [transcrypt][]
- gocryptfs reverse mode:
  [gocryptfs-reverse][]
- Related planning:
  `doc/planning/kopia-source-watch-package.md`

[kopia-docs]: https://kopia.io/docs/
[gcrypt]: https://github.com/spwhitton/git-remote-gcrypt
[git-crypt]: https://github.com/AGWA/git-crypt
[git-agecrypt]: https://github.com/vlaci/git-agecrypt
[transcrypt]: https://github.com/elasticdog/transcrypt
[gocryptfs-reverse]: https://nuetzlich.net/gocryptfs/reverse_mode_crypto/
