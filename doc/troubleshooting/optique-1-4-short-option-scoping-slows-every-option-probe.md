# `@optique/core` 1.4 pre-releases: short-option scoping adds work to every option probe, so a thousand paths after `--` parse about 1.8 times slower than on 1.3.6

Unapproved and not final.
An AI session wrote this document on 2026-11-12 and extended it on 2026-11-13,
and the owner has not approved it.
Done and checked by that session:
the measurements and the source reading reported here.
Open:
a model from another vendor reviewed the text as of commits `5a4e07d2e3` and `426d33d638`,
and the corrections made after that as of commit `2d06b4ae60`,
and its findings were folded in;
the corrections made after that last pass are not reviewed,
and upstream's pull request 1121 was not merged at this writing.

Status,
2026-11-12:
unreleased upstream.
The cost is on upstream `main` and in the `1.4.0-dev` builds since 2026-09-27;
no stable release has it,
and no package in this repository uses a 1.4 build.
It is written up because upstream's maintainer asked this project to assess `main`,
and because upstream can still change it before 1.4.0.

Status,
2026-11-13:
upstream is fixing it.
The maintainer opened [upstream issue 1120][upstream issue] at 2026-11-13T08:16:37Z,
"Reduce option-scope overhead when probing positional arguments",
which says the regression
"was reported in Monochromatic's research record and short-option scoping investigation,
then reproduced locally",
and links this document at commit `249bbf19ea`.
Pull request 1121 for it was opened at 2026-11-13T10:10:30Z and is open at this writing.
What this project measured of that pull request is under "Upstream's pull request 1121 as measured here".

The measurement behind it is in `doc/research/optique-typo-suggestion-overhead-upstream-record.md`,
section "What the 1.4 line added",
with the raw records in the evidence archive beside that document.

## Symptom

Nothing fails.
A parse that succeeds takes longer on the 1.4 line than on the 1.3 line,
by an amount that grows with tokens times option parsers.

Where suggestion work used to dominate,
the 1.4 line is still faster overall,
because upstream pull request 997 removed that work.
The cost shows where there is no suggestion work to remove:

- A thousand paths after `--`,
   through a parser with the option set of `git commit`:
   `1.4.0-dev.2664` took 1.80 times as long as 1.3.6 (1.36 to 1.91 across seven paired rounds),
   and `1.4.0-dev.2971` took 1.73 times as long (1.51 to 2.12).
   Every round was slower in both comparisons.
- A thousand paths with no separator,
   before pull request 997:
   `1.4.0-dev.2664` took 1.24 times as long as 1.3.2,
   the 1.3 patch level it contains (1.14 to 1.43,
   all seven rounds slower).

## Root cause

### Step 1: every child probe goes through `withChildContext()`, which now builds an option scope

`object()` offers each token to each field parser.
On `main`,
the helper that prepares the context for one such offer ends by calling `withParserOptionScope()`,
[`packages/core/src/constructs.ts` lines 248 to 260][child context]:

```ts
// packages/core/src/constructs.ts at commit 1d6e4594 on main
function withChildContext<TState>(
  context: ParserContext<unknown>,
  segment: PropertyKey,
  state: TState,
  parser?: Parser<Mode, unknown, unknown>,
  usage?: Usage,
): ParserContext<TState> {
  const childState = parser == null
    ? state
    : getParseChildState(context.state, state, parser) as TState;
  const child = withSharedChildContext(context, segment, childState, usage);
  return parser == null ? child : withParserOptionScope(child, parser);
}
```

### Step 2: `withParserOptionScope()` copies the context and the inherited set on each call

[`packages/core/src/short-option.ts` lines 274 to 299][parser scope]:

```ts
// packages/core/src/short-option.ts at commit 1d6e4594 on main
export function withParserOptionScope<S>(
  context: ParserContext<S>,
  parser: Parser<Mode, unknown, unknown>,
  segment?: PropertyKey,
): ParserContext<S> & ScopedContext {
  const inherited = (context as ScopedContext)[optionScopeKey];
  const selections = inherited?.selections ??
    new Map<ScopeSource, ScopeSource>();
  return {
    ...context,
    [optionScopeKey]: {
      names: inherited?.names ?? new Set(),
      sources: new Set([
        ...(inherited?.sources ?? []),
        bindScope(
          parserScope(parser),
          segment == null
            ? scopePath(context)
            : [...scopePath(context), segment],
          selections,
        ),
      ]),
      selections,
    },
  };
}
```

Each call spreads the context into a new object,
builds a new set from the inherited sources,
and resolves a bound scope for the parser.

### Step 3: `forkOptionScope()` copies the selections map where a branch is tried

[`packages/core/src/short-option.ts` lines 302 to 313][fork scope]:

```ts
// packages/core/src/short-option.ts at commit 1d6e4594 on main
export function forkOptionScope<S>(
  context: ParserContext<S>,
): ParserContext<S> & ScopedContext {
  const scoped = (context as ScopedContext)[optionScopeKey];
  if (scoped == null) return context;
  const selections = new Map(scoped.selections);
  scopeBindings.set(selections, getScopeBindings(scoped.selections));
  return {
    ...context,
    [optionScopeKey]: { ...scoped, selections },
  };
}
```

### Where it came from

`packages/core/src/short-option.ts` does not exist at tag 1.3.2.
It was added by commit `fe63bb3f`,
"Support attached short-option values",
on 2026-09-27,
in upstream pull request 981,
and extended by the commits that followed it in the same pull request.
It is the one source file outside tests that the 1.4 line had and 1.3.2 lacked
at the commit `1.4.0-dev.2664` was built from.

### What the profiles attribute

Per parse,
from `node --cpu-prof` on the input with paths after `--`:

- 1.3.6:
   41.2 ms in all;
   no `short-option.js`.
- `1.4.0-dev.2664`:
   72.1 ms in all;
   `short-option.js` 25.9 ms,
   of which `withParserOptionScope` 8.0,
   `forkOptionScope` 6.1 and `withOptionScopeChild` 6.1.
- `1.4.0-dev.2971`:
   86.3 ms in all;
   `short-option.js` 27.0 ms.

Of the 30.8 ms that 1.3.6 to `1.4.0-dev.2664` adds,
`short-option.js` is 25.9 ms.
These totals are not the untimed medians,
from which they differ by a few milliseconds either way;
the differences between versions are what the comparison rests on.

In a sweep over the number of options,
the cost on `1.4.0-dev.2971` grew in a straight line,
about 1.6 to 1.9 microseconds per token and option,
so the scope work is a constant paid on every probe and does not itself grow with the grammar.

### A reading that turned out wrong

An earlier version of the research document said `main`
"had become 24% slower than 1.3.6".
`1.4.0-dev.2664` was built on 2026-09-29 and 1.3.6 was tagged on 2026-10-15,
so that comparison was out of order.
The build contains 1.3.2,
and that is the release to set it beside.

## Verification

### Versions under test

`@optique/core` 1.1.1,
1.3.2,
1.3.6,
`1.4.0-dev.2664` and `1.4.0-dev.2971` from npm,
on Node v26.13.1,
in a container from `docker.io/cachyos/cachyos-v4:latest` run with
`--cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10` and no mounts,
on 2026-11-12.

### Harness

The script embedded in `doc/troubleshooting/optique-eager-typo-suggestions-on-positional-arguments.md`,
with one line changed so that `--` comes before the paths:

```ts
// reproducer.ts, the one changed line
const args = ["--message", "x", "--", ...paths];
```

Unlike this repository's old wrapper,
which cut its argument list at the separator,
the script hands all thousand paths to `parseSync`.

### Input that is slower on the 1.4 line

That script as written,
two rounds,
lowest and highest round median:

- 1.1.1:
   32.9 to 48.4 ms
- 1.3.6:
   38.2 to 47.1 ms
