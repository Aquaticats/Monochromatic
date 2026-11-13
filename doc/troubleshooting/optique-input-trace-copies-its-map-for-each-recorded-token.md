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
No person and no model from another vendor has reviewed that section or the second patch.

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
the median of five parses was:

- 500 tokens:
   7.7 ms
- 1,000 tokens:
   24.5 ms
- 2,000 tokens:
   84.2 ms
- 4,000 tokens:
   310.6 ms
- 8,000 tokens:
   1,331.9 ms

Sixteen times the tokens cost 173 times as long.
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
The patched build on Node 26.13.1,
median of five parses:

- 2,000 tokens:
   22.5 ms
- 4,000 tokens:
   49.2 ms
- 8,000 tokens:
   144.3 ms
- 16,000 tokens:
   777.8 ms
- 32,000 tokens:
   5,702.5 ms,
   from a second run in which 16,000 tokens took 823.5 ms

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

The unpatched build at the same sizes,
median of three parses:
16,000 tokens 6,687.4 ms and 32,000 tokens 29,164.0 ms,
with `set` in `input-trace.js` at 92.7% of the parse.
So the trace copy is by far the largest of the three at every size measured,
the prototype removes that one only,
and what is left at the larger sizes is mostly the other two,
nine tenths of the patched parse at 16,000 to 32,000 tokens.
At 4,000 to 8,000 the same two functions are 14.6% and 13.3% of the patched parse,
27.9% together,
and the two files they sit in are 54.7%;
the profile does not separate the copies from the other work in those functions,
so it gives no closer figure for the copies' own share at those sizes.
The patched build takes about an eighth of the time at 16,000 tokens and about a fifth at 32,000.
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
lowest and highest fitted exponent,
then lowest and highest median for 8,000 tokens:

- 1.0.15:
   1.85 to 1.92;
   1,319 to 1,358 ms
- 1.1.13:
   1.85 to 1.93;
   1,324 to 1,403 ms
- 1.2.13:
   1.82 to 1.92;
   1,318 to 1,385 ms
- 1.3.6,
   in the same rounds:
   1.84 to 1.90;
   1,298 to 1,395 ms

The four overlap throughout,
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

Five hundred tokens took 7.7 ms on Node and 1.3.6,
and fewer take less;
the cost per token at that size is about a tenth of the cost per token at 8,000.

## Verified workarounds

### Split a long list of positional tokens across several parses

Eight parses of 1,000 tokens cost about 8 times 24.5 ms,
196 ms,
against 1,332 ms for one parse of 8,000,
by the figures under "Symptom".

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
a run of positional tokens took 0.49 times as long as with the first patch alone at 8,000 tokens (0.33 to 0.59),
0.21 times at 16,000 and 0.18 times at 32,000,
where the median was 1.4 s against 7.3 s,
and 30.6 s on `main`
(Deno 2.9.8,
five rounds paired by round).

Tradeoffs of the second:

- A state of more than 1,024 items is a `Proxy`.
   Printing it shows an empty array,
   `structuredClone()` and `Object.freeze()` on it throw,
   and so does editing it in place,
   as listed under "What a caller can notice".
- Just past that size it may cost up to about a tenth on a line that alternates options and arguments;
   the runs that measured it do not agree.
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
   The other primitives then slice that view through its proxy,
   and a line that alternates options and arguments took 8.8 times as long at 24,000 tokens and 12.0 times at 48,000.
   The figures are under "Prototypes for the two further copies".
- Copying the state of `multiple()` with `slice` and `push` in place of the spread.
   It measured the same as the spread.
   Copying it with `concat` took about a third of the time at 16,000 tokens and left the growth as it was.

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
   1,182.8 ms to 150.7 ms,
   0.12 times as long (0.118 to 0.128).
   Fitted exponent 1.77 to 1.87 before,
   1.18 to 1.27 after.
- A thousand paths through the `git commit` option set:
   75.1 ms to 58.7 ms,
   0.78 (0.73 to 0.82).
- The same after `--`:
   60.1 ms to 46.4 ms,
   0.76 (0.74 to 0.79).

Every round was faster in all three.

Together with upstream's pull request 1121,
same day,
same container limits,
seven rounds with the order of the arms rotated,
each arm from source on Deno except 1.3.6 from npm:
`main` at `84f95710`,
the pull request's head `a175cdb7`,
and each with the patch applied.
Median over the rounds:

