# Cli-git push fails after hk retirement because config-based hook remains

## Symptom

The first push after commit `122c5dbd2` removed hk/Pkl provisioning failed with native Git's configured pre-push
command:

```text
test "${HK:-1}" = "0" || hk run pre-push --from-hook "$@": line 1: hk: command not found
error: failed to push some refs to 'https://github.com/Aquaticat/Monochromatic'
```

Cli-git completed its policy gate;
native Git then loaded a stale `hook.hk-pre-push.command` from `.git/config` and could not resolve the retired `hk`
executable.
The remote was unchanged.

## Environment

The checkout used Git 2.54 config-based hooks.
`git config --show-origin --get-regexp '(^|\.)hk([.-]|$)|hooksPath'` reported local
`hook.hk-pre-commit.*` and `hook.hk-pre-push.*` keys.
`.git/hooks/pre-push` remained the unrelated Git LFS hook and contained no hk command.
No global hk key was present on this machine.

## Root cause

Hk 1.50.0's installer writes `hook.hk-<event>.command` and `hook.hk-<event>.event` into local or global Git
configuration.
Those registrations are user or checkout state outside the tracked repository.
Deleting `hk.pkl` and removing the mise tool declaration therefore cannot remove them automatically.
Git correctly continued invoking the configured command after the executable disappeared.

The relevant upstream source is `jdx/hk` tag `v1.50.0`,
`src/cli/install.rs` function `remove_config_entries`.
It enumerates `^hook\.hk-`,
deduplicates names,
and runs `git config <scope> --unset-all <key>`.
The repository-owned migration mirrors that exact ownership prefix without depending on the retired binary.

## Fix

Remove the registrations by hand, in each Git configuration scope that holds them.
List them with their scope and origin:

```sh
# Run in the affected checkout; nothing is changed by this command.
git config --show-scope --show-origin --get-regexp '^hook\.hk-'
```

Exit code 1 with no output means the machine has no retired registration.
Then remove each listed key from the scope the listing names:

```sh
# Use --local for a "local" line and --global for a "global" line.
# <key> is the key as printed, for example hook.hk-pre-push.command.
git config --local --unset-all <key>
git config --global --unset-all <key>
```

Remove only keys that begin with `hook.hk-`, the prefix that hk's own `remove_config_entries` owns.
Do not delete unrelated `hook.*` entries or files under `.git/hooks`.
Run the listing again afterwards: it must print nothing.

A repository-owned task (`cleanup:hk-git-config`), its disposable-fixture test and a runbook automated this removal.
They were retired on 2026-10-13 with the TypeScript wrapper they ran on (user decision recorded in
`doc/planning/cli-git-native-cutover.md`), because the registrations are one-time user state and the manual commands
above are the complete procedure.

## Verified bridge

Issue `#357` forbids automated verification from mutating real per-user Git state.
The failed checkpoint push was therefore retried as:

```sh
git push --no-verify
```

Native Git skipped the stale hook,
but cli-git still ran its manual-push policies because `--no-verify` is not a cli-git policy bypass.
The push succeeded.
This bridge is suitable only until the owning user runs the explicit cleanup.

The retired cleanup task had been verified against disposable repositories and a disposable global configuration
(`GIT_CONFIG_GLOBAL`) for absent, configured, repeated and unrelated-key cases;
that verification no longer exists.

## Rejected hypotheses

### Cli-git skipped policy enforcement

Rejected because cli-git settled its manual-push lifecycle before native Git attempted the stale hook command.
The retry used native `--no-verify`,
which cli-git forwards without treating it as a policy escape.

### A legacy hk script remained under `.git/hooks`

Rejected by reading the actual pre-push file.
It is the Git LFS hook;
the hk invocation came from `.git/config` as Git's diagnostic showed.

### Removing the mise tool should remove Git configuration

Rejected because mise owns executable provisioning and the repository lockfile,
not per-user or per-checkout Git configuration.
Removing those declarations cannot safely infer which external Git scopes the user intended to mutate.

## Upstream status

No upstream report is appropriate.
Hk documents `hk uninstall` and `hk uninstall --global` for this state.
The migration failure was an expected lifecycle boundary when repository-managed tool removal preceded explicit
user-owned config cleanup.