- `1.4.0-dev.2971`:
   73.7 to 80.5 ms

The same shape in the measurement harness,
seven rounds with the versions rotated,
each process's median taken as one observation:

- 1.1.1:
   41.4 ms
- 1.3.6:
   41.3 ms
- `1.4.0-dev.2664`:
   75.1 ms
- `1.4.0-dev.2971`:
   79.4 ms

1.1.1 against 1.3.6 gave 1.05 (0.92 to 1.26),
four rounds one way and three the other:
no difference.
`1.4.0-dev.2664` against `1.4.0-dev.2971` gave 1.06 (0.96 to 1.16),
which seven rounds cannot tell from no difference.

### Input where the cost is hidden

A thousand paths with no separator.
`1.4.0-dev.2971` took 0.51 times as long as 1.3.6 there (0.46 to 0.61),
because the suggestion work pull request 997 removed was larger than the scope work pull request 981 added.

### The three runtimes

On 2026-11-13 the reproduction repository's `commit-paths.ts --separator` was run on three runtimes,
in the same kind of container with no network,
five alternating rounds each.
`1.4.0-dev.2971` against 1.3.6:

- Node 26.13.1:
   1.90 (1.80 to 2.11)
- Deno 2.9.8:
   1.85 (1.77 to 2.00)
- Bun 1.4.4:
   2.61 (2.51 to 2.75)

Every round was slower on every runtime.
Bun's 1.3.6 figure is lower than the others' (21.1 ms against 36.9 on Node),
while its figure for the pre-release is about the same (55.2 against 70.5),
so the scope work costs relatively more there.

### Upstream's pull request 1121 as measured here

Seven rounds with the order of the arms rotated,
on Deno 2.9.8,
1.3.6 from npm and the other arms from source:
`main` at `84f95710` and the pull request's head `a175cdb7`.
A thousand paths after `--`:

- 1.3.6:
   30.8 ms
- `main`:
   56.5 ms,
   1.87 times 1.3.6 (1.79 to 1.92)
- Pull request 1121:
   42.0 ms,
   0.75 times `main` (0.71 to 0.77) and 1.37 times 1.3.6 (1.34 to 1.42)

Every round went the same way in each comparison.
The pull request's own figures,
on Node and another machine,
are 37.0,
77.6 and 54.3 ms,
which is 0.70 times `main` and 1.47 times 1.3.6;
it says
"the separator case remains slower than 1.3.6".
The two measurements agree on the direction and roughly on the size.

With no separator the pull request took 0.80 times as long as `main` (0.79 to 0.81).

What remains after it is still option-scope work,
since 1.3.6 carries every other cost of this parse too.
A separate cost that both carry happens to be about as large:
with the input-trace patch of
`doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.md` on top of the pull request,
the same parse took 0.94 times as long as 1.3.6 (0.86 to 1.07,
two of seven rounds slower).

## Verified workarounds

### Stay on the 1.3 line for inputs with many tokens after `--`

Verified by the figures under "Input that is slower on the 1.4 line".

Tradeoffs:

- The 1.3 line still searches for typo suggestions on every mismatch before a separator,
   which costs more than the scope work does;
   `doc/troubleshooting/optique-eager-typo-suggestions-on-positional-arguments.md` has those figures.
- The 1.3 line has no attached short-option values,
   the feature the scope work exists for.

No way to switch the scope work off inside the 1.4 line was looked for.

## What does not work

- Putting `--` before the positional tokens.
   It avoids the suggestion search and not the scope work,
   which is why this input shows the cost.
- A literal `errors.noMatch`.
   It concerns the suggestion search only.

## Upstream filing artifact

### Upstream filing decision

Decision,
2026-11-13:
no new report.
Upstream issue 1120 is this finding,
opened by the maintainer from this document,
so a report from here would duplicate it.
The record from 2026-11-12 that follows was written before that issue existed and is kept as the audit trail.

