# meow's per-user configuration, its layering, and its trust model

## Status

Accepted 2026-09-17,
after the brief on
[`doc/planning/monorepo-manager-route-research/per-user-config.md`](../planning/monorepo-manager-route-research/per-user-config.md).
The user chose "Repo proposes,
you accept" for outside writes,
"Go with the logic in our cli-git" for trust,
"Yes,
in a separate non-shadowing kind" for per-user tasks,
and "Switch,
default on everywhere" for automated runs.
Every decision here applies to meow 0.x only (user,
 2026-09-17).
Design:
"Per-user configuration" in
[`doc/planning/monorepo-manager-from-scratch-design.md`](../planning/monorepo-manager-from-scratch-design.md).

## Context

- Rules that write outside a repository,
   such as the JetBrains Harper settings in `file-enforcer.config.ts`,
   moved to a per-user configuration on 2026-09-17.
- meow serves repositories other than Monochromatic,
   so an unseen repository is untrusted input.
- Measured on this machine:
   `XDG_CONFIG_HOME` is unset,
   two of the three `XDG_CONFIG_DIRS` entries are owned by root,
   `/home` is a symlink to `var/home`,
   and a file watch dies after one temp-plus-rename save while a directory watch survives.
- cli-git already owns a trust model in this repository:
   [`doc/decision/cli-git-policies-platform.md`](cli-git-policies-platform.md)
   and `package/git-policy/cli/src/allowed-worktree-dirs.ts`.
   Note 2026-11-10T18:32:51-05:00: that file is deleted (2026-10-13 `642b6d72cb`).
   The allowlist it held is `default_allowed_worktree_dirs` in
   `package/git-policy/cli/src/native/invocation/effective_target.rs`,
   and the `trust`, `untrust` and `status` commands this decision adopts are retired
   (`package/git-policy/cli/src/native/invocation/management.rs:93`).

## Decision

- Discovery,
   in order:
   `--user-config <path>`,
   `MEOW_CONFIG`,
   `$XDG_CONFIG_HOME/meow/meow.hcl` when absolute,
   then `$HOME/.config/meow/meow.hcl`.
  No `XDG_CONFIG_DIRS` search list,
   because two of this machine's three entries are root-owned
   and a trust-bearing file must be one the user authors.
- `--no-user-config` and `MEOW_NO_USER_CONFIG` skip the per-user file and record an `Absent` read.
  The per-user file is read everywhere else,
   with no CI detection.
- Layering is per-attribute declared scope:
   each schema attribute is `user_only`,
   `repository_only`,
   or `both` with a stated winner,
   and a value in a layer that may not set it is ignored with a JSON diagnostic.
- Writes outside a repository are proposals:
   the repository declares content and no outside destination,
   and the per-user file supplies destination and consent,
   matchable by path prefix.
  An unmatched proposal is reported rather than skipped.
- The per-user file may define tasks in a kind that cannot shadow a repository task name
   and does not join a repository's task graph or cache keys.
- Trust follows cli-git:
   explicit `trust`,
   `trust --yes`,
   `untrust`,
   and `status`;
   an untrusted or changed configuration blocks every configuration-loading command with no built-in fallback;
   trust identity is the filesystem ID paired with the canonical configuration path,
   encoded reversibly rather than hashed;
   comparison is exact bytes and evaluation runs the stored snapshot;
   the registry lives under the operating-system account home rather than an environment-derived path;
   and trusted configuration is not sandboxed.
- Cache keys are two-level:
   the whole per-user file digest keys configuration evaluation,
   while task entries carry only the per-user values that task consumed.
- One daemon per canonical repository root,
   socket under `$XDG_RUNTIME_DIR`;
   reload carries a generation number and pins in-flight tasks;
   a malformed per-user file keeps the last good snapshot and reports a span.
- The per-user configuration's containing directory is watched,
   never the file inode.

## Consequences

- Listing a repository's task graph loads configuration,
   so it blocks until that configuration is trusted.
  This is stricter than the restricted mode the research proposed.
- Trust is the one place meow does not use its XXH3-128 cache key hash,
   because cli-git's model compares exact bytes.
- The trust registry's location does not follow XDG,
   while per-user configuration discovery does;
   the difference is deliberate,
   because environment variables are repository-influenced and a trust record must not be.
- The recorded sentence that the cache key includes both configuration digests holds for the evaluation key,
   not for a task entry's key.
- meow never writes to the user's HCL,
   so their formatting is untouched,
   which matters because meow ships no formatter.

## Rejected

- Copying OpenTofu's dotfile-first discovery:
   measured,
   it never reaches `$HOME/.config/opentofu/tofurc` when `XDG_CONFIG_HOME` is unset,
   which is this machine's state.
- A `conf.d` fragment directory:
   it reopens a precedence question inside one author's own files,
   where OpenTofu's own merge admits the order "was unfortunately never well specified".
- Whole-file replacement,
   per-key layering in either direction,
   and refusing overlaps:
   per-attribute scope expresses the Harper case,
   where the repository owns the content and only the destination is user-scope.
- Trust on first use,
   a prefix whitelist alone,
   and the restricted mode from the research:
   the user chose cli-git's explicit-trust model instead.
- Automatic CI detection:
   it decides a build's inputs from an inferred environment rather than something written down.
- Per-user tasks in the same kinds as repository tasks:
   a file outside version control would change a repository build's result.
