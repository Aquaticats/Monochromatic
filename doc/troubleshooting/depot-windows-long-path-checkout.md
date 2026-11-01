# Git for Windows 2.55.0.windows.5 fails a Depot harness checkout at a long fixture path

## Symptom

Depot Windows desktop jobs `117578793896` and `117578794032`
in [run `39094767696`](https://github.com/Aquaticats/Monochromatic/actions/runs/39094767696)
stop in `Checkout frozen trial provenance harness`.
Git for Windows `2.55.0.windows.5` emits `Filename too long`
for deeply nested Git-policy fixture objects and exits with code `1`.
The affected jobs do not execute the desktop consumers.
The unchanged GitHub ARM row checks out the same harness revision successfully.

The complete original logs and artifact digest checks are retained in the private
`depot-native-followup-artifacts/desktop.windows/collection.json` receipt.
The original source case remains unchanged.

## Root cause and verification boundary

The immediate failing operation is Git's working-tree checkout.
The [Git for Windows documentation][long-paths]
states that long-path support is disabled by default
and that `core.longpaths=true` enables support in C-based Git commands.
The prior run did not probe its effective configuration,
so an unset or false setting on those particular runners is not yet a measured fact.
Do not generalize this checkout failure to Windows build or test compatibility.

The [measured checkout paths](../audit/depot-native-checkout-paths-2026-10-18.evidence.json)
explain why the rows differed without assuming different Git configuration:
the same fixture path occupies 260 characters under Depot's long harness directory,
247 under the GitHub ARM workspace,
and 243 under Depot's shorter native harness directory.
The longer Depot checkout fails;
the other original checkouts succeed.

The read-only source clone is Git for Windows release `v2.55.0.windows.5`,
commit `32c4f7689275d233577576630e1ac5b7eb354eb0`.
In `compat/mingw.h:211`,
the long-path wrapper passes the effective configuration to path conversion:

```c
/* git-for-windows/git v2.55.0.windows.5, compat/mingw.h:211 */
static inline int xutftowcs_long_path(wchar_t *wcs, const char *utf)
{
    return xutftowcs_path_ex(wcs, utf, MAX_LONG_PATH, -1, MAX_PATH,
                           are_long_paths_enabled());
}
```

In `compat/mingw.c:4252`,
the long-path handler rejects a path requiring expansion when expansion is disabled:

```c
/* git-for-windows/git v2.55.0.windows.5, compat/mingw.c:4252 */
if (!expand || result >= MAX_LONG_PATH - 6) {
    errno = ENAMETOOLONG;
    return -1;
}
```

This source trace supports the documented opt-in and the measured checkout recovery.
It does not reconstruct the unprobed effective configuration of the first failed runner instances.

## Verified recovery

The recovery harness starts with a Node command before either checkout.
It queries the effective `core.longpaths` value and origin,
enables the option in the disposable runner's user configuration,
and verifies that the effective value is `true`.
The [Git configuration documentation][git-config]
defines the user configuration scope used by `--global`.

The executed configuration change,
inside the disposable Windows runner,
is:

```sh
# Executed by the recovery bootstrap on a disposable CI runner.
git config --global core.longpaths true
```

Its tradeoff is a user-scoped setting for subsequent Git operations on that runner.
It neither changes other tools' path support nor modifies a human workstation's configuration.

Harness revision `20b7e331ad5a4e29bd493aba1829fff5f07d27a8`
preserves every existing consumer step value and every matrix row.
The recovery adds only the bootstrap and changes the workflow name.
Its historical source-case manifest hashes are unchanged.
The first failed allocations remain part of the evaluation cost.

The real recovery is complete in
[run `39095647113`](https://github.com/Aquaticats/Monochromatic/actions/runs/39095647113).
All matrix rows record an unset effective `core.longpaths` value before the bootstrap
and `true` afterward.
Both Depot desktop checkouts then succeed at the same 260-character fixture path.
Their original nextest consumers each pass 1432 executed cases,
with one additional summary skip.
The native binary,
PE inspection,
Bash runtime and launch checks succeed.
The original PowerShell file assertion remains failed;
the independent merged-stream and uncaptured-console probes verify the actual version.
The [consumer evidence](../audit/depot-native-consumers-and-variation-2026-10-18.evidence.json)
keeps those outcomes separate.

The unchanged full checkout and original consumers establish recovery beyond configuration readback.
The declared `depot-windows-2025-4` label is retained.
The original full checkout is retained as the positive control for the reported path failure.

## What does not work

The unchanged long harness checkout failed before any consumer ran.
Shorter successful checkouts alone did not verify recovery at the failing path length.
The recovery therefore retained the original long checkout and original consumers.
Attributing the difference to a Depot-specific Git default was rejected:
all recovery rows,
including GitHub ARM,
reported the setting unset before bootstrap.

## Upstream filing decision

Git's documented opt-in is not evidence of an upstream Git defect.
The documentation also limits its claim to supported Git operations;
other tools still need their own real consumer verification.
No Windows registry or account setting is changed.
The supported configuration and real consumers pass the checkout recovery boundary.
No upstream Git defect is established by this evidence.

- Upstream fault is unestablished:
  the supported configuration recovers the real checkout and consumers.
- A configuration opt-in already exists;
  no additional causal source fix is identified.
- Long paths are supported within the documented operations after opt-in.
- Contribution acceptance was not assessed because no filing is proposed.
- Maintainer intent to change the documented default was not established.
- The consumer configuration patch was verified;
  no upstream patch is required for this recovery.

The `.out-of-scope/` catalog has no Git for Windows exemption.
The tracker search `repo:git-for-windows/git "Filename too long" "core.longpaths"`
returned [issue 3318][different-issue],
whose `git status --ignored` performance regression is a different operation and failure boundary.
There is nothing to add to that thread from this checkout recovery.
No upstream message was sent.

[long-paths]: https://gitforwindows.org/git-cannot-create-a-file-or-directory-with-a-long-path
[git-config]: https://git-scm.com/docs/git-config
[different-issue]: https://github.com/git-for-windows/git/issues/3318
