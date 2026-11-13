# `@optique/core` 1.0.0 to `main`: the input trace copies its whole map for each token it records, so a successful parse takes time close to the square of the positional token count

Unapproved and not final.
An AI session wrote this document on 2026-11-13,
and the owner has not approved it.
Done and checked by that session:
the measurements and the source reading reported here.
Open:
a model from another vendor reviewed the text as of commits `5a4e07d2e3` and `426d33d638`,
and the corrections made after that as of commit `2d06b4ae60`,
and its findings were folded in;
the corrections made after that last pass are not reviewed.
Fixes for the two further copies named under "What is left after the fix" were prototyped by an AI agent
and rechecked by this session on 2026-11-13,
as recorded under "Prototypes for the two further copies":
the one for `multiple()`'s state holds up and is kept as a second patch,
and the one for the token buffer slows other input and is not kept.
A model from another vendor reviewed that section and the second patch as of commit `758946f3e1`,
and its findings were folded in.
What changed after that is not reviewed:
the fix it asked for,
the timing on Node and Bun,
and the timing rewritten as ratios,
first in that section and then,
on the owner's choice of 2026-11-13,
through the whole document,
where absolute times now stand only as scale.
No person has reviewed any of it.

Status,
2026-11-13:
present in every release from 1.0.0 and on upstream `main`.
Upstream's maintainer knows of it:
upstream [issue 1120][scoped out],
opened that day about another cost,
says of this one that it
"contributes to large-input costs,
but fixing that pre-existing behavior is outside this issue's scope."
No upstream issue is about it.

No package in this repository is known to pass enough positional tokens to an Optique parser to notice;
which consumers do was not checked.
It is written up because it was found while answering upstream's question about parsing cost,
and because a fix was prototyped and checked.

The measurement that first showed it is in `doc/research/optique-typo-suggestion-overhead-upstream-record.md`,
section "A cost that grows with the square of the token count".

## Symptom

Nothing fails.
A parse that succeeds takes longer than the token count explains.
With one option parser and a run of positional tokens,
on 1.3.6 and Node 26.13.1,
each doubling of the tokens from 500 to 8,000 multiplied the median time of five parses,
within one run,
by 3.2,
3.4,
3.7 and 4.3,
where a cost proportional to the tokens would give 2 each time and their square 4.
Sixteen times the tokens cost 173 times as long.
For scale only,
that run took 7.7 ms at 500 tokens and 1.3 s at 8,000.
A straight-line fit of the logarithms gives an exponent of 1.85,
where 1 is linear and 2 is quadratic.

At the sizes most command lines have,
the cost is too small to see.
It shows once other costs are small:
with few option parsers,
or on builds where upstream pull request 997 has removed the suggestion work.

## Root cause

### Step 1: each recorded input goes through `recordTrace()`

`option()` and `argument()` record the raw input of each value they accept,
so that a parser which depends on another's value can be replayed.
They do it through one helper,
[`packages/core/src/primitives.ts` lines 162 to 176][record trace]:

```ts
// packages/core/src/primitives.ts at commit 1d6e4594 on main
function recordTrace<TState>(
  context: ParserContext<TState>,
  entry: TraceEntry,
): ParserContext<TState> {
  if (context.exec?.trace == null) return context;
  const trace = context.exec.trace.set(context.exec.path, entry);
  return {
    ...context,
    trace,
    exec: {
      ...context.exec,
      trace,
    },
  };
}
```

The file calls it at six places,
lines 1231,
1258,
1344,
1371,
2711 and 2738 at that commit.

### Step 2: `set()` copies every entry already stored

[`packages/core/src/input-trace.ts` lines 140 to 144][trace set]:

```ts
// packages/core/src/input-trace.ts at commit 1d6e4594 on main
  set(path: readonly PropertyKey[], entry: TraceEntry): InputTrace {
    const copy = new Map(this.#entries);
    copy.set(serializePath(path), entry);
    return new InputTraceImpl(copy);
  }
```

The trace has to be immutable:
a failed alternative discards its context,
and the parse goes on from a trace taken earlier,
which must not show the discarded writes.
The copy is how that is kept.

### Step 3: N recorded tokens copy 1 + 2 + ... + N entries

`multiple(argument(...))` records one entry per positional token,
each under its own path,
so the map grows by one entry per token and each `set()` copies all of them.
Recording N tokens copies N(N + 1) / 2 entries.

### Where it came from

Commit `b2137fed`,
"Add InputTrace for recording raw inputs during parsing",
2026-03-28,
part of upstream issue 752 and pull request 760.
`git tag --contains b2137fed` lists 1.0.0 first.
`set()` is unchanged since:
the last commit on the file,
`aefcb845` on 2026-05-14,
changed comments.
Issue 752 does not discuss how the trace stores its entries.

### What the profile attributes

On 1.3.6 with a thousand paths after `--`,
through a parser with the option set of `git commit`,
`node --cpu-prof` put 51.6% of the parse in `input-trace.js`,
and 78.6% at four thousand paths
(research document,
same section).

### What is left after the fix: two more copies per token

With the prototype under "Upstream filing artifact",
the exponent over 500 to 8,000 tokens falls to between 1.18 and 1.27,
not to 1,
and above that it rises again.
On the patched build on Node 26.13.1,
each doubling from 2,000 to 16,000 tokens multiplied the median of five parses by 2.2,
2.9 and 5.4,
and in a second run the step from 16,000 to 32,000 multiplied it by 6.9.

Two more places copy an array that grows with the token count,
once per token,
and a profile of the patched build names both.

`multiple()` rebuilds its whole state array for each item it accepts,
[`packages/core/src/modifiers.ts` lines 2743 to 2746][multiple state]:

```ts
// packages/core/src/modifiers.ts at commit 1d6e4594 on main
        state: annotateFreshArray(context.state, [
          ...(added ? context.state : context.state.slice(0, -1)),
          nextItemState,
        ]),
```

`argument()` copies the rest of the token buffer for each token it consumes,
[`packages/core/src/primitives.ts` line 2725][argument buffer]:

```ts
// packages/core/src/primitives.ts at commit 1d6e4594 on main
              buffer: context.buffer.slice(i + 1),
```

`node --cpu-prof` on the patched build,
self time under the parse:

- Parses of 4,000 and 8,000 tokens together:
   `modifiers.js` 38.9%,
   `short-option.js` 17.7%,
   `primitives.js` 15.8%,
   the new trie 4.7%,
   `input-trace.js` 1.8%.
- Parses of 16,000 and 32,000 tokens together:
   `multiple()`'s `parseSync` 71.2%,
   the function in `primitives.js` that holds the `argument()` excerpt 19.1%,
   the trie 0.9%.

