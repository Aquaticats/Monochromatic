# cli-git runs policy code from the trusted snapshot, not from src or dist

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

## Mechanism

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
