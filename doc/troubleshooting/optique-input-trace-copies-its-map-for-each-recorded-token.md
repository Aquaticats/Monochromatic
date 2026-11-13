# `@optique/core` 1.0.0 to `main`: the input trace copies its whole map for each token it records, so a successful parse takes time close to the square of the positional token count

Unapproved and not final.
An AI session wrote this document on 2026-11-13,
and the owner has not approved it.
Done and checked by that session:
the measurements and the source reading reported here.
Open:
a review by a model from another vendor was still running when this was written,
and the two further copies named under "What is left after the fix" are profiled and not fixed.

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
and what is left is the other two:
the patched build takes about an eighth of the time at 16,000 tokens and about a fifth at 32,000.
The prototype's author,
an AI agent,
had attributed the remainder to the buffer copy alone;
the profile puts more of it in `multiple()`.

Neither of the two was prototyped.
Each changes a value that other code reads as a plain array
(the state of `multiple()`,
and the context's token buffer),
so a fix reaches further than the trace's did.

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
which is the trace's share of that parse.

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

## What does not work

- Putting `--` before the positional tokens,
   as under "Inputs that are slow".
- A literal `errors.noMatch`,
   or a `1.4.0-dev` build.
   Both concern the typo-suggestion work,
   which is a different cost.
- Waiting for upstream pull request 1121.
   It changes the option-scope work and does not touch `input-trace.ts`.

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

The six constraints:

1.  Upstream's fault:
    yes,
    as a cost.
2.  Upstream can fix it:
    yes.
    The prototype changes one class and adds one internal module.
3.  Supported use case:
    yes.
    Upstream issue 989 set out to reduce "parsing time for commands with many positional arguments",
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
Upstream's policy ends:
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

Correctness,
run in a container with upstream's toolchain on 2026-11-13:

- `deno fmt --check`,
   `deno lint` and `deno check` on the four touched source files:
   clean.
- The two new test files on Deno:
   2 passed (32 steps),
   0 failed.
   On Bun 1.3.0:
   32 pass,
   0 fail.
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
- `git apply` of the patch onto `main` at `84f95710` and onto pull request 1121's head `a175cdb7`:
   clean on both.

Upstream's core suite on Bun 1.3.0,
run on 2026-11-13 on an unpatched worktree of `1d6e4594`,
then on the patched tree,
then on the unpatched one again:

- Unpatched,
   both runs:
   7,668 pass,
   157 fail,
   and two test files that did not load
   (`facade.test.ts` and `short-options.test.ts`,
   "Cannot find module '@optique/env'",
   a workspace package that was not built in that fresh worktree).
- Patched:
   8,457 pass,
   155 fail,
   every file loaded.
- By test name,
   the same 154 tests fail in all three runs.
   No test fails only with the patch.
   Every failure with the patch is in the suites "completion module" and "joined Bash option completion".

So the patch adds no Bun failure among the tests that ran both ways.
The tests of the two files that did not load without the patch ran only with it,
and none of them failed there.
Why the shell-completion tests fail in this container was not looked into.

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

So on this input the trace copy is about what still separates the pull request from 1.3.6.
The pull request does not change the 8,000-token figure (1.00 against `main`,
0.82 to 1.04),
and the patch does not depend on it.

The records are in `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`,
folder `measurements-2026-11-13`:
`cross-runtime/results.jsonl` for the three runtimes and the first patched comparison,
`upstream-arms-results.jsonl` for the comparison with the pull request,
and the container recipes and runners that produced them.

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
    tags and manual dispatch,
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
long. The core suite passes on Deno and Node with it, and it applies cleanly to main and to #1121's branch.

It does not make the parse linear. A profile of the patched build shows two more per-argument copies behind this
one: `multiple()` rebuilds its state array for each item (*modifiers.ts*, `annotateFreshArray(context.state, [...])`)
and `argument()` slices the rest of the buffer for each token (`context.buffer.slice(i + 1)`). They are what is
left: 16,000 arguments take 6.7 s unpatched and 0.8 s patched, 32,000 take 29.2 s and 5.7 s. I have not tried to
change either.

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
[policy pull requests]: https://github.com/dahlia/optique/blob/84f9571069584d8e36a3a4cfbaebf8a9a61fc645/AI_POLICY.md?plain=1#L14-L18
[policy issues]: https://github.com/dahlia/optique/blob/84f9571069584d8e36a3a4cfbaebf8a9a61fc645/AI_POLICY.md?plain=1#L25-L27
[no timing]: https://github.com/dahlia/optique/pull/1121#:~:text=without%20timing%20thresholds