- A thousand paths after `--`:
   1.3.6 30.8 ms,
   `main` 56.5 ms,
   pull request 42.0 ms,
   `main` with the patch 44.7 ms,
   pull request with the patch 28.4 ms.
- A thousand paths with no separator:
   1.3.6 133.3 ms,
   `main` 71.0 ms,
   pull request 56.7 ms,
   pull request with the patch 42.9 ms.
- 8,000 positional tokens:
   1.3.6 1,119.8 ms,
   `main` 1,130.4 ms,
   pull request 1,132.5 ms,
   pull request with the patch 134.1 ms.

Paired by round,
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
1.45 by the medians listed here.
The patch removes from the pull request a cost that 1.3.6 also carries and that is about as large as the gap
(42.0 less 28.4 is 13.6 ms,
against a gap of 11.2 ms),
which is why the two land together.
An independent review on 2026-11-13 caught an earlier sentence here,
which called the trace copy what separates the two.
The pull request does not change the 8,000-token figure (1.00 against `main`,
0.82 to 1.04),
and the patch does not depend on it.

The records are in `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`,
folder `measurements-2026-11-13`:
`cross-runtime/results.jsonl` for the three runtimes and the first patched comparison,
`upstream-arms-results.jsonl` for the comparison with the pull request,
and the container recipes and runners that produced them.

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
No person has read the diffs or this section,
and no third model has reviewed either.

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
   It makes a command line that alternates options and arguments about ten times slower once it takes effect,
   and it is not kept as a patch;
   it is in the evidence archive for anyone who wants to repeat the measurement.
- With both copies gone,
   a run of positional arguments parses in time proportional to its length up to 128,000 tokens.
   That is measured for a run of positional arguments and for nothing else.

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
   At 16,000 tokens such a parse took about 300 ms with the state fix against 940 to 1,360 ms without
   (three rounds),
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

#### Timing

Deno 2.9.8,
every arm a source checkout read through an import map,
in a two-CPU container with nothing mounted and no network,
five rounds with the order of the arms rotated.
The arms:
`main`;
base,
which is `main` with the trace patch;
the state fix;
the buffer fix;
both;
and both with the thresholds at zero.

How much two runs of the same code differ sets what can be claimed.
The five rounds carry their own measure of it:
below its threshold an arm runs the very code of the base,
and there the state fix over the base read 1.06 at 1,000 positional tokens with single rounds from 0.56 to 1.53,
and the buffer fix over the base reached 1.42 in one round on alternating input of 12,000 tokens.
Rounds two to five overlapped other containers of the same session,
which is part of that spread.
A later check with nothing else of this session running ran the base twice per round for ten rounds:
the second run over the first had a median between 0.97 and 1.03 at every size,
and single rounds from 0.75 to 1.25.
So in the five rounds a median within a tenth of 1,
or a range that straddles 1,
is no finding in either direction,
and the ten-round check is the finer instrument where it was used.

A run of positional arguments and one unused option (`token-scaling.ts`),
median in milliseconds:

- 2,000 tokens:
   `main` 96.4,
   base 37.1,
   state fix 30.4,
   both 28.3,
   thresholds at zero 27.4
- 4,000:
   343.1,
   70.8,
   43.4,
   42.0,
   53.0
- 8,000:
   1,400.9,
   216.8,
   84.0,
   85.9,
   74.7
- 16,000:
   6,184.3,
   1,029.5,
   198.1,
   197.5,
   168.1
- 32,000:
   30,603.5,
   7,303.4,
   1,365.3,
   694.9,
   320.4

The state fix against the base,
paired by round:
0.49 at 8,000 tokens (0.33 to 0.59),
0.21 at 16,000 (0.17 to 0.32)
and 0.18 at 32,000 (0.12 to 0.22),
slower in none of those fifteen pairs.
At 2,000 and 4,000 it reads 0.82 and 0.78,
which is near the spread of identical code.

Larger sizes were run in a step of their own,
one parse per size in a fresh process,
so its figures are not a continuation of that list:

- State fix:
   3,711 ms at 64,000 tokens and 24,262 ms at 128,000.
- Both:
   784 ms and 2,038 ms.
- Thresholds at zero:
   802 ms and 1,672 ms.
- Base:
   25,912 ms at 64,000.

Growth per doubling,
within a step
(1 is proportional,
2 is the square):

- Base:
   0.93,
   1.61,
   2.25 and 2.83 from 2,000 to 32,000.
- State fix:
   0.51,
   0.95,
   1.24 and 2.78,
   then 2.71 from 64,000 to 128,000.
