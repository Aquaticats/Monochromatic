# pnpm-shadow

Shadows `pnpm`.
Before every pnpm run it checks the `package.json` files of the pnpm workspace that contains the current directory,
warns on standard error about every dependency version that is not `catalog:`, `catalog:<name>` or `workspace:*`,
then runs the real `pnpm` with the arguments untouched and exits with pnpm's own status.
It never blocks pnpm.

Why it exists:
pnpm's `catalogMode: strict` does not reject a literal version such as `"zod": "^4.6.7"` in a manifest,
so such a manifest installs with no warning.
The commit policy `mono/catalog-specifiers` in cli-git and the workflow `.github/workflows/catalog-specifiers.yml` refuse it
at commit time and in CI;
this shadow is the advice a person sees at the moment they run pnpm,
before a commit exists.
All three use the one rule in the `monochromatic-catalog-specifier` crate,
so they cannot disagree about what is allowed.

## Modes

- Started as `pnpm` (a link to the binary, such as one in a directory that comes first on `PATH`),
  it is the shadow described above.
- Started under any other name, such as `pnpm-shadow`,
  it answers only `--help` and `--version`.
  The artifact is deliberately not named `pnpm`,
  so a `target/release/pnpm` or a tool `bin/pnpm` never shadows pnpm by accident.
  This crate does not link itself as `pnpm`;
  linking it is a toolchain-plan change that is not made here.

## What it checks

- The workspace is found like pnpm finds it:
  the nearest directory, starting at the current directory and going up, that holds `pnpm-workspace.yaml`.
  Only the file's presence is tested; the YAML is never read.
  With no such directory the shadow says nothing and pnpm runs.
- The tree under the workspace root is walked with the `ignore` crate,
  which honors `.gitignore`, even when the workspace is not a git checkout.
  `.git` and `node_modules` are never entered,
  dot-directories are,
  and the user's global ignore file is not read, so two people see the same tree.
  Symbolic links are not followed.
- Every `package.json` is read and checked by `monochromatic-catalog-specifier`.
  The rule is in that crate's README:
  `dependencies`, `devDependencies` and `optionalDependencies` accept `catalog:`, `catalog:<name>` or exactly `workspace:*`;
  `peerDependencies` accept `*`, `catalog:` or `catalog:<name>`.
- A tracked `package.json5`, `package.yaml` or `package.yml` is reported by file name and never read.
- A file or directory that cannot be read becomes a note in the same report, never a failure.

## What it prints

One block on standard error, only when there is something to say:
a first line with the count and the workspace root,
one `pnpm-shadow: warning:` line per finding or note,
and a closing line that states the rule and says the shadow is advice only.
Standard output is never written by the shadow,
so pnpm's output is exactly pnpm's.
It warns on every run, including `pnpm --version`, and has no switch to turn that off:
the way to silence it is to fix the manifests it names.

## Exit codes

- pnpm's own exit code, or `128` plus the signal number when pnpm died from a signal.
- `127`: no real `pnpm` found on `PATH`.
- `126`: the real `pnpm` was found but could not be started.

The real pnpm is the first executable named `pnpm` on `PATH` that does not resolve to this binary.

## Limits

- Unix only.
  The build fails on any other platform on purpose;
  a cross-platform version is a later change.
- It checks the tree of its own current directory's workspace.
  It does not parse `--dir` or `-C`, so `pnpm --dir /other` from a different tree checks the wrong tree.
- It reads the worktree, not the index,
  so unstaged edits are checked and a deleted-but-tracked file is not.
  The cli-git policy and the CI workflow are the checks of record.
- Each run walks the whole workspace tree.
  The cost on this repository is in the handover record `doc/handover/catalog-specifier-policy.md`.

## Checks

```bash
mise run //package/pnpm-shadow/cli:lint
mise run //package/pnpm-shadow/cli:lint:clippy
mise run //package/pnpm-shadow/cli:test
```
