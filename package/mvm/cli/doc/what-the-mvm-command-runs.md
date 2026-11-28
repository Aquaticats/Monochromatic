# What the `mvm` command runs

Status:
unapproved and non-final.
A Pi round wrote this on 2026-11-28
and the owner declares completion.
What is done:
the manifest's `bin` names `src/cli.ts`,
the build writes `dist/final/node/cli.mjs` as a launcher for that source,
and `src/the-mvm-command-runs-the-source-beside-it.unit.test.ts` holds it.
What is checked:
that test,
the build,
both lints,
the unit suite,
and real runs of `mvm --help` and `mvm --backend libvirt list`
(section "How it was checked").
What is open:
whether a `mvm` that is ever published should run a built copy again
(section "What a published `mvm` would need").

## The defect this records

Free page reporting landed in `src/domain-xml-builders.ts` on 2026-11-25
(commit `269b812807`,
`doc/decision/mvm-guests-report-free-pages.md`).
Six guests made between 2026-11-27T21:25 and 2026-11-28T01:08
were defined with `<memballoon model='virtio'>` alone,
while two made at 04:53 and 05:16 carried `freePageReporting='on'`
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
"Six of the eight guests could not give memory back").

The `mvm` those rounds ran was `node_modules/.bin/mvm`,
a shim pnpm writes from the manifest's `bin` field,
which named `dist/final/node/cli.mjs`.
Nothing rebuilt that copy when the source moved,
and every worktree inherits the checkout's ignored files,
so a worktree made after the merge still ran a build from before it.
Nothing said so,
and a decision recorded as shipped did not reach a guest for two days
(issue 6655).

## The ways weighed

The workload is one person and many agent rounds at once,
each in a worktree of its own,
each running `mvm` minutes after a merge changed its source.

### The command runs its TypeScript source: chosen

`bin` names `src/cli.ts`,
which is the file `mise run //package/mvm/cli:run` already ran,
and Node strips its types on the way in.
The package is `private: true`,
so no registry install ever carries this manifest.

No build output sits between the command and its code,
so there is nothing to compare,
nothing to refuse,
and no window
in which a merge can leave a run on older code.
It also needs no new code path.

Cost,
measured on 2026-11-28 with both arms alternated in the same rounds
(fifteen rounds of `mvm --help`,
the shim reaching the built bundle against `node src/cli.ts`):
the bundle's median was 135.5 ms (125.2 to 178.4),
the source's 166.0 ms (154.5 to 208.9),
a ratio of 1.225 on the medians
and 1.205 on the round pairs (0.905 to 1.566).
A Node start-up dominates both figures.
`mise run //package/mvm/cli:run` has run `node src/cli.ts` since before this change,
so unbundled execution is this package's existing supported path and not a new one;
what is new is that the shim takes it too.
Once the built entry is the launcher below,
the shim's median is 109.6 ms against the source's 103.9 ms over eleven rounds,
a ratio of 1.055,
so the launcher's extra hop costs about nothing.

### The command compares its build with its source and stops: rejected

Two ways to decide whether a built copy is older.

By change time:
the ignored-state copy a worktree is made with stamps its own times.
In a worktree made on 2026-11-28 at 05:56,
`src/cli.ts` carried 05:51:28 and `dist/final/node/cli.mjs` 05:51:42,
so the copy looked **newer** than the source it was built from,
while its content was the checkout's build of 05:05:04.
A comparison on change times reads the order two copies were written in,
never which code a build holds,
so it is silent exactly when it matters and refuses when nothing is wrong.

By content:
the build would record a digest of its inputs and the command would compare it at start.
That needs a new writer in the build,
an input list that names what the bundler really read,
a digest of the source tree at every command start,
and a refusal for every build that predates the receipt.
It also cannot answer the case it was written for:
the copy that runs first after a merge is the build from before the guard,
so that run stays silent and only later ones refuse.
Every round in a worktree would then have to rebuild before it could act.

### The build writes a launcher for the source: chosen, beside the first

`dist/final/node/cli.mjs` holds no program:
it is one import of `../../../src/cli.ts`.
Nothing executes an older copy through it,
and a `node_modules/.bin/mvm` shim written before the manifest changed
still reaches the source,
because the path it names leads there.

This is what makes the change reach existing worktrees:
pnpm rewrites a shim only on install,
so without the launcher every worktree made before that install
would keep running its old bundle.
With it,
`mise run //package/mvm/cli:build` in a worktree is enough,
and that is the command every round touching this package already runs.

### The worktree launcher builds `mvm` when it makes a worktree: rejected alone

It covers worktree creation and nothing else.
A merge into a worktree after it was made re-stales the copy,
and a person editing this source in the checkout runs a build of it either way.
It also puts a rolldown run in every worktree of every round,
on a machine where many rounds start at once.

## What the repository does now