- Both:
   0.57,
   1.03,
   1.20 and 1.82,
   then 1.38.
- Thresholds at zero:
   0.95,
   0.50,
   1.17 and 0.93,
   then 1.06.

So on this input no further cost that grows with the square shows up to 128,000 tokens once both copies are gone.
The state fix alone leaves the buffer copy,
which takes over from about 32,000 tokens:
both fixes took 0.24 times as long as the state fix alone at 64,000 (0.17 to 0.27)
and 0.084 times at 128,000 (0.078 to 0.109).
The copy that is left does not cost a fixed amount per slot.
The arm with both fixes and the arm with the thresholds at zero differ in the buffer copies of the last 16,384 tokens
(and in the views the second makes on short states and buffers);
that difference is 30 ms in a parse of 16,000 tokens and 375 ms in one of 32,000.
The recheck did not find out why.

Options and arguments alternating for the whole line
(`-v v0 p0 -v v1 p1`,
a script written for the recheck),
median in milliseconds at 6,000,
12,000,
24,000 and 48,000 tokens:

- Base:
   48.8,
   127.7,
   489.9,
   2,651.1
- State fix:
   55.0,
   90.1,
   310.6,
   2,657.9
- Buffer fix:
   47.5,
   124.4,
   4,411.8,
   30,310.6
- Both:
   58.0,
   89.3,
   4,638.3,
   28,734.2

