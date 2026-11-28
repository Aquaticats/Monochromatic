# What the `mvm` command runs

Status: unapproved and non-final.
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
(commit `269b812807`, `doc/decision/mvm-guests-report-free-pages.md`).
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
so there is nothing to compare, nothing to refuse, and no window
in which a merge can leave a run on older code.
It also needs no new code path.

Cost, measured on 2026-11-28 with both arms alternated in the same rounds
(fifteen rounds of `mvm --help`,
the shim reaching the built bundle against `node src/cli.ts`):
the bundle's median was 135.5 ms (125.2 to 178.4),
the source's 166.0 ms (154.5 to 208.9),
a ratio of 1.225 on the medians
and 1.205 on the round pairs (0.905 to 1.566).
A Node start-up dominates both figures.
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
That needs a new writer in the build, an input list that names what the bundler really read,
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
With it, `mise run //package/mvm/cli:build` in a worktree is enough,
and that is the command every round touching this package already runs.

### The worktree launcher builds `mvm` when it makes a worktree: rejected alone

It covers worktree creation and nothing else.
A merge into a worktree after it was made re-stales the copy,
and a person editing this source in the checkout runs a build of it either way.
It also puts a rolldown run in every worktree of every round,
on a machine where many rounds start at once.

## What the repository does now

- `package/mvm/cli/package.json` names `src/cli.ts` in `bin`.
- The build keeps both entries, so `cli.d.mts` is still emitted,
  and replaces the CLI entry chunk's code with the launcher
  (`rolldown.node.config.ts`, `theCommandRunsTheSourceBesideItPlugin`).
- `pnpm install` in a checkout writes a shim that names `src/cli.ts` directly.
  Until it runs, `node_modules/.bin/mvm` reaches the launcher, which runs the same source.
- A published `mvm` would be the exception, below.

## What holds it

`src/the-mvm-command-runs-the-source-beside-it.unit.test.ts` has three cases:

- the manifest's `bin` names `src/cli.ts`, so no build output stands between the command
  and its code;
- that file starts and answers `--help` when it is run;
- the built entry a shim an older install wrote names imports exactly that file,
  which a bundle written by an older build would not.

Pointing `bin` back at the built copy turns the first and third red
(`Control's red` in the round's report).
Replacing the launcher with a bundle turns the third red.

## How it was checked

Recorded by the round that made the change,
each command run once from the worktree's root:

- `mise run //package/mvm/cli:build`
- `mise run //package/mvm/cli:lint:types`
- `mise run //package/mvm/cli:lint:oxlint`
- `node src/the-mvm-command-runs-the-source-beside-it.unit.test.ts`
  from the package folder, after the build
- `mise run //package/mvm/cli:test:unit`

and the real command, from the worktree's root:
`./node_modules/.bin/mvm --help` and `./node_modules/.bin/mvm --backend libvirt list`
both answered.

## What a published `mvm` would need

Node refuses TypeScript under `node_modules`,
so a `bin` naming `src/cli.ts` works only where the package is never installed from a
registry, which is what `private: true` states
(`doc/decision/npm-publishing.md`, "Published surface is the built artifact;
`./ts` is stripped at publish").
Were `mvm` to be published,
its `bin` would have to name a built copy again,
and the comparison above (a receipt the build writes and the command checks, refusing when
it does not match) would become the only guard that can run there.
That choice is the owner's, and nothing here makes it.