- `package/mvm/cli/package.json` names `src/cli.ts` in `bin`.
- The build keeps both entries,
  so `cli.d.mts` is still emitted,
  and replaces the CLI entry chunk's code with the launcher
  (`rolldown.node.config.ts`,
  `theCommandRunsTheSourceBesideItPlugin`).
- `pnpm install` in a checkout writes a shim that names `src/cli.ts` directly.
  Until it runs,
  `node_modules/.bin/mvm` reaches the launcher,
  which runs the same source.
  No install ran in the round that wrote this (it is banned in a worktree),
  so the fresh shim itself is unexercised here.
  The repository already carries that shape:
  `package/claude-code-plugin/source` declares
  `"bin": { "spawn-claude": "./src/cli/spawn-claude.ts" }`
  and the installed shim for it ends
  `exec node .../package/claude-code-plugin/source/src/cli/spawn-claude.ts`.
- A published `mvm` would be the exception,
  below.

## What holds it

`src/the-mvm-command-runs-the-source-beside-it.unit.test.ts` has three cases:

- the manifest's `bin` names `src/cli.ts`,
  so no build output stands between the command
  and its code;
- that file starts and answers `--help` when it is run;
- the built entry a shim an older install wrote names imports exactly that file,
  which a bundle written by an older build would not.

The third case refuses two shapes a built entry can wrongly have:
one with no import line at all,
which is a bundle,
and one holding the import line *beside* a program of its own,
which would run that program as well as the source.
Every line other than the shebang,
the comments and the one import must be absent.

Pointing `bin` back at the built copy turned the first and third red,
replacing the launcher with a real bundle turned the third red,
and a file holding the import line plus two lines of program turned it red as well.
The round's report quotes all three.

### The one state this does not repair by itself

The guard holds from the moment a build writes the launcher into a checkout or worktree.
Before that build,
a worktree still holds the old shim *and* the old bundle its `dist` was copied with,
so its `mvm` runs that bundle silently,
which is the state issue 6655 was filed about.
Two things are therefore the coordinating session's,
not this package's:

- after merging a change to this package,
  run `mise run //package/mvm/cli:build` in the main checkout,
  because the ignored `dist` worktrees inherit is the checkout's;
- a round that merges into its own worktree rebuilds there,
  which is what `mise run //package/mvm/cli:build` already does for it.

Nothing in the package can see a build that has not happened yet:
the code that would compare is the code that is stale.

## How it was checked

Recorded by the round that made the change,
each command run once from the worktree's root:

- `mise run //package/mvm/cli:build`
- `mise run //package/mvm/cli:lint:types`
- `mise run //package/mvm/cli:lint:oxlint`
- `node src/the-mvm-command-runs-the-source-beside-it.unit.test.ts`
  from the package folder,
  after the build
- `mise run //package/mvm/cli:test:unit`,
  three times over the round

and the real command,
from the worktree's root:
`./node_modules/.bin/mvm --help` and `./node_modules/.bin/mvm --backend libvirt list`
both answered.

The unit suite's first run ended red in one file this change does not touch:
`src/cli-session-daemon.unit.test.ts` failed
"starts one unit when two processes find no daemon at the same time"
with `expected [] to have a length of 1 but got +0`.
The second run of the same code passed in 27.40 s,
the run after the merge into `main` passed in 27.06 s,
and that file passes alone in 6.6 s.
The case starts two CLIs at once and expects exactly one of them to lose the unit name;
the stand-in `systemd-run` settles the claim with an exclusive create,
`{ flag: \'wx\' }` in the program text it writes
(`src/fixture.fake-systemd-run.ts`),
so a run in which the first CLI has finished before the second reaches the start
has no loser to find,
and how many files run beside it decides how often that happens.
Whether this change makes it more likely is not measured:
separating the two arms needs a campaign of repeated suite runs,
which this round's rules ban,
so a later round should measure it or make the case start its two CLIs in a way that
cannot serialize.

## What a published `mvm` would need

Node refuses TypeScript under `node_modules`,
so a `bin` naming `src/cli.ts` works only where the package is never installed from a
registry,
which is what `private: true` states
(`doc/decision/npm-publishing.md`,
"Published surface is the built artifact;
`./ts` is stripped at publish").

The tarball would not be enough either:
the manifest's `files` is `["dist/final", "LICENSES"]`,
so a `pnpm pack` of this package now carries a launcher whose `../../../src/cli.ts` is not in
the tarball.
No task checks it.
`npm:package:pack` in the root `mise.toml` takes a package name by hand and nothing calls it,
and the changeset release path skips a private package,
so the hole is real but unexercised.
Were `mvm` to be published,
its `bin` would have to name a built copy again,
and the comparison above (a receipt the build writes and the command checks,
refusing when it does not match) would become the only guard that can run there.
That choice is the owner's,
and nothing here makes it.
