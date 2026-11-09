# `@monochromatic-dev/container-limit-core`

The one lowest disk weight,
and the Podman options that bound a started container or an image build.

Repository rule HXR (`AGENTS.md`) starts every container that can exhaust the host with CPU,
memory,
process and
disk-weight limits.
A container sits in its own cgroup scope outside the unit of whatever launched it,
so the launcher's `IOWeight` never
reaches it;
the options have to be on the container's own command line.
This package holds the weight once,
so a change of it is one edit,
and the tasks,
runners and workflows that start
containers take it from here instead of spelling their own copy.

## Exports

Import the TypeScript source with the `/ts` subpath.
A `mise.toml` script of a package without a `package.json` imports `src/index.ts` by relative path
(`../../container-limit/core/src/index.ts` from a package directory),
which node reads by stripping types;
the file
imports nothing for that reason.

- `LOWEST_DISK_WEIGHT`:
  `10`,
  the lowest weight `podman run --blkio-weight` accepts.
- `diskWeightArguments()`:
  `--blkio-weight=10` and `--cgroup-conf=io.weight=10`.
  The first sets `io.bfq.weight`,
  which the BFQ scheduler reads;
  the second sets `io.weight`,
  which iocost reads.
  Each leaves the other file at the default,
  so both are passed.
- `containerLimitArguments({ memory, cpus, pidsLimit })`:
  the options of a started container (`podman run` or
  `podman create`):
  memory,
  CPUs,
  processes and the weight.
- `imageBuildLimitArguments({ memory, cpus })`:
  the options of `podman build`:
  memory and the CFS period and quota.
  `podman build` has no `--cpus`,
  no `--pids-limit` and no weight option (probed on `podman` 6.1.3 with
  `podman build --help`),
  so an image build cannot be put at the lowest disk weight.

- `DISK_WEIGHT_SWITCH` (`CONTAINER_LIMIT_DISK_WEIGHT`) and `DISK_WEIGHT_OFF` (`off`):
  when the variable has that value
  `diskWeightArguments()` and `containerLimitArguments()` leave the weight out.
  GitHub-hosted runners do not delegate the cgroup `io` controller,
  so a container started with the weight is refused
  there (run 42447009237 of `.github/workflows/music-player-release.yml`,
  2026-11-08:
  `crun: opening file 'io.weight' for writing: No such file or directory`).
  `ide-release.yml`,
  `music-player-release.yml` and `env-local-json.yml` set the variable for their jobs;
  a developer machine never does.

`docker run` has `--blkio-weight` and no `--cgroup-conf`,
so `.github/workflows/pnpr-publish.yml` keeps a literal
`--blkio-weight=10`;
`src/container-start-sites.unit.test.ts` reads it and fails when it differs from the constant.

## Start sites ledger

`src/container-start-sites.unit.test.ts` lists every tracked file that starts a container or builds an image,
and fails
when a file names `podman` or `docker` as a program and is not on the list,
so a new start site cannot skip the limits
unnoticed.
Each listed starter must carry one of the shared helpers or the literal options.

The list is not the whole check:
the test also reads every start the file writes,
because a file that imports a helper and never calls it in a start's own argument list once passed on the helper's
name alone
(`package/terminal/app/mise.toml` before `bbadfac560` had `--memory=4g --cpus=4`,
no process option and no disk weight).
`src/container-starts-in-source-text.ts` finds each start,
`src/argument-text-of-a-container-start.ts` reads the argument text that reaches the program
(an array literal after the program name,
the program name as an array literal first element,
a call that returns the arguments,
or a shell command line with its continuations joined),
`src/defined-text-in-source-text.ts` and `src/imported-module-by-name.ts` follow the names that text uses to where
the arguments are written,
and `src/source-text-without-comments.ts` blanks each language's comments so a mention in one is not a start.

A start whose bound is a size decision the owner has not made is listed by path in the test
(`UNBOUNDED_SITE_DECISIONS`) with the reason,
and the paired test fails when such a path gains a bound,
so an entry leaves the list with its fix.
`doc/handover/container-limits-everywhere.md` carries the decision each one waits on.

## Verification

`mise run //package/container-limit/core:test:unit` runs the tests.
