# pnpm-shadow

EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE

Shadows `pnpm`.
Before a pnpm command that resolves or changes dependencies,
it checks the `package.json` files of the pnpm workspace that contains the current directory,
warns on standard error about every dependency version that is not `catalog:`,
`catalog:<name>` or `workspace:*`,
then runs the real `pnpm` with the arguments untouched.
Every other command goes straight to pnpm.
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

- Started as `pnpm` (a link to the binary,
  such as one in a directory that comes first on `PATH`),
  it is the shadow described above.
- Started under any other name,
  such as `pnpm-shadow`,
  it answers only `--help` and `--version`.
  The artifact is deliberately not named `pnpm`,
  so a `target/release/pnpm` or a tool `bin/pnpm` never shadows pnpm by accident.
  This crate does not link itself as `pnpm`;
  linking it is a toolchain-plan change that is not made here.

## When it warns

The user's rule is that the warning appears only where it makes sense:
when pnpm is about to resolve or change dependencies from `package.json`.
The command word is the first word that is not an option or an option's value,
looking through one `recursive` word.
These commands warn
(read from `pnpm --help` of pnpm 12.16.1,
aliases included):
`add`,
`install` (`i`),
`install-test` (`it`),
`update` (`up`,
`upgrade`),
`remove` (`rm`,
`uninstall`,
`un`,
`uni`),
`dedupe`,
`import`,
`link` (`ln`),
`unlink` (`dislink`),
and `ci` (`clean-install`,
`ic`,
`install-clean`).

- Any other command,
  such as `run`,
  `list`,
  `root`,
  `exec`,
  `dlx`,
  `test`,
  `fetch` or `prune`,
  does no scan at all.
- A help or version word before a literal `--` (`--help`,
  `-h`,
  `--version`,
  `-v`) cancels the warning:
  `pnpm install --help` installs nothing.
- Everything after a literal `--` belongs to a script and is never read.
- An option that takes a value (`--dir`,
  `-C`,
  `--filter`,
  `-F`,
  `--registry` and the others listed in `src/verb.rs`)
  consumes the next word,
  so `pnpm --filter install list` is a `list`.
  A pnpm option that takes a value and is missing from that list would make the next word look like the command;
  the effect is a missed warning,
  never a wrong one,
  and the fix is one line in `VALUE_OPTIONS`.

## What it checks

- The workspace is found like pnpm finds it:
  the nearest directory,
  starting at the current directory and going up,
  that holds `pnpm-workspace.yaml`.
  Only the file's presence is tested;
  the YAML is never read.
  With no such directory the shadow says nothing and pnpm runs.
- The tree under the workspace root is walked in parallel with the `ignore` crate,
  which honors `.gitignore`,
  even when the workspace is not a git checkout.
  `.git` and `node_modules` are never entered,
  dot-directories are,
  and the user's global ignore file is not read,
  so two people see the same tree.
  Symbolic links are not followed.
- Every `package.json` is read and checked by `monochromatic-catalog-specifier`.
  The rule is in that crate's README:
  `dependencies`,
  `devDependencies` and `optionalDependencies` accept `catalog:`,
  `catalog:<name>` or exactly `workspace:*`;
  `peerDependencies` accept `*`,
  `catalog:` or `catalog:<name>`.
- A tracked `package.json5`,
  `package.yaml` or `package.yml` is reported by file name and never read.
- A file or directory that cannot be read becomes a note in the same report,
  never a failure.

## What it prints

One block on standard error,
only when there is something to say:
a first line with the count and the workspace root,
one `warning:` line per finding or note,
and a closing line that states the rule and says the warning is advice only.
The warning lines never name the wrapper:
the wrapper names itself in exactly two places,
after `pnpm --version` or `pnpm -v` (one line on standard output:
`pnpm-shadow <version>, shadowing pnpm at <path of the real pnpm>`)
and after `pnpm --help`,
`pnpm -h` or `pnpm help` (a blank line and a short section with the same line).
Both appear only when the command line is exactly that one word and pnpm succeeded,
so `pnpm install --help` prints exactly what pnpm printed.
The same two failures name the wrapper because they are the wrapper's own:
no real pnpm found,
and pnpm could not be started.
There is no switch to turn the warning off:
the way to silence it is to fix the manifests it names.

## How it hands over

For every command line except a lone version or help word,
the shadow replaces itself with pnpm (`exec`):
no wrapper process stays resident,
and pnpm's exit status and signals are pnpm's own.
For a lone version or help word it waits for pnpm,
because the notice follows pnpm's text.

## Exit codes

