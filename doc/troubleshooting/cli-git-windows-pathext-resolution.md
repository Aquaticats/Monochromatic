# Cli-git Windows PATHEXT resolution

## Symptom

The Windows job in hosted run `29168995711` failed before trust tests began:

```text
Error: cli-git: could not find real git binary on PATH.
```

The preceding real filesystem identity step passed,
and GitHub Actions itself used Git during checkout.
Git was therefore installed and reachable by the Windows command shell.

## Environment

The failure occurred on `windows-latest` with Node `26.5.0`.
The same workflow passed on `ubuntu-latest` and `macos-latest`.

## Diagnosis

`package/git-policy/cli/src/resolve-git.ts` split `PATH` correctly with Node's platform delimiter,
but constructed only an extensionless `git` candidate in every directory.

Windows GitHub runners expose Git as an executable such as `git.exe`.
Windows command lookup applies `PATHEXT`,
but direct filesystem probes do not append those extensions automatically.
Cli-git used `fs.access` on the extensionless path,
so every otherwise valid Windows PATH entry appeared absent.

The hosted log identified the exact failing boundary:
`resolveGit` threw at `package/git-policy/cli/src/resolve-git.ts`
before `trust-service.unit.test.ts` created any trust fixture.

## Root cause

The resolver modeled POSIX executable naming while claiming cross-platform PATH lookup.
It accounted for Windows PATH separators through `node:path.delimiter`,
but omitted Windows executable suffix selection through `PATHEXT`.

## Fix

On Windows,
the resolver now expands each PATH directory using `PATHEXT` in declared order.
It probes names such as `git.COM`,
`git.EXE`,
`git.BAT`,
and `git.CMD`,
while retaining existing self-shim byte inspection for every candidate.

Non-Windows platforms continue probing only `git`.
The resolver accepts injected platform and PATHEXT values
so Linux unit tests can reproduce Windows naming
without pretending to execute a Windows binary.

## Verification

A regression fixture creates `git.EXE`,
places an absent `.COM` candidate before `.EXE` through injected PATHEXT,
and proves the resolver selects the executable path.

The package resolver unit test,
type check,
and zero-warning Oxlint pass locally.
Final cross-platform hosted workflow run `29171565815`
passed this user boundary with Git installed on an actual Windows runner.

## Rejected hypotheses

### Git was absent from the runner

Rejected because checkout and post-job cleanup invoked the runner's Windows Git successfully.

### PATH used the wrong separator

Rejected because the resolver already used `node:path.delimiter`,
which is semicolon on Windows.
The missing component was executable suffix expansion.

### Trust registry permissions caused the failure

Rejected because resolution failed before trust-service fixture setup or registry access.

## Upstream status

No upstream report is appropriate.
GitHub Actions,
Node,
and Git exposed expected Windows behavior.
The defect was cli-git's incomplete PATH lookup model.