What this project has that the issue and its pull request do not
(the three-runtime figures,
a second measurement of the pull request,
and what the input-trace patch does to the remainder)
goes into the reply the maintainer asked for,
drafted in `doc/troubleshooting/optique-eager-typo-suggestions-on-positional-arguments.md`.
A separate comment on issue 1120 would repeat that reply,
so none is drafted.

Decision as it stood on 2026-11-12:
a new report is warranted,
since nothing upstream covers this;
whether and how to send it is the owner's decision.

`.out-of-scope/`,
read 2026-11-12:
`.out-of-scope/jsr.md` mentions `@optique/*` in a list of packages once imported from JSR
and exempts JSR registry defects only,
so no exemption applies.

Duplicate search,
2026-11-12:
`gh issue list` and `gh pr list` on `dahlia/optique`,
all states,
for `"short-option" OR "option scope" OR forkOptionScope OR withParserOptionScope`,
return the feature's own issue 965 and pull request 981 and nothing about its cost.

Work in progress,
2026-11-12:
the one open pull request,
1113,
changes help synopses.
No branch or issue addresses option-scope cost.

The six constraints:

1.  Upstream's fault:
    yes,
    as a cost;
    whether it is acceptable for 1.4.0 is upstream's call.
2.  Upstream can fix it:
    yes.
    The work is in three small functions.
3.  Supported use case:
    yes.
    Upstream issue 989 says
    "a successful parse with many paths should not require a full typo diagnostic for each option non-match".
4.  Contribution welcome:
    the maintainer asked this project to assess
    "against main for the upcoming change".
    Upstream's `AI_POLICY.md` requires AI use to be disclosed
    and the text to be "reviewed and edited by a human before submission".
5.  Likely to fix:
    no signal either way.
6.  Minimal fix prototyped:
    no,
    and none was attempted.
    The feature is six weeks old and its pull request changed 27 files with 2,328 insertions.
    A patch into it from outside,
    without the author's knowledge of which scopes may be shared,
    would be the "hypothetically correct code" upstream's policy rules out,
    and upstream accepts AI-assisted pull requests only for accepted issues.
    The report gives the measurement,
    the profile and the three functions instead.

Risk to the poster:
a report would go out under the owner's account.
The last of the policy's numbered rules reads:
"Violations of this policy may result in being banned from contributing."
The draft was written with AI assistance,
so posting it unreviewed,
unedited or undisclosed would break that policy.
No trust or blocklist file was found in the upstream tree at `1d6e4594`;
check the live repository again on the day of posting.

Readiness check,
2026-11-12:

1.  Latest release and other maintained lines:
    the cost is absent from every release;
    measured on two builds of `main`.
2.  Prior art in the code:
    read pull request 981's description and the three functions.
3.  Intake:
    upstream has no issue templates.
4.  Automation on a new filing:
    not checked for issues;
    pull requests get Codex,
    CodeRabbit and Codecov comments.
5.  Duplicate,
    work-in-progress and trust-list searches:
    run today,
    as recorded in this section.
6.  The draft's reproduction run exactly as written:
    the embedded script with the one changed line was run as written,
    with the figures under "Input that is slower on the 1.4 line".
7.  Reproduction runnable by others on a clean machine through CI:
    a repository is being prepared under the owner's account;
    not done at this writing.
8.  Verdict:
    no recommendation to post yet.
    The blockers are item 7,
    which the agent can clear,
    and the owner's review and edit of the text,
    which upstream's policy requires.

### No draft issue

The draft issue this section held on 2026-11-12 was never posted,
and upstream issue 1120 now says what it said,
with the maintainer's own reproduction.
Nothing is to be filed from it.

[upstream issue]: https://github.com/dahlia/optique/issues/1120#:~:text=was%20reported%20in%20Monochromatic%27s
[child context]: https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/constructs.ts#L248-L260
[parser scope]: https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/short-option.ts#L274-L299
[fork scope]: https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/short-option.ts#L302-L313
