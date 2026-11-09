# cli-git runs policy code from the trusted snapshot, not from src or dist

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

## Symptom

An edit to git-policy plugin or CLI source has no effect on gate behavior,
even after `mise run //package/git-policy/cli:build` rebuilds
`package/git-policy/cli/dist/final/node/index.mjs`.
The expected bytes are visibly present in both the source file and the rebuilt dist bundle,
yet the commit gate keeps the old behavior.

Concrete incident (2026-07-17,
 #376 landing):
`SCANNER_SELF_MATCH_PATHS` gained `package/forbidden-strings/cli/data/builtin-rules.ported.txt`,
the dist was rebuilt and the change committed and pushed,
but `git commit` and `git cli-git check -- <path>` still flagged the file
(`security/forbidden-strings` finding for pre-existing rule content).
Reading the running code mid-investigation showed identifiers flipping between readable and minified forms,
which was the trusted snapshot bundle being confused for regenerated source.

`package/forbidden-strings/cli/data/builtin-rules.ported.txt` is not in the working tree and no
commit in this repository carries it.

## Mechanism

This mechanism is retired.
The TypeScript wrapper it describes was replaced by the native `monochromatic-git-policy-cli`,
whose last TypeScript commit is `f4fc1fa213ece65672e4314983a75266837892f9`
(`package/git-policy/cli/SPEC.md`, section "What this file specifies");
the native executable reads `monochromatic-git-policy-cli.config.jsonc` as data,
runs no repository-supplied code,
and answers `plugins` and `trust` with an explanation of the retirement
(`package/git-policy/cli/SPEC.md`, configuration validation).
The paragraphs below keep the shape of the removed mechanism.

`cli-git.config.ts` imports plugins from `@monochromatic-dev/git-policy-cli/ts`,
which resolves to TypeScript source.
`git cli-git trust` bundles the config plus its whole import graph into a frozen snapshot at
`~/.local/state/cli-git/trust/v1/records/<volume-key>/path/<base64 of config path>/snapshots/config.mjs`,
and every later invocation executes policy code from that snapshot.
Rebuilding the CLI dist only refreshes the `git` shim wrapper
(`node_modules/.bin/git` points at the dist bundle);
it never refreshes the snapshot the policies run from.

Trust records are keyed by config path,
so every worktree carries its own snapshot;
a fresh worktree reports `config-untrusted` until trusted.

## Diagnosis

The record tree below is the leftover TypeScript registry;
the native wrapper reads none of it,
so this recipe walks dead state and is kept for a machine that still holds the old records.

Decode the record directory names to find the snapshot for a given config path,
then check it for the expected bytes and its build time:

```sh
# any shell; record path segments are base64 of the config path
for d in ~/.local/state/cli-git/trust/v1/records/*/path/*/; do
  basename "$d" | base64 -d
done
grep --quiet '<expected bytes>' <record>/snapshots/config.mjs && echo has-fix
stat --format '%y' <record>/snapshots/config.mjs
```

A snapshot mtime older than the source edit proves the gate never saw the change.

## Remediation

The native wrapper has no equivalent remedy:
it executes only policy code compiled into the executable,
so a rebuilt
`monochromatic-git-policy-cli` is what the gate runs, and
`git cli-git trust --yes` now prints an explanation, changes nothing and exits `0`
(`package/git-policy/cli/SPEC.md`, management commands).
The steps below apply only to the retired TypeScript wrapper.

Run `git cli-git trust --yes` in the affected worktree after changing any code reachable from
`cli-git.config.ts`.
Verify non-vacuously afterwards:
re-run the exact failing invocation
(`git cli-git check -- <path>`)
and confirm a canary at a non-exempt path still produces a finding.

For throwaway worktrees created with `--no-worktree-copy`,
the gate additionally needs the ignored state it skipped:
the scanner binary at `package/forbidden-strings/cli/target/release/forbidden-strings`
and a `node_modules` symlink to the main worktree,
because the trust rebuild resolves tsconfig packages through it.