- pnpm's own exit code,
  or the signal that killed pnpm (after `exec`,
  the caller sees pnpm's death itself);
  in the waiting case `128` plus the signal number.
- `127`:
  no real `pnpm` found on `PATH`.
- `126`:
  the real `pnpm` was found but could not be started.

The real pnpm is the first executable named `pnpm` on `PATH` that does not resolve to this binary
and that no earlier shadow instance in the same chain already picked (see "Launchers and the loop guard").

## Never starts a shadow

A shadow never starts a shadow of any version as its real tool,
whatever the path or the version of the candidate
(owner,
2026-11-07:
"We should never invoke ourselves recursively.
That's bad design.").
Every shadow program (`cargo-shadow`,
`mpm-shadow`,
`pnpm-shadow` and `search-shadow`) carries the same identity text in the ELF section `.monochromatic_shadow_identity`
(`src/shadow_identity.rs`,
a byte-identical copy in each crate;
`package/search-shadow/cli/tests/shadow_identity_copies.rs` fails when the copies differ).
The lookup skips a candidate that carries it,
or a launcher whose record leads to one,
without starting it,
and the start checks the open file again.
A shadow that finds `MONOCHROMATIC_SHADOW_STARTED_BY` left for its own process or its direct parent
was started by a shadow:
it does nothing,
prints one line and exits with status 2.
The full description,
the measurement and the cost are in `package/search-shadow/cli/README.md`,
section "Never starts a shadow",
and the incident in `doc/troubleshooting/search-shadow-copies-start-each-other-without-end.md`.

The loop guard below is older and stays:
it can no longer see a second hop,
because the second shadow stops on arrival,
and it still bounds anything the marker cannot see.
A launcher script that leads back to the shadow and has no `.launch` record beside it
used to be run once and skipped by the second instance;
now the second instance stops with exit status 2 and the one line.
The toolchain's own launchers always have a record and are skipped before they are run.

## Launchers and the loop guard

The toolchain links `pnpm` through a launcher:
a separate executable that starts the shadow under the name `pnpm`.
That link resolves to the launcher,
not to the shadow's own executable,
so comparing resolved paths alone cannot tell that the first `pnpm` on `PATH` is the shadow itself.
The first release did exactly that
(found by the toolchain session on 2026-11-01):
it ran its own link as the real pnpm without end,
and on `--version` it started children until the process limit.

- Each instance records the resolved path of the candidate it picks in the environment value `PNPM_SHADOW_GUARD`,
  and the instance that candidate starts skips every recorded path,
  so each hop shrinks the candidate list and the chain ends at the real pnpm.
  A hard limit of eight hops stops anything else with a message.
- The value is bound to a process:
  after `exec` the process id is unchanged,
  so an exec guard counts only for the process with that id;
  a spawn guard counts only for the direct child of the process that wrote it.
  A script that the real pnpm runs,
  and that calls `pnpm` again,
  therefore never inherits it and finds the real pnpm.
- An instance started by another instance does not warn again.
- For `--version` and `--help` through a launcher,
  only the innermost instance knows which pnpm really ran,
  so it prints the notice and leaves a marker file named for the outer instance's process id in `XDG_RUNTIME_DIR`;
  the outer instance claims the marker after pnpm finishes and stays silent.
  Without `XDG_RUNTIME_DIR` there is no handshake and a launcher chain prints the notice twice,
  the outer one naming the launcher link.

## Cost

Measured on 2026-11-01 on this repository (22,508 walked entries,
257 `package.json`),
pnpm 12.16.1,
100 interleaved pairs per command (the real pnpm and the shadow back to back,
order alternating),
median of the paired difference:
`pnpm --version` 0.85 ms over 6.6 ms,
`pnpm root` 0.59 ms over 11.8 ms,
`pnpm install --help` 0.78 ms over 6.6 ms.
`/usr/bin/true` takes about 0.5 ms to start,
so the wrapper's fixed cost is near the cost of starting any process.
A command that warns pays for the walk:
the parallel walk's median was 10.0 ms against 78.9 ms for the serial walk (30 alternating rounds).
The numbers and their spread are in `doc/handover/catalog-specifier-policy.md`.

## Limits

- Unix only.
  The build fails on any other platform on purpose;
  a cross-platform version is a later change.
- It checks the tree of its own current directory's workspace.
  It does not parse the value of `--dir` or `-C` as a directory,
  so `pnpm --dir /other install` from a different tree checks the wrong tree.
- It reads the worktree,
  not the index,
  so unstaged edits are checked and a deleted-but-tracked file is not.
  The cli-git policy and the CI workflow are the checks of record.

## Checks

```bash
mise run //package/pnpm-shadow/cli:lint
mise run //package/pnpm-shadow/cli:lint:clippy
mise run //package/pnpm-shadow/cli:test
```