The buffer fix took 8.8 times as long as the base at 24,000 tokens (8.5 to 11.2)
and 12.0 times at 48,000 (9.8 to 13.8),
slower in every round,
and both fixes together 10.5 and 11.2 times.
The cause is in the source.
On `main`,
`buffer.slice(` is called at 41 places outside the tests,
and sixteen of them in `primitives.ts` build the next buffer that way,
[the option parser's among them][option buffer].
The diff changes the two in `argument()`.
Once `argument()` has made the buffer a view,
the next option parser calls `Array.prototype.slice` on it,
which reads every remaining token through the proxy.
With the thresholds at zero the same input took 5.1,
10.1 and 12.9 times as long as the base at 3,000,
6,000 and 12,000 tokens.
The agent did not measure this shape:
its scripts were the positional run and a thousand paths.

The state fix on the same input,
from the ten-round check:
0.98 and 0.99 times the base at 1,500 and 3,000 tokens,
1.11 at 4,500 (0.91 to 1.45,
slower in eight of ten rounds,
where the same-code control reads 1.03),
0.94 at 6,000,
then 0.87,
0.78 and 0.76 at 7,500,
9,000 and 12,000,
slower in none of thirty pairs.
This input fills two states,
which pass 1,024 items at about 3,100 tokens.
A later five-round run at the same sizes read 1.17 at 4,500 for the same arm (0.88 to 1.65)
and 0.99 for the kept patch (0.53 to 1.06).
So just past its threshold the fix may cost up to about a tenth on this shape or nothing;
the runs do not agree,
and whether 1,024 is the right threshold for inputs other than a positional run is not settled.
At 24,000 tokens it reads 0.70 (0.62 to 0.72),
and at 48,000 it gains nothing (0.99).
What takes the time in the base at 48,000 alternating tokens was not profiled.

A parser shaped like `git commit`,
twenty-eight options and the paths (`commit-paths.ts`):

- A thousand paths:
   no arm differs from the base by more than the spread,
   with or without `--`,
   except thresholds-at-zero.
- 20,000 paths:
   the state fix took 0.43 times as long as the base (0.41 to 0.45),
   and 0.41 after `--` (0.31 to 0.74).
   The buffer fix reads 0.97 and 1.08,
   inside the spread.

All of those timings ran on the agent's diffs.
The kept patch differs from the agent's state fix in code that runs only when something writes to a state,
and it was timed beside that arm and the base in five more rounds to make sure:

- Against the base on a positional run:
   0.39 at 8,000 tokens (0.33 to 0.59),
   0.20 at 16,000 (0.15 to 0.28)
   and 0.22 at 32,000 (0.14 to 0.33),
   slower in none of fifteen pairs.
- At 20,000 paths through the `git commit` shape:
   0.42 (0.27 to 0.47).
- Against the agent's arm:
   between 0.93 and 1.16 at every positional size,
   each range straddling 1.

One smaller thing was tried on the side.
Writing the same copy as
`(added ? context.state : context.state.slice(0, -1)).concat([nextItemState])`,
with no spread,
took 0.33 times as long as the base at 16,000 positional tokens (0.28 to 0.44),
0.38 at 32,000 (0.27 to 0.66)
and 0.60 at 20,000 paths (0.49 to 0.62).
It changes nothing a caller can see and upstream's suite passes with it on the three runtimes
(778,
8,086 and 8,086).
It does not change the growth:
2.82 per doubling from 16,000 to 32,000 and 2.29 from 64,000 to 128,000.
It was timed on Deno only,
and below 16,000 tokens its gain is inside the spread.
Copying with `slice` and then `push` gained nothing measurable.

#### Where the agent's report and the recheck differ

- The report gives one fitted exponent for both fixes,
   1.14.
   Per doubling it is the four figures listed under "Timing",
   and 1.82 from 16,000 to 32,000.
- It calls the buffer fix marginal,
   "nothing below its threshold" and 1.25 times at 32,000.
   On a positional run it is worth far more at larger sizes,
   and on alternating input it is a tenfold loss.
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
- Any timing on Node or Bun for these arms,
   and any timing of the kept patch without the trace patch under it
   (it applies to `main` alone,
   and was timed only on top of the trace patch).
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

With one option and a run of positional arguments, the median of five parses on 1.3.6 and Node 26:

- 500 arguments: 7.7 ms
- 2,000 arguments: 84.2 ms
- 8,000 arguments: 1,331.9 ms

The fitted exponent of time against argument count was 1.85 on Node, 1.92 on Deno and 1.63 on Bun for 1.3.6, and
1.75, 1.78 and 1.57 for 1.4.0-dev.2971. `recordTrace()` in *primitives.ts* calls `set()` once per recorded input,
and `set()` does `new Map(this.#entries)`:

https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/input-trace.ts#L140-L144

A script that fits the exponent, and a workflow that runs it on pinned versions:
https://github.com/Aquaticat/optique-parse-cost-reproduction (*token-scaling.ts*).

I have a patch that keeps the entries in an immutable hash trie instead, so a write copies only the path to its
key and earlier traces stay untouched. From source on Deno, 8,000 arguments went from 1,183 ms to 151 ms and the
exponent from about 1.8 to about 1.2; a thousand paths through a `git commit`-shaped parser took 0.78 times as
long. The core suite passes on Deno and Node with it, and it applies cleanly to main as of #1121's merge.

It does not make the parse linear. Two more per-argument copies sit behind this one: `multiple()` rebuilds its
state array for each item (*modifiers.ts*, `annotateFreshArray(context.state, [...])`), and each primitive slices
the rest of the buffer for each token it consumes (`context.buffer.slice(i + 1)` in `argument()`, and fourteen more
places in *primitives.ts*). On Deno, 16,000 arguments take 6.2 s on main and 1.0 s with this patch, 32,000 take
30.6 s and 7.3 s.

I tried both. For the state I have a second patch: above 1,024 items the state becomes an array-shaped view (a
`Proxy`) over an immutable vector, and below that nothing changes. On top of this patch, 16,000 arguments go from
1.0 s to 0.2 s and 32,000 from 7.3 s to 1.4 s, and the core suite passes on Deno, Node and Bun, also with the
threshold lowered to two so that nearly every state is a view. Its cost is that such a state is a proxy: it prints
as an empty array, `structuredClone()` and `Object.freeze()` throw on it, and in-place edits throw. For the buffer
I have nothing to offer: a view cut in `argument()` alone made a line that alternates options and arguments about
ten times slower, because the other primitives then slice the view through the proxy. With the state fixed, the
buffer copy is most of what is left from about 32,000 arguments.

It also bears on what #1121 leaves. Measured the same way, seven rounds with the arms rotated, a thousand paths
after `--` took 1.37 times as long on #1121's branch as on 1.3.6 (1.34 to 1.42), and 0.94 times with this patch
on top (0.86 to 1.07, two of seven rounds slower).

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
[option buffer]: https://github.com/dahlia/optique/blob/d26f1a244d4434f9dbfeb0317f6db32533cb8c32/packages/core/src/primitives.ts#L1251
[policy pull requests]: https://github.com/dahlia/optique/blob/84f9571069584d8e36a3a4cfbaebf8a9a61fc645/AI_POLICY.md?plain=1#L14-L18
[policy issues]: https://github.com/dahlia/optique/blob/84f9571069584d8e36a3a4cfbaebf8a9a61fc645/AI_POLICY.md?plain=1#L25-L27
[no timing]: https://github.com/dahlia/optique/pull/1121#:~:text=without%20timing%20thresholds