In a profile of the unpatched build at the same sizes,
`set` in `input-trace.js` is 92.7% of the parse.
So the trace copy is by far the largest of the three at every size measured,
the prototype removes that one only,
and what is left at the larger sizes is mostly the other two,
nine tenths of the patched parse at 16,000 to 32,000 tokens.
At 4,000 to 8,000 the same two functions are 14.6% and 13.3% of the patched parse,
27.9% together,
and the two files they sit in are 54.7%;
the profile does not separate the copies from the other work in those functions,
so it gives no closer figure for the copies' own share at those sizes.
Run against the unpatched build in the same rounds,
five of them with the order rotated,
the patched build took 0.12 times as long at 16,000 tokens on Node (0.11 to 0.13)
and 0.25 times on Bun (0.20 to 0.32);
on Deno the unpatched one took 6.0 times as long at 16,000 (5.4 to 6.9) and 4.4 times at 32,000 (4.0 to 5.2).
The prototype's author,
an AI agent,
had attributed the remainder to the buffer copy alone;
the profile puts more of it in `multiple()`.

The buffer copy is not `argument()`'s alone.
On `main` at `d26f1a24`,
sixteen places in `primitives.ts` build the next buffer with `context.buffer.slice(...)`,
[the option parser's among them][option buffer],
so every primitive that consumes a token pays it;
`argument()` is the one a run of positional tokens goes through.

On 2026-11-13 the owner chose to have fixes for both prototyped before any reply to upstream.
The result is under "Prototypes for the two further copies":
the fix for the state of `multiple()` holds up,
and the one tried for the buffer does not.
Each changes a value that other code reads as a plain array
(the state of `multiple()`,
and the context's token buffer),
so a fix reaches further than the trace's did,
and for the buffer that reach is what defeated the attempt.

### A figure that turned out too narrow

The reproduction repository's README first gave the exponent as "1.75 to 1.97",
from one script run on one machine.
Across the runs since,
it ranges from 1.56 to 1.92,
and it depends on the runtime:
the figures are under "Verification".

## Verification

### Versions under test

`@optique/core` 1.3.6 and `1.4.0-dev.2971` from npm,
each installed from a tarball whose SHA-256 was checked
(`6a24982a…` and `fbfa4083…`,
the values the reproduction repository's workflow pins),
and upstream source at commit `1d6e459442e92f83006a7c34e4f11f5018c9ab39`,
on Node 26.13.1,
Deno 2.9.8 and Bun 1.4.4,
in a container from `docker.io/cachyos/cachyos-v4:latest` run with
`--network none --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10` and no mounts,
on 2026-11-13.
1.1.1 was measured on 2026-11-12 with an earlier script (research document).
The newest release of each older line,
1.0.15,
1.1.13 and 1.2.13,
resolved from the registry on 2026-11-13,
was run the same day on Node beside 1.3.6.

### Harness

`token-scaling.ts` in the reproduction repository,
`Aquaticat/optique-parse-cost-reproduction`
(public since 2026-11-13T14:41:43Z;
commit `76b15277df53e5577cdc4e67bfeda3b7302b12c6`).
Its parser and timed region,
which are all that matter here:

```ts
// token-scaling.ts, the parser and one timed parse
import { object } from "@optique/core/constructs";
import { multiple, optional } from "@optique/core/modifiers";
import { parseSync } from "@optique/core/parser";
import { argument, option } from "@optique/core/primitives";
import { string } from "@optique/core/valueparser";

const parser = object({
  name: optional(option("--name", string())),
  paths: multiple(argument(string())),
});

for (const tokens of [500, 1000, 2000, 4000, 8000]) {
  const input = Array.from({ length: tokens }, (_, index) => `p${index.toString(36)}`);
  parseSync(parser, input);
  const startedAt = performance.now();
  const result = parseSync(parser, input);
  const elapsed = performance.now() - startedAt;
  if (!result.success || result.value.paths.length !== tokens) throw new Error("wrong parse");
  console.log(tokens, elapsed.toFixed(1));
}
```

The full script takes the median of five parses per count,
fits the exponent,
and runs unchanged on the three runtimes:

```sh
npm install --no-save --save-exact @optique/core@1.3.6
node token-scaling.ts
bun token-scaling.ts
deno run --no-config --import-map maps/npm-1.3.6.json token-scaling.ts
```

### Inputs that are slow

The script as written.
Fitted exponent,
1.3.6 then `1.4.0-dev.2971`:

- Node:
   1.85 and 1.75
- Deno:
   1.92 and 1.78
- Bun:
   1.63 and 1.57

The older release lines on Node,
five rounds with the order rotated,
lowest and highest fitted exponent:

- 1.0.15:
   1.85 to 1.92
- 1.1.13:
   1.85 to 1.93
- 1.2.13:
   1.82 to 1.92
- 1.3.6,
   in the same rounds:
   1.84 to 1.90

The four overlap throughout,
and so do their medians at 8,000 tokens in those rounds
(for scale only,
1.3 to 1.4 s each),
so the cost is the same on every release line from 1.0 to 1.3.

On GitHub's runners,
the workflow of the reproduction repository fitted 1.70 and 1.56 on Node
(run `43306311950`),
and its job that asserts an exponent below 1.5 failed on that step in each of three runs
(`43306241322`,
`43306311950`,
`43443075690`).

Putting `--` before the tokens does not avoid it:
the trace records tokens after the separator too.
The prototype made a thousand paths after `--` 0.76 times as long (0.74 to 0.79 over five alternating rounds),
so the trace copy was about a quarter of that parse.

### Inputs that stay fast

Five hundred tokens or fewer.
On Node and 1.3.6,
the cost per token at 500 tokens is about a tenth of the cost per token at 8,000 in the same run.

## Verified workarounds

### Split a long list of positional tokens across several parses

Eight parses of 1,000 tokens cost about 0.15 times as much as one parse of 8,000,
reckoned from the medians of the run under "Symptom";
eight separate parses were not timed against one.

Tradeoffs:

- The caller has to repeat the options for each batch and join the results.
- A parser that validates across all positional values
   (a count limit,
   a uniqueness check) no longer sees them together.

### Apply the prototype

The patch under "Upstream filing artifact" removes the trace copy,
the largest of the three copies,
and leaves the other two.
It was checked against upstream's source,
not as a patch to a published package's built files,
so using it means building `@optique/core` from source.

Tradeoff:
it is unreviewed by upstream and adds about 350 lines to maintain.

The second patch,
[`optique-multiple-rebuilds-its-state-array-for-each-accepted-item.patch`](optique-multiple-rebuilds-its-state-array-for-each-accepted-item.patch),
goes on top of it and removes the state copy of `multiple()`.
With both,
against the first patch alone and paired by round,
a run of positional tokens took 0.50 times as long at 8,000 tokens on Node 26.13.1,
0.24 times at 16,000 and 0.16 times at 32,000;
on Bun 1.4.4 the same ratios are 0.58,
0.50 and 0.43.
The ranges are under "Timing of a run of positional arguments".

Tradeoffs of the second:

- A state of more than 1,024 items is a `Proxy`.
   Printing it shows an empty array,
   `structuredClone()` and `Object.freeze()` on it throw,
   and so does editing it in place,
   as listed under "What a caller can notice".
- Just past that size it can cost on a line that alternates options and arguments:
   about a seventh at 6,000 tokens on Bun,
   and nothing measurable on Node.
- It is unreviewed by upstream and by any person,
   and adds about 600 lines of source and 770 of tests to maintain.
- The buffer copy stays,
   and from about 32,000 positional tokens it is most of the parse.

## What does not work

- Putting `--` before the positional tokens,
   as under "Inputs that are slow".
- A literal `errors.noMatch`,
   or a `1.4.0-dev` build.
   Both concern the typo-suggestion work,
   which is a different cost.
- Upstream pull request 1121,
   merged on 2026-11-13.
   It changes the option-scope work and does not touch `input-trace.ts`.
- Making the token buffer a view in `argument()` only,
   the second of the two prototypes.
   The other primitives then slice that view through its proxy.
   With the state fix in both arms,
   a line that alternates options and arguments took 18.3 times as long with the view as without it
   at 24,000 tokens on Node,
   and 48.1 times on Bun.
   The figures are under "Timing of options and arguments alternating".
- Copying the state of `multiple()` with `slice` and `push` in place of the spread.
   On Deno,
   the only runtime it was timed on,
   it measured the same as the spread.
   Copying it with `concat` took 0.44 times as long at 16,000 tokens on Node and 0.69 on Bun,
   and left the growth as it was.

## Upstream filing artifact

### Upstream filing decision

Decision:
a new report is warranted,
since upstream has no issue for this and has said its own issue 1120 does not cover it;
whether and how to send it is the owner's decision.

The owner's decision,
2026-11-13,
asked whether to raise the patch in the reply,
open the issue,
open a discussion or keep it here:
"Keep it local for now.
I want to have everything figured out ourselves before giving replies."
So nothing is filed for now.
The owner added the same day that this
"means it can be in the draft":
the reply draft in `doc/troubleshooting/optique-eager-typo-suggestions-on-positional-arguments.md`
reports the patch and asks whether upstream will take an issue,
and the draft issue here waits on that.

`.out-of-scope/`,
read 2026-11-13:
`.out-of-scope/jsr.md` line 43 mentions `@optique/*` in a list of packages once imported from JSR
and exempts JSR registry defects only,
so no exemption applies.

Duplicate search,
2026-11-13,
on `dahlia/optique`,
all states:

- `gh issue list --search 'InputTrace'`:
   issues 750 and 752,
   which introduced the trace.
- `gh pr list --search 'InputTrace'`:
   pull requests 760 and 765,
   the same work.
- `gh issue list --search 'quadratic OR "input trace" OR "trace copy" OR "copies its"'`:
   issues 750,
   752 and 1084;
   1084 is about startup and packaging cost.

None reports the copy.
The one mention is the sentence in issue 1120 quoted under "Status".

Work in progress,
2026-11-13:

- The one open pull request,
   1121 (branch `perf/option-scope-overhead`,
   head `a175cdb7`),
   changes `short-option.ts` and `execution-context.ts` and not `input-trace.ts`.
- `gh api repos/dahlia/optique/commits --method GET -f path=packages/core/src/input-trace.ts`:
   newest commit 2026-05-14.
- Branches:
   the maintenance lines,
   `main` and `perf/option-scope-overhead`;
   none named for the trace.
- Issue 1120 is assigned to the maintainer and scoped to exclude this.

Nobody is fixing it.
The maintainer is working in the neighbouring code today,
so read pull request 1121 as it stands on the day before posting.

The same checks again later on 2026-11-13,
after upstream moved:

- Pull request 1121 was merged at 2026-11-13T15:37:30Z as commit `d26f1a24`,
   which closed issue 1120.
   The merge commit's tree is that of the branch head `a175cdb7`,
   so the patch's clean application to that head holds for `main` as of the merge.
- `gh pr list --state open`:
   none.
- `gh issue list --state all`:
   nothing newer than issue 1120,
   and nothing about the trace.
- The newest commit on `packages/core/src/input-trace.ts` is still the one of 2026-05-14.

So nobody is fixing it as of that check either.

The six constraints:

1.  Upstream's fault:
    yes,
    as a cost.
2.  Upstream can fix it:
    yes.
    The prototype changes one class and adds one internal module.
3.  Supported use case:
    yes.
    Upstream issue 989 says
    "a successful parse with many paths should not require a full typo diagnostic for each option non-match",
    which takes a parse of many paths as a case to serve,
    and issue 1120 grants that this copy "contributes to large-input costs".
4.  Contribution welcome:
    conditionally.
    The maintainer built issue 1120 on this repository's write-up and linked it.
    Upstream's [`AI_POLICY.md`][policy pull requests] allows a pull request made with AI only for an accepted issue,
    "If you want to share code for a non-accepted issue,
    open a discussion or attach it to an existing discussion",
    and [requires][policy issues] issue text made with AI to be
    "reviewed and edited by a human before submission".
5.  Likely to fix:
    no signal against.
    The maintainer called it pre-existing and outside one issue's scope,
    which declines nothing.
6.  Minimal fix prototyped:
    yes,
    as recorded under "Prototype".

Risk to the poster:
a report would go out under the owner's account.
The last of the policy's numbered rules reads:
"Violations of this policy may result in being banned from contributing."
The draft and the patch were made with AI assistance,
so posting either unreviewed,
unedited or undisclosed would break that policy.
The upstream tree at `84f95710` holds no trust or blocklist file
(a listing of the whole tree for names matching `vouch`,
`denounce`,
`blocklist` or `trust` returns nothing;
`.github/` holds three workflows);
check the live repository again on the day of posting.
Filing is the owner's decision.

### Prototype

Made in a disposable clone of `https://github.com/dahlia/optique.git` at `1d6e4594`,
by an AI agent (DeepSeek through Pi) on 2026-11-12,
then read and rechecked on 2026-11-13 by a session from another vendor that did not write it.
The patch is [`optique-input-trace-copies-its-map-for-each-recorded-token.patch`](optique-input-trace-copies-its-map-for-each-recorded-token.patch),
640 lines added across five files.
It leaves out upstream's `CHANGES.md`,
which upstream's `sacho sync` regenerates from the fragment the patch adds.

What it does:
`InputTraceImpl` keeps its entries in an immutable hash-array-mapped trie
(`packages/core/src/internal/immutable-string-map.ts`) in place of a copied `Map`.
A write copies the nodes on the path to its key and shares the rest,
so earlier traces stay as they were.

One change made in the recheck:
the prototype had a test that timed 1,000 writes against 4,000 and failed above a ratio.
It was removed,
because upstream's pull request 1121 describes its own tests as written
"[without timing thresholds][no timing]"
and upstream has three open issues about tests that fail under load (1116,
1117,
1118).
With it gone,
no test in the patch would fail if the whole-map copy came back:
the tests guard what the trace returns,
and the growth is guarded only by the measurement.

Correctness,
run in a container with upstream's toolchain on 2026-11-13:

- `deno fmt --check`,
   `deno lint` and `deno check` on the four touched source files:
   clean.
- The two test files the patch touches,
   one new and one extended,
   on Deno:
   2 passed (32 steps),
   0 failed.
   On Bun they ran as part of the whole suite,
   reported in this list.
- Four deliberate breaks of the trie,
   each restored after its run
   (an overwrite that edits a shared leaf,
   a rank that ignores lower slots,
   a delete that keeps a cleared bit,
   a write that edits a shared branch):
   the new tests failed on every one.
- Upstream's core suite on Deno:
   777 passed (8,289 steps),
   0 failed,
   29 ignored.
- Upstream's core suite on Node 26 after `pnpm build`:
   8,075 pass,
   0 fail,
   529 skipped.
- `sacho sync --force`,
   `sacho fmt` and `sacho check`:
   exit 0.
   The exit status of the check is in the records;
   those of the other two were printed to the session and not saved.
- `git apply` of the patch onto `main` at `84f95710` and onto pull request 1121's head `a175cdb7`:
   clean on both,
   checked again with the patch as committed here.

Upstream's core suite on Bun 1.4.4,
the current release and the one this repository's toolchain lock records,
run on 2026-11-13 on an unpatched worktree of `1d6e4594`,
then on the patched tree,
then on the unpatched one again:

- Patched:
   8,075 pass,
   529 skipped,
   0 fail,
   every file loaded.
- Unpatched,
   both runs:
   7,286 pass,
   529 skipped,
   no test failed,
   and two test files did not load
   (`facade.test.ts` and `short-options.test.ts`,
   "Cannot find module '@optique/env'",
   a workspace package that was not built in that fresh worktree),
   which Bun counts as two failures.

So the patched tree passes upstream's suite on the current Bun.
The tests of the two files that did not load without the patch ran only with it.

An earlier version of this paragraph reported the same comparison on Bun 1.3.0,
with 154 failing shell-completion tests on both sides.
That Bun was old:
the prototype's container image pinned it with no reason given.
Those failures are not there on the current release,
they are no finding about upstream,
and nothing about them was or is to be reported.
The owner,
2026-11-13:
"we don't care about support for older versions,
just like Bun itself doesn't care."

Effect,
on Deno 2.9.8 from source,
five alternating rounds,
unpatched `1d6e4594` against patched:

- 8,000 positional tokens:
   0.12 times as long (0.118 to 0.128).
   Fitted exponent 1.77 to 1.87 before,
   1.18 to 1.27 after.
- A thousand paths through the `git commit` option set:
   0.78 (0.73 to 0.82).
- The same after `--`:
   0.76 (0.74 to 0.79).

Every round was faster in all three.
The same comparison at larger sizes on Node and Bun is under "What is left after the fix".
For scale only,
the medians went from 1,182.8 to 150.7 ms,
from 75.1 to 58.7 ms and from 60.1 to 46.4 ms.

Together with upstream's pull request 1121,
same day,
same container limits,
seven rounds with the order of the arms rotated,
each arm from source on Deno except 1.3.6 from npm:
`main` at `84f95710`,
the pull request's head `a175cdb7`,
and each with the patch applied.
It was measured on Deno only.
Each arm's median over the seven rounds,
as a ratio to the median of 1.3.6 in the same rounds:

- A thousand paths after `--`:
   `main` 1.83,
   pull request 1.36,
   `main` with the patch 1.45,
   pull request with the patch 0.92.
- A thousand paths with no separator:
   `main` 0.53,
   pull request 0.43,
   pull request with the patch 0.32.
- 8,000 positional tokens:
   `main` 1.01,
   pull request 1.01,
   pull request with the patch 0.12.

For scale only,
the medians of 1.3.6 were 30.8,
133.3 and 1,119.8 ms.

Paired by round,
which is the firmer figure,
for the paths after `--`:

- `main` against 1.3.6:
   1.87 (1.79 to 1.92),
   all seven rounds slower.
- The pull request against `main`:
   0.75 (0.71 to 0.77),
   all seven faster.
- The pull request against 1.3.6:
   1.37 (1.34 to 1.42),
   all seven slower,
   which agrees with the pull request's own statement that
   "the separator case remains slower than 1.3.6".
- The pull request with the patch against 1.3.6:
   0.94 (0.86 to 1.07),
   five rounds faster and two slower,
   which seven rounds cannot tell from no difference.

That last figure needs reading with care.
1.3.6 pays for the trace copy too,
so the copy is no part of the gap between the pull request and 1.3.6;
that gap is option-scope work the pull request leaves.
`main` with the patch against 1.3.6 shows it:
1.45 by the medians of the seven rounds.
The patch removes from the pull request a cost that 1.3.6 also carries and that is about as large as the gap
(by those medians,
the patch takes 0.44 of the 1.3.6 time off the pull request,
and the gap is 0.36 of it),
which is why the two land together.
An independent review on 2026-11-13 caught an earlier sentence here,
which called the trace copy what separates the two.
The pull request does not change the 8,000-token figure (1.00 against `main`,
0.82 to 1.04),
and the patch does not depend on it.

The same comparison was repeated on Node 26.13.1 and Bun 1.4.4 with every arm a built package,
after the owner asked on 2026-11-13 why a timing was on Deno only:
seven rounds with the order of the arms rotated,
`main` before the merge at `84f95710`,
the merge `d26f1a24` with and without the patch,
1.3.6 from npm,
and the merge's package linked a second time as a same-code control.
A thousand paths after `--`,
paired by round:

- The merge's package over itself:
   Node 1.00 (0.96 to 1.03),
   Bun 1.06 (0.95 to 1.09).
- The pull request against 1.3.6:
   Node 1.37 (1.27 to 1.63),
   Bun 1.95 (1.62 to 1.99),
   all seven rounds slower on each.
- The patch on the merge against the merge:
   Node 0.67 (0.61 to 0.76),
   all seven faster;
   Bun 1.07 (0.95 to 1.22),
   five of seven slower,
   which is no further from 1 than the control.
- The pull request with the patch against 1.3.6:
   Node 0.92 (0.84 to 1.05),
   one of seven slower;
   Bun 1.92 (1.84 to 2.28),
   all seven slower.

With no separator the patch on the merge against the merge was 0.73 on Node (0.70 to 0.78),
all seven faster,
and 0.96 on Bun (0.72 to 1.15),
three of seven slower.

So at a thousand tokens the patch is a gain on Node and Deno and nothing measurable on Bun.
On Bun its gain shows with more tokens:
in the rounds under "What is left after the fix" the patched build over `main` was 0.74 at 2,000 positional tokens,
0.55 at 4,000 and 0.30 at 8,000.
The sentence that the patch brings the pull request level with 1.3.6 after `--` therefore holds on Node and Deno
and not on Bun,
where the pull request with the patch still takes about twice as long as 1.3.6.

The records are in `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`,
folder `measurements-2026-11-13`:
`cross-runtime/results.jsonl` for the three runtimes and the first patched comparison,
`upstream-arms-results.jsonl` for the comparison with the pull request,
and the container recipes and runners that produced them.
The repeat on Node and Bun is in the folder `pull-request-1121-on-node-and-bun-2026-11-13` of the same archive.

### Prototypes for the two further copies

Unapproved and not final.
On 2026-11-13 the owner chose to have fixes for the two copies named under
"What is left after the fix" prototyped before any reply to upstream.
An AI agent (DeepSeek through Pi) wrote two diffs that day in a disposable clone,
on upstream `main` at `d26f1a24` with the trace patch applied.
An AI session from another vendor rechecked them the same day:
it applied the agent's diff files itself to a clean checkout of that commit,
ran probes and timings of its own,
and took nothing from the agent's build trees.
A model from a third vendor then reviewed the kept patch and this section as of commit `758946f3e1`;
it recomputed the figures from the records,
found one more flaw in the patch,
which is fixed,
and one sentence the figures contradicted,
which is corrected.
The timing on Node and Bun and the present form of the timing parts came after that review.
No person has read the diffs or this section.

The result:

- The diff for `multiple()`'s state holds up,
   with five changes made since,
   three of them fixes of flaws in its view
   (two found in the recheck,
   one by the independent review).
   It is kept as
   [`optique-multiple-rebuilds-its-state-array-for-each-accepted-item.patch`](optique-multiple-rebuilds-its-state-array-for-each-accepted-item.patch),
   1,404 lines added across seven files.
- The diff for the token buffer does not.
   Once it takes effect,
   a command line that alternates options and arguments takes eleven to thirteen times as long on Node
   and forty to seventy-seven times as long on Bun.
   It is not kept as a patch;
   it is in the evidence archive for anyone who wants to repeat the measurement.
- With both copies gone,
   a run of positional arguments parses in time about proportional to its length up to 128,000 tokens
   on Bun,
   and on Deno with the thresholds at zero.
   On Node with the thresholds as written the growth stays above that,
   and the arm that would explain it was not built.
   This is measured for a run of positional arguments and for nothing else.

#### What the two diffs do

Both keep the types as they are
(`readonly TState[]` for the state,
`readonly string[]` for the buffer)
by handing out a view:
a `Proxy` over an empty array that answers index reads,
`length` and the `Array.prototype` methods from storage it does not own.

- State:
   above 1,024 items the state of `multiple()` becomes a view of an immutable vector,
   a 32-way tree in which an append or a drop of the last item copies only the nodes on one path.
   Up to 1,024 items the code keeps the plain arrays and the expression it had.
- Buffer:
   while 16,384 tokens or more are left,
   `argument()` cuts the buffer as a view that remembers the array it was cut from and how far;
   with fewer left it slices as before.

The thresholds are the agent's,
from its own measurement that a view costs more than a copy on short inputs.
The recheck's timings agree that a threshold is needed:
with every state and buffer made a view,
a parse of a thousand paths through a parser with twenty-eight options took 1.59 times as long as without
(1.47 to 1.68 over five rounds).

#### Behaviour

- All four diff files apply to `main` at `d26f1a24` with no fuzz,
   the combined one gives the same tree as the three it combines,
   and no diff removes a line from a test file upstream already had.
   The agent's working tree was identical to the tree made from its diff files.
- Upstream's `packages/core` suite,
   on Deno 2.9.8,
   Node 26.13.1 and Bun 1.4.4,
   in a container with no network:
   - `main` with the trace patch:
      778 passed on Deno (8,301 steps),
      8,086 on Node,
      8,086 on Bun,
      none failing.
   - With the agent's diffs as written:
      784 (8,340 steps),
      8,125 and 8,125,
      none failing.
   - With the kept patch:
      782 (8,332 steps),
      8,117 and 8,117,
      none failing,
      and `deno fmt --check`,
      `deno lint` and `deno check` clean on the six source files it adds or changes.
- "Every existing test passes" says little at those thresholds:
   a test runs the new code only if it builds a state of 1,024 items or leaves 16,384 tokens in a buffer,
   and how many of upstream's tests do was not counted.
   With both thresholds set to two,
   so that nearly every state and buffer is a view,
   the suite still passes whole:
   784,
   8,125 and 8,125 on the agent's diffs,
   and 782,
   8,117 and 8,117 on the kept patch with its one threshold at two.
   With both set to zero,
   five of the agent's new test cases fail
   (Node and Bun count five failing tests,
   Deno five failing steps in four tests)
   and no test upstream had before does;
   each of the five sizes itself as the threshold less one and reads index -1 or a token that is not there.
- Two probes written for the recheck print what the library returns and how states and buffers read to a caller
   (`Array.isArray`,
   `length`,
   index,
   spread,
   `slice`,
   `map`,
   `concat`,
   JSON,
   earlier contexts after later ones were made).
   One stays under 300 tokens;
   the other passes both thresholds,
   takes 1,067 parse steps one at a time,
   and replaces the last item of a state of 1,031 items in two different ways from one earlier state.
   Every arm gave output identical to the base on both.
   Three deliberately broken copies were run beside them:
   a break of the plain-array path showed on both probes,
   and two breaks inside the view code showed on the long probe only,
   so the long probe does reach the view code.
- A parse that carries annotations copies them onto states as it goes.
   At 16,000 tokens such a parse took between 0.23 and 0.32 times as long with the state fix as without,
   in three rounds on Deno with the arms taking turns,
   so annotations do not undo it.

Five changes made after the agent's work,
all in the kept patch,
four found in the recheck and one by the independent review:

- The agent's parser-level tests only append to a long state.
   A view that failed to drop the item being replaced was caught by one unit test and by no test of `multiple()`.
   A test now fills a state past the threshold with two-option items,
   replaces the last one in two ways,
   and reads all three states back;
   it fails under that break.
- `Object.freeze()` on a view threw only after it had made the view's empty target array non-extensible,
   and from then on `Object.keys()` of that view threw as well,
   on all three runtimes.
   The view now refuses to be made non-extensible,
   so a freeze still throws and the view stays readable;
   a test guards it and fails when the refusal is removed.
- The view refused a write only to an index it holds.
   A write past its end went through to the empty target array:
   `state[state.length] = x` raised nothing,
   a `push()` threw only after it had written,
   and either way the value then read back at that index and showed in `Object.keys()` while `length` stayed.
   `Object.defineProperty()` on an index raised nothing either,
   and on an index the view holds it made `Object.keys()` throw afterwards.
   All of that was measured on the three runtimes on the agent's diffs.
   The view now refuses every index,
   in assignments and in `Object.defineProperty()`,
   and a test guards it and fails when the refusal is narrowed again.
- The view reported its `length` descriptor from the empty target,
   value 0,
   while reading `length` gave the real count.
   A copy made with `Object.defineProperties([], Object.getOwnPropertyDescriptors(state))` came back empty.
   The independent review of 2026-11-13 found this and showed it with a state of 1,027 items;
   the library copies array states that way in `annotation-state.ts`,
   and the reviewer traced that a `multiple()` state does not reach that code in the current flow.
   The descriptor now carries the real length,
   and a test guards it and fails when the target's own value is reported again.
- Comments:
   one cited two of the agent's measurement scripts by name,
   which upstream does not have;
   one said a view is built with no closures while each caller passes a function made per view;
   and two module headers and the error message spoke of the input buffer,
   which the kept patch does not touch.

The suite counts were taken on the kept patch as it now is,
and so were six deliberate breaks,
each caught by a test that names what it guards:
the two of the replace-last path,
a state view that reads the wrong item,
and one for each of the three view fixes.
The two probes were run on the agent's diffs and again on the kept patch,
with the break of the state view made in the kept patch the second time.

One of the nine full Bun runs made in the recheck failed one test,
"Bash Readline file completion quoting > keeps backticks literal in a quoted tilde pattern",
whose interactive `bash` ended on `SIGPIPE`.
It is in a file the patch does not touch,
it passed alone five times of five on the base and on the state fix,
and the three later full runs of that arm passed.
What made it fail once was not found.

#### What a caller can notice

A state above the threshold is a `Proxy`,
and a caller that only reads it as an array sees no difference.
One that does more can
(measured on the three runtimes,
with every state made a view):

- Printing it shows the empty target:
   `[]` from `console.log` on Deno and Bun,
   `Proxy([])` on Node.
   `JSON.stringify` and a spread show the items.
- `structuredClone()` throws `DataCloneError`.
- Every write to an index or to `length` throws a `TypeError` and leaves the state as it was:
   an assignment,
   `Object.defineProperty()`,
   `delete`,
   and `push`,
   `pop`,
   `unshift`,
   `splice`,
   `fill`,
   `copyWithin`,
   `sort` and `reverse`.
   On a plain array each of them changed that parse's own copy.
   A property that is not an index can still be set,
   symbol-keyed or named,
   which is how the library attaches annotations.
- `Object.getOwnPropertyDescriptor()` of an index says `writable: true`,
   although a write throws.
   Saying `false` there would make the elements of a copy made from the descriptors read-only.
- `Object.freeze()`,
   `Object.seal()` and `Object.preventExtensions()` throw,
   and `Object.isFrozen()` is false.

A search of every upstream package's source for `Object.freeze`,
`structuredClone`,
`Object.seal` and `preventExtensions` finds none applied to a `multiple()` state or a token buffer.
What third-party parsers do with a state they are handed was not looked at.

#### How the timing was measured

Every figure here is a ratio between two arms that ran one after the other in the same round,
given as the median over the rounds with the range of the single rounds.
The machine was shared with other work the whole time,
so no absolute time stands on its own,
and none is given as a finding.
The order of the arms rotated from round to round.
A ratio whose range straddles 1 is no finding in either direction.

How far two runs of identical code differ was measured in the runs themselves.
Below its threshold an arm runs the very code of the base.
On Deno the state fix over the base had a median of 0.82 at 100 tokens and 1.06 at 1,000,
with single rounds from 0.56 to 1.53;
on Node and Bun no size wholly below the threshold was timed.
One check ran the base twice in each of ten rounds:
the second over the first had a median between 0.97 and 1.03 at every size,
with single rounds from 0.75 to 1.25.

The runtimes:

- Node 26.13.1 and Bun 1.4.4,
   on the package built from each arm with upstream's own build.
   These are the runtimes this repository runs on,
   and they were timed last,
   after the owner asked on 2026-11-13 why the timing was on Deno only.
   The state fix on them is the kept patch as it stood before its last change,
   which touches only how a view answers a request for its `length` descriptor.
- Deno 2.9.8,
   from source through an import map.
   The first rounds were taken there because that needs no build,
   which was a convenience and no reason.
   Deno is also being wound down:
   its maintainers [said on 2026-10-15][deno state] that it is maintained "until October 2027",
   with bug fixes and no new features.
   Upstream supports it,
   so its figures are kept and labelled.
   On Deno the first rounds ran the agent's diffs,
   and the kept patch was timed beside the agent's arm in five more.

The arms:
`main`;
the base,
which is `main` with the trace patch;
the state fix;
the buffer fix,
on Deno alone and together with the state fix on all three;
and on Deno only,
both fixes with their thresholds at zero.
Five rounds each unless a count is given.

#### Timing of a run of positional arguments

One unused option and the arguments (`token-scaling.ts`).

The state fix over the base:

- Node:
   0.50 at 8,000 tokens (0.48 to 0.87),
   0.24 at 16,000 (0.19 to 0.33),
   0.16 at 32,000 (0.14 to 0.19).
- Bun:
   0.58 at 8,000 (0.25 to 0.83),
   0.50 at 16,000 (0.42 to 0.78),
   0.43 at 32,000 (0.41 to 0.45).
- Deno:
   0.39 at 8,000 (0.33 to 0.59),
   0.20 at 16,000 (0.15 to 0.28),
   0.22 at 32,000 (0.14 to 0.33).

It was slower in none of the forty-five pairs at those sizes.
At 2,000 and 4,000 tokens it reads between 0.72 and 0.94 on the three runtimes,
with three of thirty pairs slower,
which is near the spread of identical code.
On Bun the gain is smaller than on Node:
0.43 against 0.16 at 32,000 tokens.

For comparison,
the base over `main`,
which is what the trace patch does,
measured the same way at 16,000 tokens:
0.12 on Node (0.11 to 0.13) and 0.25 on Bun (0.20 to 0.32).

How the time grows when the input doubles,
as the ratio of two sizes within one process,
written as an exponent
(1 is proportional,
2 is the square),
from 2,000 to 32,000 tokens and then from 64,000 to 128,000:

- Base on Node:
   0.49,
   1.61,
   2.41,
   2.83.
- Base on Bun:
   1.24,
   1.35,
   1.60,
   1.91.
- State fix on Node:
   0.54,
   0.85,
   1.30,
   2.48,
   then 2.62.
- State fix on Bun:
   0.95,
   0.91,
   1.52,
   1.59,
   then 1.95.
- Both fixes on Node:
   0.38,
   1.12,
   1.28,
   1.73,
   then 1.53.
- Both fixes on Bun:
   1.07,
   1.00,
   1.25,
   0.96,
   then 0.84.
- Both fixes with thresholds at zero on Deno:
   0.95,
   0.50,
   1.17,
   0.93,
   then 1.06.

So with the state fix alone the growth climbs back toward the square as the input grows,
and that is the buffer copy.
Both fixes over the state fix alone,
on a positional run:

- Node:
   0.72 at 32,000 tokens (0.61 to 0.83),
   0.24 at 64,000 (0.22 to 0.25),
   0.11 at 128,000 (0.10 to 0.15).
- Bun:
   0.60 at 32,000 (0.54 to 0.64),
   0.31 at 64,000 (0.20 to 0.38),
   0.14 at 128,000 (0.12 to 0.18).

With both copies gone,
no cost that grows with the square shows on this input up to 128,000 tokens:
on Deno with the thresholds at zero the exponents stay near 1,
and on Bun with the thresholds as written they do too.
On Node with the thresholds as written the last two are 1.73 and 1.53,
and the arm that would say whether that is the threshold's leftover or something else,
thresholds at zero,
was not built for Node.

#### Timing of options and arguments alternating

`-v v0 p0 -v v1 p1` for the whole line,
a script written for the recheck.

The buffer fix makes this slower by an order of magnitude once it takes effect.
Both fixes over the base:

- Node:
   10.9 at 24,000 tokens (7.1 to 13.9),
   13.2 at 48,000 (9.2 to 17.7).
- Bun:
   39.9 at 24,000 (29.2 to 46.5),
   77.0 at 48,000 (63.6 to 101.9).
- Deno:
   10.5 at 24,000 (8.4 to 16.1),
   11.2 at 48,000 (10.1 to 11.9);
   the buffer fix alone,
   8.8 (8.5 to 11.2) and 12.0 (9.8 to 13.8).

Those ratios set a build with both fixes against a build with neither.
Both fixes over the state fix alone,
which holds the state fix constant and so shows what the buffer fix adds:

- Node:
   18.3 at 24,000 tokens (13.5 to 22.3),
   13.0 at 48,000 (10.4 to 16.7).
- Bun:
   48.1 at 24,000 (24.1 to 65.0),
   105.0 at 48,000 (74.9 to 153.8).
- Deno:
   15.6 at 24,000 (12.0 to 22.6),
   11.4 at 48,000 (9.5 to 12.8).

On Deno both arms of that second comparison carry the state fix as the agent wrote it.
On Node and Bun the build with both fixes carries the agent's and the other arm is the kept patch,
which differs from it in the traps for writes,
freezing and descriptors.
That the two measure alike where the buffer fix is not in effect was checked in the same rounds:
up to 12,000 tokens the same ratio had medians from 0.99 to 1.05 on Node,
and from 0.96 to 1.10 on Bun,
the 1.10 at 3,000 tokens,
where all five rounds were slower.

At 24,000 and 48,000 tokens every comparison was slower in every round on every runtime.
At 12,000 tokens and below,
where the buffer fix does not take effect,
there is no such loss.
The cause is in the source.
On `main`,
`buffer.slice(` is called at 41 places outside the tests,
and sixteen of them in `primitives.ts` build the next buffer that way,
[the option parser's among them][option buffer].
The diff changes the two in `argument()`.
Once `argument()` has made the buffer a view,
the next option parser calls `Array.prototype.slice` on it,
which reads every remaining token through the proxy.
On Deno with the thresholds at zero the same input took 5.1,
10.1 and 12.9 times as long as the base at 3,000,
6,000 and 12,000 tokens.
The agent did not measure this shape:
its scripts were the positional run and a thousand paths.

The state fix over the base on the same input:

- Node:
   1.01 at 3,000 tokens,
   1.05 at 4,500 (0.89 to 1.27),
   0.98 at 6,000,
   0.72 at 12,000 (0.56 to 0.83),
   0.60 at 24,000 (0.39 to 0.68),
   0.94 at 48,000 (0.86 to 1.06).
- Bun:
   0.98 at 3,000,
   1.08 at 4,500 (0.91 to 2.11),
   1.14 at 6,000 (1.03 to 2.16,
   slower in all five rounds),
   0.85 at 12,000 (0.75 to 2.31),
   0.77 at 24,000 (0.66 to 1.65),
   0.71 at 48,000 (0.66 to 0.95).
- Deno,
   ten rounds with a same-code control in each:
   0.98 at 1,500,
   0.99 at 3,000,
   1.11 at 4,500 (0.91 to 1.45,
   slower in eight of ten,
   where the control reads 1.03),
   0.94 at 6,000,
   then 0.87,
   0.78 and 0.76 at 7,500,
   9,000 and 12,000,
   slower in none of thirty pairs.
   A later five rounds read 0.99 at 4,500 for the kept patch (0.53 to 1.06).

This input fills two states,
which pass 1,024 items at about 3,100 tokens.
Just past that the fix costs something on this shape:
about a seventh at 6,000 tokens on Bun,
about a tenth at 4,500 on Deno in one of two checks,
and nothing measurable on Node.
So 1,024,
which the agent chose from a positional run,
is somewhat low for this shape,
and what a better threshold is was not worked out.
At 48,000 tokens the fix gains little on Node and Deno;
what takes the time in the base there was not profiled.

#### Timing of a parser shaped like `git commit`

Twenty-eight options and the paths (`commit-paths.ts`).

- A thousand paths:
   no arm differs from the base by more than the spread on any runtime
   (with and without `--` on Deno,
   without on Node and Bun),
   except thresholds-at-zero on Deno,
   1.59 (1.47 to 1.68).
   That last figure is why a threshold is needed at all.
- 20,000 paths,
   the state fix over the base:
   0.43 on Node (0.37 to 0.48),
   0.79 on Bun (0.55 to 0.93),
   0.42 on Deno (0.27 to 0.47),
   slower in none of fifteen pairs.
- 20,000 paths,
   both fixes over the state fix:
   1.16 on Node (1.03 to 1.17,
   slower in all five rounds)
   and 0.99 on Bun.
   So here too the buffer fix costs on Node where it takes effect.

#### Timing of the same copy written with `concat`

One smaller thing was tried on the side:
`(added ? context.state : context.state.slice(0, -1)).concat([nextItemState])` in place of the spread.
It changes nothing a caller can see,
and upstream's suite passes with it on the three runtimes
(778,
8,086 and 8,086).
Over the base on a positional run:

- Node:
   0.72 at 2,000 tokens,
   0.71 at 4,000,
   0.72 at 8,000,
   0.44 at 16,000 (0.41 to 0.46),
   0.46 at 32,000 (0.35 to 0.49),
   slower in none of twenty-five pairs.
- Bun:
   0.76,
   0.71,
   0.69,
   0.69 (0.61 to 0.71)
   and 0.61 (0.46 to 0.63),
   slower in one of twenty-five.
- Deno:
   0.74,
   0.80,
   0.56,
   0.33 (0.28 to 0.44)
   and 0.38 (0.27 to 0.66),
   slower in none of twenty-five.

At 20,000 paths it reads 0.60 on Node (0.47 to 0.69) and 0.80 on Bun (0.72 to 1.28).
It does not change how the time grows:
on Node the exponent from 16,000 to 32,000 tokens is 2.88 with it and 2.83 without.
Against the state fix it is level or ahead up to 4,000 tokens and then falls behind,
2.6 times as long at 32,000 on Node and 1.4 times on Bun.
Copying with `slice` and then `push`,
timed on Deno only,
gained nothing measurable.

#### Absolute times for scale only

Absolute times from these runs,
which depend on what else the machine was doing and are no finding:
at 32,000 positional tokens the base took about six seconds on Node and two on Bun,
and the state fix about one second on each;
at 48,000 alternating tokens both fixes took about half a minute on Node and a minute and three quarters on Bun,
against two and a half seconds and one and a half for the base.

#### Where the agent's report and the recheck differ

- The report gives one fitted exponent for both fixes,
   1.14.
   Per doubling it is the figures listed under "Timing of a run of positional arguments",
   and on Deno,
   where the report measured,
   1.82 from 16,000 to 32,000.
- It calls the buffer fix marginal,
   "nothing below its threshold" and 1.25 times at 32,000.
   On a positional run it is worth far more at larger sizes,
   and on alternating input it is a loss of ten times and more.
- It prices a copy at "a third of a nanosecond per token".
   The price is not constant.
- It says Node,
   Deno and Bun "all run V8".
   Bun runs JavaScriptCore,
   so the suite on Bun did exercise the proxy on a second engine.
- It blames this machine's Git wrapper for emptying `node_modules` in its throwaway worktrees.
   The feed of its run shows the failed imports and no cause.

#### What is not established

- What dominates the alternating input at 48,000 tokens.
- Why the growth of both fixes stays above proportional on Node at the largest sizes.
- Any timing of the kept patch without the trace patch under it
   (it applies to `main` alone,
   and was timed only on top of the trace patch),
   and any timing of its last form,
   which differs from the timed one in how a view answers for its `length` descriptor.
- `multiple()` inside `multiple()`,
   a `min` or `max` bound at these sizes,
   and timing through an asynchronous value parser,
   whose results the probes did compare.
- A good threshold for inputs other than a positional run.
- A way to remove the buffer copy that does not slow other parsers.
   Changing all sixteen places that build the next buffer,
   or giving the context an offset into one array,
   was not tried;
   the second changes what `buffer[0]` means to every parser a user has written.

The records are in `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`,
folder `two-remaining-copies-recheck-2026-11-13`:
the agent's four diff files and its report,
the scripts that staged the arms and ran each check,
and every result quoted here.

### Readiness check

2026-11-13:

1.  Latest release and other maintained lines:
    reproduced on 1.3.6,
    the latest release,
    on `1.4.0-dev.2971`,
    and on `main` from source.
    1.1.1 on 2026-11-12,
    and 1.0.15,
    1.1.13 and 1.2.13 on 2026-11-13,
    with the same growth as 1.3.6.
2.  Prior art in the code:
    read issue 752 and the trace's history;
    no stated reason for the copy.
3.  Intake:
    upstream has no issue or pull request templates.
    Its policy asks for an accepted issue before a pull request made with AI,
    so the artifact is an issue,
    with the patch linked from it.
4.  Automation on a new filing:
    upstream's `main.yaml` runs on pull requests,
    pushes to branches and to tags,
    and manual dispatch,
    not on issues.
5.  Duplicate,
    work-in-progress and trust-list searches:
    run today,
    as recorded under "Upstream filing decision".
6.  The draft's reproduction run exactly as written:
    yes,
    the three commands under "Harness",
    in the container described under "Versions under test".
7.  Reproduction runnable by others on a clean machine through CI:
    the repository exists and three runs fail on the asserting step.
    The owner had it made public on 2026-11-13,
    after every file,
    commit message and commit author was read:
    the files hold only the reproduction,
    and its first commit,
    made by GitHub when the repository was created,
    carries the account's own address and not its noreply address,
    which the owner was told before choosing.
8.  Verdict:
    the evidence is in place for a report,
    and the owner has decided not to send one for now.
    If that changes,
    upstream's policy requires a human to review and edit the text first,
    and the risk named under "Upstream filing decision" comes with posting.

### Draft issue (do not post as-is)

An AI session wrote every line of it and no person has reviewed it.

~~~~md
`InputTraceImpl.set()` copies its whole map for each recorded input, so a successful parse of N positional
arguments copies N(N + 1) / 2 trace entries. You noted it in #1120 as outside that issue's scope; this is the
issue for it.

With one option and a run of positional arguments on 1.3.6 and Node 26, sixteen times the arguments (500 to 8,000)
took 173 times as long in one run: each doubling multiplied the time by 3.2 to 4.3, where a linear parse would give
2.

The fitted exponent of time against argument count was 1.85 on Node, 1.92 on Deno and 1.63 on Bun for 1.3.6, and
1.75, 1.78 and 1.57 for 1.4.0-dev.2971. `recordTrace()` in *primitives.ts* calls `set()` once per recorded input,
and `set()` does `new Map(this.#entries)`:

https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/input-trace.ts#L140-L144

A script that fits the exponent, and a workflow that runs it on pinned versions:
https://github.com/Aquaticat/optique-parse-cost-reproduction (*token-scaling.ts*).

I have a patch that keeps the entries in an immutable hash trie instead, so a write copies only the path to its
key and earlier traces stay untouched. Run against the unpatched build in the same rounds, 8,000 arguments take
0.11 times as long with it on Node (0.10 to 0.12) and 0.30 times on Bun (0.15 to 0.33), and the fitted exponent
goes from about 1.8 to about 1.2 (that last from source on Deno). The gain grows with the count: a thousand paths
through a `git commit`-shaped parser took 0.73 times as long on Node (0.70 to 0.78) and showed no difference on Bun
(0.96, 0.72 to 1.15). The core suite passes on Deno, Node and Bun with it, and it applies cleanly to main as of
#1121's merge.

It does not make the parse linear. Two more per-argument copies sit behind this one: `multiple()` rebuilds its
state array for each item (*modifiers.ts*, `annotateFreshArray(context.state, [...])`), and each primitive slices
the rest of the buffer for each token it consumes (`context.buffer.slice(i + 1)` in `argument()`, and fourteen more
places in *primitives.ts*). Timed as built packages on Node 26 and Bun 1.4, five rounds with the arms taking turns,
16,000 arguments take 0.12 times as long with this patch as on main on Node (0.11 to 0.13) and 0.25 times on Bun
(0.20 to 0.32).

I tried both. For the state I have a second patch: above 1,024 items the state becomes an array-shaped view (a
`Proxy`) over an immutable vector, and below that nothing changes. On top of this patch, 16,000 arguments take
0.24 times as long on Node and 0.50 on Bun, and 32,000 take 0.16 and 0.43. The core suite passes with it on Deno,
Node and Bun, also with the threshold lowered to two so that nearly every state is a view. Its cost is that such a
state is a proxy: it prints as an empty array, `structuredClone()` and `Object.freeze()` throw on it, and in-place
edits throw. For the buffer I have nothing to offer: with the state copy already removed, also cutting a view in
`argument()` made a line that alternates options and arguments 13 to 18 times slower on Node and 48 to 105 times
slower on Bun, because the other primitives then slice the view through the proxy. With the state fixed, the buffer
copy is most of what is left from about 32,000 arguments.

It also bears on what #1121 leaves, on Node. As built packages, seven rounds with the arms rotated, a thousand
paths after `--` took 1.37 times as long on #1121's merge as on 1.3.6 on Node (1.27 to 1.63) and 0.92 times with
this patch on top (0.84 to 1.05, one of seven rounds slower). On Bun the merge took 1.95 times as long as 1.3.6
(1.62 to 1.99) and the patch changed nothing at that size (1.92, 1.84 to 2.28).

The patch, with its tests: [link to the patch file in this repository at a commit].

Per AI_POLICY.md I am not opening a pull request without an accepted issue. Would you take one for this?

[Owner: add the disclosure of AI assistance that upstream's policy asks for, in your own words: the patch was
written by an AI agent and rechecked by another.]
~~~~

[scoped out]: https://github.com/dahlia/optique/issues/1120#:~:text=The%20separate%20whole%2DMap%20copy,outside%20this%20issue%27s%20scope
[record trace]: https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/primitives.ts#L162-L176
[trace set]: https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/input-trace.ts#L140-L144
[multiple state]: https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/modifiers.ts#L2743-L2746
[argument buffer]: https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/primitives.ts#L2725
[deno state]: https://github.com/denoland/deno/issues/37030#:~:text=will%20continue%20maintain%20Deno%20until%20October%202027
[option buffer]: https://github.com/dahlia/optique/blob/d26f1a244d4434f9dbfeb0317f6db32533cb8c32/packages/core/src/primitives.ts#L1251
[policy pull requests]: https://github.com/dahlia/optique/blob/84f9571069584d8e36a3a4cfbaebf8a9a61fc645/AI_POLICY.md?plain=1#L14-L18
[policy issues]: https://github.com/dahlia/optique/blob/84f9571069584d8e36a3a4cfbaebf8a9a61fc645/AI_POLICY.md?plain=1#L25-L27
[no timing]: https://github.com/dahlia/optique/pull/1121#:~:text=without%20timing%20thresholds
