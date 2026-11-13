# `@optique/core` 0.7.0 to 1.3.1: each positional token builds a typo suggestion in every option parser that rejects it, so a successful parse of many paths takes seconds

Unapproved and not final.
An AI session wrote this document on 2026-11-12 and rewrote its reply draft on 2026-11-13,
and the owner has not approved it.
Done and checked by that session:
the measurements and the source reading reported here,
and one review by a model from another vendor of the text as it stood on 2026-11-12.
Open:
a second such review read the text as of commits `5a4e07d2e3` and `426d33d638`,
and the corrections made after that as of commit `2d06b4ae60`,
and its findings were folded in;
the corrections made after that last pass are not reviewed,
and the reply draft waits for the owner.
On 2026-11-13 the owner pointed out that this machine is shared and no timing stands on its own,
and the timing figures here,
the reply draft's among them,
were rewritten as ratios between versions run in the same round,
with absolute times kept only as scale;
that rewrite is not reviewed either.

Status,
2026-11-12:
upstream fixed this in two steps.
`@optique/core` 1.3.2 removed most of the cost,
and a change merged for 1.4.0 removes the rest of the suggestion work.
No package in this repository can meet the slowest form any more:
`pnpm-lock.yaml` resolves `@optique/core` 1.3.6 and no other version.
The consumer where the cost was measured,
the TypeScript Git policy wrapper,
was deleted on 2026-10-13 (`642b6d72cb`).
Whether another consumer passes many positional tokens before a `--` was not checked.
This document stays as the record of the cause,
the measurements,
and what works around it on each version.

How the cost entered upstream,
how upstream handled it,
and the full measurement are in `doc/research/optique-typo-suggestion-overhead-upstream-record.md`.

## Symptom

Nothing fails.
A parse that succeeds is slow,
and the time grows with the number of positional tokens.

The reported case is this repository's commit `4879a44e6a2460ab3c2531744e24a1f2aef27aeb`,
on `@optique/core` 1.1.1:
"A commit naming a thousand paths spent 2.56s of 4.24s there".

The trigger is an `object()` that holds `option()` or `flag()` parsers beside `multiple(argument(...))`,
given positional tokens before any `--`.
Three variants exist by version:

- 0.7.0 to 1.3.1:
   seconds for a thousand paths,
   growing with the length of each token;
   nearly all of it in `levenshteinDistance`.
- 1.3.2 to 1.3.6:
   about a fifteenth of the time 1.3.1 takes on a thousand real paths,
   and the same whatever the length of a token;
   about three quarters of it in collecting option names and searching them.
- `1.4.0-dev` builds from `1.4.0-dev.2665` on:
   about half the time of 1.3.6 again;
   no suggestion code runs.

Those ratios are from seven rounds with the versions paired by round,
under "Inputs that are slow".
For scale only,
on the machine used they came to seconds for a thousand paths on the first group,
and about 0.18 ms and 0.09 ms per positional token on the other two.

## Root cause

### Step 1: `object()` offers every token to every field parser

`object()` sorts its field parsers by priority and,
for each token,
tries them in that order until one consumes it.
Option parsers have a higher priority than positional ones,
so a path is offered to every option parser before `argument()` sees it.
[`packages/core/src/constructs.ts` lines 5869 to 5884 at 1.1.1][object loop]:

```ts
// packages/core/src/constructs.ts at tag 1.1.1
    while (madeProgress && currentContext.buffer.length > 0) {
      madeProgress = false;
      const getFieldState = createFieldStateGetter(
        currentContext.state,
        getObjectParseChildState,
      );

      for (const [field, parser] of parserPairs) {
        const result = (parser as Parser<"sync", unknown, unknown>).parse(
          withChildContext(
            currentContext,
            field,
            getFieldState(field, parser),
            parser,
          ),
        );
```

### Step 2: an option parser that does not match builds its error message at once

The no-match path of `option()` returns a failure whose `error` is already computed.
With a custom `errors.noMatch` it first collects every option name and searches them,
then calls or returns the custom value;
without one it calls `createErrorWithSuggestions()`,
which does the same collecting and searching.
[`packages/core/src/primitives.ts` lines 1232 to 1271 at 1.1.1][no-match 1.1.1]:

```ts
// packages/core/src/primitives.ts at tag 1.1.1
      // Find similar options from context usage and suggest them
      const invalidOption = context.buffer[0];

      // Check if custom noMatch error is provided
      if (options.errors?.noMatch) {
        const candidates = new Set<string>();
        for (const name of extractOptionNames(context.usage)) {
          candidates.add(name);
        }
        const suggestions = findSimilar(
          invalidOption,
          candidates,
          DEFAULT_FIND_SIMILAR_OPTIONS,
        );

        const errorMessage = typeof options.errors.noMatch === "function"
          ? options.errors.noMatch(invalidOption, suggestions)
          : options.errors.noMatch;

        return {
          success: false,
          consumed: 0,
          error: errorMessage,
        };
      }

      const baseError = message`No matched option for ${
        eOptionName(invalidOption)
      }.`;

      return {
        success: false,
        consumed: 0,
        error: createErrorWithSuggestions(
          baseError,
          invalidOption,
          context.usage,
          "option",
        ),
      };
```

`flag()` has the same path.
When `argument()` then consumes the token,
the failure and its message are thrown away.

### Step 3: the search computes a full edit distance against every option name

[`packages/core/src/suggestion.ts` lines 165 to 186 at 1.1.1][search 1.1.1]:

```ts
// packages/core/src/suggestion.ts at tag 1.1.1
  for (const candidate of candidates) {
    // Normalize candidate for comparison
    const normalizedCandidate = caseSensitive
      ? candidate
      : candidate.toLowerCase();

    // Skip exact duplicate candidates
    if (seen.has(candidate)) continue;
    seen.add(candidate);

    // Calculate distance
    const distance = levenshteinDistance(normalizedInput, normalizedCandidate);

    // Early termination for exact match
    if (distance === 0) {
      return [candidate];
    }
```

`levenshteinDistance()` does work in proportion to token length times name length.
So one parse does work in proportion to
tokens times option parsers times option names times token length.
A profile of a thousand-path parse on 1.1.1 put 93.1% of the parse in `levenshteinDistance`.

### Step 4: 1.3.2 skips most of the distance work, and all of it for a literal message

Upstream pull request 992 added two things.
A literal `errors.noMatch` returns before any collecting or searching,
[`packages/core/src/primitives.ts` lines 1302 to 1347 at 1.3.6][no-match 1.3.6]:

```ts
// packages/core/src/primitives.ts at tag 1.3.6
      if (options.errors?.noMatch) {
        // Literal messages do not use suggestions.
        if (typeof options.errors.noMatch !== "function") {
          return {
            success: false,
            consumed: 0,
            error: options.errors.noMatch,
          };
        }
```

and the search rejects a candidate by length before computing a distance,
[`packages/core/src/suggestion.ts` lines 184 to 192 at 1.3.6][search 1.3.6]:

```ts
// packages/core/src/suggestion.ts at tag 1.3.6
    const minimumDistance = Math.abs(
      normalizedInput.length - normalizedCandidate.length,
    );
    if (
      minimumDistance > maxDistance ||
      minimumDistance / input.length > maxDistanceRatio
    ) continue;

    const distance = levenshteinDistance(normalizedInput, normalizedCandidate);
```

The default path and the callback path still collect and search on every mismatch.
On 1.3.6 the profile put 46.6% of the parse in `suggestion.js` and 27.3% in `usage.js`,
with `levenshteinDistance` itself at 5.6%.

### Step 5: 1.4.0 builds the message only when something reads it

Upstream pull request 997 makes a failure's `error` a getter that runs its builder on first read,
[`packages/core/src/internal/failure.ts` lines 16 to 39 on `main`][deferred failure]:

```ts
// packages/core/src/internal/failure.ts at commit 1d6e4594 on main
export function createDeferredFailure(
  consumed: number,
  createError: () => Message,
): DeferredFailure {
  let settled:
    | { readonly kind: "value"; readonly value: Message }
    | { readonly kind: "throw"; readonly error: unknown }
    | undefined;
  return {
    success: false,
    consumed,
    get error(): Message {
      if (settled == null) {
        try {
          settled = { kind: "value", value: createError() };
```

A discarded failure is never read,
so its message is never built.
Step 1 is unchanged:
every token is still offered to every option parser,
and the profile of `1.4.0-dev.2971` shows what that costs with the suggestion work gone:
`short-option.js` 31.3%,
`input-trace.js` 23.3%,
`constructs.js` 14.2%,
`failure.js` 8.3%.

### Two readings that turned out wrong

- The design note for the feature,
   upstream issue 38,
   estimated "less than 1ms" for "a typical CLI with 50 options".
   It counted one search per error on an input shaped like an option name.
   Steps 1 and 2 show the search running once per token per option parser on parses that succeed.
- Upstream pull request 992 says
   "Short-path default/callback runs showed no improvement."
   On this repository's parser,
   a thousand tokens of one to three characters took 0.62 times as long on 1.3.2 as on 1.3.1,
   in one comparison of one run each.
   The length check in step 4 rejects this parser's long option names against a short token;
   that explanation was read from the code and not tested on its own.

## Verification

### Versions under test

`@optique/core` from npm,
by tag and upstream commit where a tag exists:

- 1.1.1:
   `b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5`
- 1.3.1,
   1.3.2
- 1.3.6:
   `d969c9f6014bb0dee510fb407efe3806d2636dcf`
- `1.4.0-dev.2664`,
   built from `6f258296`,
   the commit before pull request 997
- `1.4.0-dev.2971`,
   built from `1d6e4594`,
   upstream `main` on 2026-11-12

Node v26.13.1,
in a container from `docker.io/cachyos/cachyos-v4:latest` run with
`--cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10` and no mounts,
on 2026-11-12.

### Harness

A standalone script that needs only `@optique/core` beside it.
Its parser has the shape of the old commit parser:
the same Git options and flags,
without the wrapper's own `--no-enforce-only`,
then `multiple(argument(string()))`
and `passThrough()`.

```ts
// reproducer.ts
// Reproducer: a successful parse of many positional paths through a parser shaped like
// a `git commit` option set. Needs only @optique/core beside it.
// Run: node reproducer.ts        (Node 24 or newer runs TypeScript directly)

import { object } from "@optique/core/constructs";
import { multiple, optional } from "@optique/core/modifiers";
import { parseSync } from "@optique/core/parser";
import { argument, flag, option, passThrough } from "@optique/core/primitives";
import { string } from "@optique/core/valueparser";

// Git accepts any unambiguous prefix of a long option, so each flag declares them all.
function prefixes(name: string, shortestStem: number): [any, ...any[]] {
  const stem = name.slice(2);
  const names = [];
  for (let length = shortestStem; length <= stem.length; length += 1) names.push(`--${stem.slice(0, length)}`);
  return names as [any, ...any[]];
}

const parser = object({
  all: multiple(flag("-a", "--all")),
  only: multiple(flag("-o", "--only")),
  noOnly: multiple(flag("--no-only")),
  include: multiple(flag("-i", ...prefixes("--include", 3))),
  interactive: multiple(flag("--interactive")),
  patch: multiple(flag("-p", "--patch")),
  amend: multiple(flag("--amend")),
  allowEmpty: multiple(flag("--allow-empty")),
  dryRun: multiple(flag(...prefixes("--dry-run", 2))),
  short: multiple(flag(...prefixes("--short", 2))),
  porcelain: multiple(flag(...prefixes("--porcelain", 3))),
  long: multiple(flag(...prefixes("--long", 1))),
  nul: multiple(flag("-z", ...prefixes("--null", 2))),
  message: optional(option("-m", "--message", string())),
  file: optional(option("-F", "--file", string())),
  reuseMessage: optional(option("-C", "--reuse-message", string())),
  reeditMessage: optional(option("-c", "--reedit-message", string())),
  squash: optional(option("--squash", string())),
  fixup: optional(option("--fixup", string())),
  author: optional(option("--author", string())),
  date: optional(option("--date", string())),
  cleanup: optional(option("--cleanup", string())),
  trailer: multiple(option("--trailer", string())),
  template: optional(option("-t", "--template", string())),
  unified: optional(option("-U", "--unified", string())),
  interHunkContext: optional(option("--inter-hunk-context", string())),
  pathspecFromFile: optional(option("--pathspec-from-file", string())),
  pathspecFileNul: multiple(flag("--pathspec-file-nul")),
  paths: multiple(argument(string())),
  unknown: passThrough({ format: "nextToken" }),
});

const pathCount = 1000;
const paths = Array.from(
  { length: pathCount },
  (_, index) => `package/group-${index % 37}/package-${index}/src/feature-${index}.unit.test.ts`,
);
const args = ["--message", "x", ...paths];

const warmupParses = 2;
const timedParses = 7;
for (let index = 0; index < warmupParses; index += 1) parseSync(parser, args);

const samples: number[] = [];
let parsedPaths = -1;
for (let index = 0; index < timedParses; index += 1) {
  const startedAt = performance.now();
  const result = parseSync(parser, args);
  samples.push(performance.now() - startedAt);
  parsedPaths = result.success ? result.value.paths.length : -1;
}
samples.sort((left, right) => left - right);

console.log(JSON.stringify({
  paths: pathCount,
  parsedPaths,
  medianMs: Number(samples[Math.floor(samples.length / 2)]!.toFixed(1)),
  minMs: Number(samples[0]!.toFixed(1)),
}));
```

Each run was a fresh process,
with the versions alternating inside each round.
Every run parsed all thousand paths.
These runs did not record the host's load,
and the rounds differ from one another by more than the careful measurement in the research document does;
the figures here are the lowest and highest round medians.

### Inputs that are slow

Paths before any `--`.
The finding is the ratio between versions,
from the measurement in the research document:
this repository's old parser,
seven rounds,
each version's process paired with the other's in the same round,
later version over earlier:

- 1.1.1 to 1.3.1:
   1.00 (0.83 to 1.04),
   four rounds one way and three the other:
   no difference.
- 1.3.1 to 1.3.2:
   0.065 (0.061 to 0.070),
   faster in all seven.
- 1.3.2 to 1.3.6:
   1.04 (0.97 to 1.20),
   no difference.
- 1.3.2 to `1.4.0-dev.2664`:
   1.24 (1.14 to 1.43),
   slower in all seven.
- 1.3.6 to `1.4.0-dev.2971`:
   0.51 (0.46 to 0.61),
   faster in all seven.

The script of this document,
over four rounds,
went the same way in each.
Its round medians,
for scale only:
2.6 to 4.2 s on 1.1.1 and 1.3.1,
0.17 to 0.30 s on 1.3.2,
1.3.6 and `1.4.0-dev.2664`,
and 0.09 to 0.13 s on `1.4.0-dev.2971`.
The four for 1.3.6 were 169.6,
175.4,
178.9 and 256.5 ms,
which shows how far one loaded round can sit from the others
and why no single time is a finding.

### Inputs that stay fast

The same script with `"--"` placed before the paths,
over two rounds.
On 1.1.1 it took about a hundredth of the time it took without the separator
(between 0.008 and 0.018,
from the extremes of the round medians of both).
For scale only,
the round medians were 33 to 48 ms on 1.1.1,
38 to 47 ms on 1.3.6 and 74 to 81 ms on `1.4.0-dev.2971`.

After `--` an option parser returns a fixed failure before it looks at the token,
so the search in step 2 is never reached.
[`packages/core/src/primitives.ts` lines 987 to 993 at 1.1.1][after separator]:

```ts
// packages/core/src/primitives.ts at tag 1.1.1
      if (context.optionsTerminated) {
        return {
          success: false,
          consumed: 0,
          error: options.errors?.optionsTerminated ??
            message`No more options can be parsed.`,
        };
```

On this input the pre-release took 1.6 to 2.2 times as long as 1.1.1 and 1.3.6,
taking each round by itself.
Two rounds are too few to call that a regression;
the seven-round measurement that does is in
`doc/troubleshooting/optique-1-4-short-option-scoping-slows-every-option-probe.md`,
1.73 times 1.3.6 (1.51 to 2.12).

## Verified workarounds

### Upgrade to 1.3.2 or later

Verified by the figures under "Inputs that are slow".

Tradeoffs:

- The same release changes two parsing behaviors,
   upstream issues 990 and 991.
   An undeclared `--name=value` token before `--` now goes to `passThrough()` and is no longer a positional argument,
   and a known value option with its value missing now fails the parse where it used to succeed.
- About a fifteenth of the earlier time remains,
   and it still grows with tokens times options.

### Put `--` before the positional tokens

Verified on 1.1.1 by the figures under "Inputs that stay fast":
about a hundredth of the time for a thousand paths.

Tradeoffs:

- Every caller has to write the separator.
- Nothing after it can be an option,
   so a flag placed after the paths becomes a path.

### On 1.3.2 or later, give every option and flag a literal `errors.noMatch`

The script with `{ errors: { noMatch: message`No match.` } }` passed to every `option()` and `flag()`,
over two rounds,
took about a quarter of the same version's time without it on 1.3.2 and on 1.3.6
(between 0.16 and 0.31,
from the extremes of the round medians of both),
and less than the pre-release.
For scale only,
its round medians were 46 to 52 ms on 1.3.2 and 45 to 48 ms on 1.3.6.

Tradeoffs:

- Those parsers no longer suggest a correction for a real typo.
- It has to be repeated on every `option()` and `flag()`;
   one that lacks it still searches.
- The message must be a literal.
   A function is still called on every mismatch,
   after the search.

### Use a `1.4.0-dev` build

Verified by the `1.4.0-dev.2971` figures.

Tradeoffs:

- It is a pre-release published from every push to any branch that is not a tag,
   and upstream's README and `docs/` say nothing about installing or supporting these builds.
- Custom mismatch callbacks run only when their message is read,
   so a callback used for a side effect may run fewer times.

### Replace the parser

What `package/git-policy/cli/` did on 2026-07-15.

Tradeoffs:

- The replacement was a different parser with its own defects.
   The owner,
   on 2026-11-12:
   "The ts repo parser for git wrapper had many bugs."
- Usage text and suggestions have to be written by hand.

## What does not work

- A literal `errors.noMatch` on 1.3.1 or earlier.
   The search runs before the literal is returned (step 2).
   Measured over two rounds with the literal on every option and flag:
   the round medians on 1.1.1 and on 1.3.1 were no lower than without it by more than the rounds differ among themselves
   (for scale only,
   2.6 to 3.2 s with it and 2.6 to 4.2 s without).
- A function-valued `errors.noMatch` on any version before 1.4.0.
   Step 2 and step 4 show the search running before the call.
   This was read from the source and not timed.
- Shorter paths on 1.3.1 or earlier.
   A thousand tokens of one to three characters still took 1.6 times as long on 1.3.1 as on 1.3.2,
   in one comparison of one run each.

## Upstream filing artifact

### Upstream filing decision

Decision:
no new issue.
The contribution,
if the owner wants one,
is a reply to the maintainer's own question,
and whether to post it is the owner's decision.

`.out-of-scope/`,
read 2026-11-12:
the one mention of Optique there is in `.out-of-scope/jsr.md`,
which lists `@optique/*` among packages once imported from JSR.
That file exempts JSR registry defects,
not how Optique parses,
so no exemption applies.

Duplicate search,
2026-11-12:
`gh issue list --repo dahlia/optique --state all --search 'performance OR slow OR benchmark OR levenshtein OR overhead OR latency in:title,body'`
returns 51 issues,
with upstream issues 988 and 989 first.
Both are closed.
They describe this cost,
cite this repository's commits as their source,
and were fixed by pull requests 992 and 997.
A new issue would duplicate them.

Work in progress,
2026-11-12:
pull request 997 is merged on `main`;
milestone "Optique 1.4" has two open issues,
neither about parsing cost;
no open pull request touches the suggestion path.

The six constraints:

1.  Upstream's fault:
    yes.
    Upstream labelled issue 988 a bug.
2.  Upstream can fix it:
    yes,
    and did.
3.  Supported use case:
    yes.
    Issue 989 names a successful parse with many paths as its goal.
4.  Contribution welcome:
    conditionally.
    The maintainer asked this project directly:
    "If you've already reproduced similar overhead on v1.3.6,
    I'd appreciate a reproducer so I can investigate."
    Upstream's `AI_POLICY.md`,
    at commit `1d6e4594`,
    governs how:
    "All AI usage in any form must be disclosed",
    and issues and discussions
    "must have a full human-in-the-loop.
    This means that any content generated with AI must have been reviewed and edited by a human before submission."
5.  Likely to fix:
    yes for the suggestion cost,
    which is fixed.
    For the cost that remains in step 1,
    no signal either way.
6.  Minimal fix prototyped:
    not applicable to the suggestion cost,
    whose fix is upstream's own.
    For the remaining per-token cost no prototype was written.
    Whether trying every option parser for every token is a defect or the design is upstream's call,
    so it is put as a question in the draft and not filed as a bug;
    and the request that led to this document was an investigation,
    not a fix.

Risk to the poster:
the reply would go out under the owner's account.
The last of the policy's numbered rules reads:
"Violations of this policy may result in being banned from contributing."
The draft and its numbers were produced with AI assistance,
so posting it unreviewed,
unedited or undisclosed would break that policy.
A search of the upstream tree at `1d6e4594` for a trust or blocklist file
(names matching `vouch`,
and the contents of `.github/`,
which holds only workflows) found none;
check the live repository again on the day of posting.

Readiness check,
2026-11-12:

1.  Latest release and other maintained lines:
    reproduced on 1.3.6,
    the latest release,
    and on 1.1.1,
    1.3.1 and 1.3.2.
    The 1.0 and 1.2 patch lines were not run.
2.  Prior art in the code:
    read issues 38,
    988 and 989 and pull requests 992 and 997 in full.
3.  Intake:
    upstream has no issue templates (`.github/` holds only workflows).
    `CONTRIBUTING.md` points to `AI_POLICY.md`.
    The reply would be a comment on this repository's own commit,
    where the maintainer asked.
4.  Automation on a new filing:
    not checked for commit comments on this repository;
    upstream's pull requests get Codex,
    CodeRabbit and Codecov comments.
5.  Duplicate,
    work-in-progress and trust-list searches:
    run today,
    as recorded in this section.
6.  The draft's reproduction run exactly as written:
    yes,
    the figures under "Verification" come from that script unchanged.
7.  Reproduction runnable by others on a clean machine through CI:
    not done on 2026-11-12.
    Done since:
    `Aquaticat/optique-parse-cost-reproduction`,
    private,
    with three workflow runs that fail on their asserting steps;
    the scripts were also run on Node,
    Deno and Bun on 2026-11-13.
8.  Verdict:
    no recommendation to post.
    The blocker is one only the owner can clear:
    upstream's policy requires a human to review and edit the text first.

### Draft reply (do not post as-is)

Not fileable as written:
it needs the owner's review,
edit and AI-assistance disclosure,
per upstream's policy.
An AI session wrote every line of it and no person has reviewed it;
it speaks for nobody until the owner makes it theirs.

It answers both things the maintainer asked:
a reassessment on 1.3.6 and on `main`,
and a reproducer.
The reassessment itself is the owner's to state,
so that line is a placeholder.

Rewritten 2026-11-13T09:36:05-05:00,
because the thread moved:

- The owner answered on the commit at 2026-11-12T13:52:56Z:
   "Looking.
   ETA:
   In 24h with 90% certainty."
   The owner followed it at 2026-11-13T14:27:49Z with
   "Sorry,
   I had some issues with my machine.
   New ETA:
   in 72h with 80% certainty.",
   which runs to 2026-11-16T14:27:49Z.
- The maintainer read this repository's documents without waiting,
   opened upstream issue 1120 from them at 2026-11-13T08:16:37Z,
   opened pull request 1121 for it two hours later,
   and merged it at 2026-11-13T15:37:30Z as commit `d26f1a24`,
   whose tree is that of the branch head `a175cdb7` measured here.
   So the draft no longer reports the short-option cost as news;
   it reports a measurement of the pull request.
- The input-trace cost has its own document and a checked patch,
   `doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.md`.
- Revised 2026-11-13T13:57:35-05:00:
   fixes for the two copies behind the trace's were prototyped and rechecked that day,
   as that document records under "Prototypes for the two further copies".
   The draft now says what came of each.
   Whether the second patch is offered to upstream at all is the owner's choice,
   and the draft's sentences about it are there to be cut.

Decided by the owner on 2026-11-13,
in answer to questions put that morning:

- On when to answer:
   "Wait for the review",
   and
   "I want to have everything figured out ourselves before giving replies."
   So the draft is not posted until the independent review is folded in and the open points are settled.
- On the input-trace patch:
   "Keep it local for now."
   and,
   when an agent took that to mean the draft should leave the patch out:
   "I only said to "keep it **local** for **now**",
   which means it can be in the draft."
   So the draft carries the patch paragraph,
   and nothing about the patch is filed or posted until the owner posts the reply.

~~~md
Thanks for #992 and #997, and for #1120 and #1121: you got to the short-option cost from the notes here before I
had sent you any of it.

[Owner: your answer to "would you consider reassessing". What the record here shows: performance was the reason for
one parser only, the Git wrapper's, which has since been rewritten; the rest of the migration rests on the grammar
and Valibot reasons you noticed.]

The numbers behind that. The parser this commit removed, unchanged from its parent, one parse of
`['--message', 'x', ...1000 paths]`, on Node 26 in a two-CPU container. The machine is shared, so I give ratios
between versions run in the same round, seven rounds, with the range of the single rounds:

- 1.3.2 over 1.3.1: 0.065 (0.061 to 0.070), faster in all seven
- 1.4.0-dev.2971 over 1.3.6: 0.51 (0.46 to 0.61), faster in all seven
- 1.1.1 against 1.3.1, and 1.3.2 against 1.3.6: no difference

So #992 removed about 93% of it and #997 about half of what was left. Counting mismatches with a function-valued
`errors.noMatch`, a successful parse makes exactly tokens times options of them on 1.3.6 and none on main.

On #1121: I measured its branch at a175cdb7, the same tree as its merge, from source on Deno only, seven rounds with
the arms rotated, a thousand paths after `--`. It took 0.75 times as long as main before the merge (0.71 to 0.77)
and 1.37 times as long as 1.3.6 (1.34 to 1.42), which agrees with your own figures. On the published builds,
1.4.0-dev.2971 against 1.3.6 was 1.90 on Node, 1.85 on Deno and 2.61 on Bun.

What #1121 leaves against 1.3.6 is still scope work, since 1.3.6 pays for everything else in this parse too. A cost
both carry is about as large: the `InputTraceImpl.set()` copy you set aside in #1120. With a patch for that on top
of #1121 the same parse took 0.94 times as long as 1.3.6 (0.86 to 1.07, two of seven rounds slower). On its own
input, 8,000 positional arguments and one option, the patched build takes 0.11 times as long as main on Node (0.10
to 0.12) and 0.30 times on Bun (0.15 to 0.33). It keeps the entries in
an immutable hash trie in place of the copied `Map`; the core suite passes with it on Deno and Node, and it applies
to main as of that merge. It does not make the parse linear: two more per-argument copies sit behind it, in
`multiple()` (the state array) and in every primitive that consumes a token (the buffer slice). I tried both. A
second patch removes the state copy by making a state of more than 1,024 items an array-shaped view over an
immutable vector: on top of the first patch, 32,000 arguments take 0.16 times as long on Node and 0.43 times on Bun
(built packages, five rounds with the arms taking turns), and the core suite passes on Deno, Node and Bun. Its cost
is that such a state is a `Proxy`, which prints as an empty array and throws on `structuredClone()`,
`Object.freeze()` and in-place edits. For the buffer I have nothing to offer: with the second patch in place, also
cutting a view in `argument()` made a line that alternates options and arguments 13 to 18 times slower on Node and
48 to 105 times slower on Bun.
The write-up and both patches: [commit permalink to
doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.md in Aquaticats/Monochromatic].

Both patches were written by an AI agent, so per AI_POLICY.md there is no pull request. Would you take an issue for
the first, and is a proxy-backed state something you would want at all?

One thing that is still true of 1.3.6: the default path collects the option names and searches them on every
mismatch, and a profile puts about three quarters of the parse there. The length check skips a name only when its
length differs from the token's by more than three or by more than half the token's length, so a token about as
long as the option names pays in full. A literal `errors.noMatch` on every option avoids all of it.

The reproducer you asked for: three scripts that run unchanged on Node, Deno and Bun, against an npm version or a
checkout, and a workflow that runs them on pinned versions:
https://github.com/Aquaticat/optique-parse-cost-reproduction

One question: after #997 every token is still offered to every option parser, a constant cost per token and option.
Is that the intended end state?

For context: the parser we replaced Optique with here was a different parser with bugs of its own and has since been
deleted, so our old 4.24 s and 0.89 s compared two different things.

[Owner: add, in your own words, the disclosure of AI assistance that upstream's policy asks for, and anything you
want to say about the cost not having been reported upstream in July.]

[Owner: whether to tell dahlia that the research record #1120 cites was marked "Status: complete" by an AI session
without your approval, was still changing that day, and now says so.]
~~~

The repository the draft links was created private on 2026-11-12T21:02:14Z
and made public at 2026-11-13T14:41:43Z,
on the owner's answer "Make public as it is".
"As it is" refers to its first commit,
which GitHub made when the repository was created and which carries the account's own address
where the two later commits carry its noreply address;
the owner was told and chose not to rewrite it.
Three of its workflow runs fail on their asserting steps,
as its README says they should
(`43306241322`,
`43306311950` and `43443075690`).

The draft's link to the input-trace write-up is a placeholder:
put a commit permalink there on the day of posting.

[object loop]: https://github.com/dahlia/optique/blob/b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5/packages/core/src/constructs.ts#L5869-L5884
[no-match 1.1.1]: https://github.com/dahlia/optique/blob/b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5/packages/core/src/primitives.ts#L1232-L1271
[search 1.1.1]: https://github.com/dahlia/optique/blob/b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5/packages/core/src/suggestion.ts#L165-L186
[after separator]: https://github.com/dahlia/optique/blob/b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5/packages/core/src/primitives.ts#L987-L993
[no-match 1.3.6]: https://github.com/dahlia/optique/blob/d969c9f6014bb0dee510fb407efe3806d2636dcf/packages/core/src/primitives.ts#L1302-L1347
[search 1.3.6]: https://github.com/dahlia/optique/blob/d969c9f6014bb0dee510fb407efe3806d2636dcf/packages/core/src/suggestion.ts#L184-L192
[deferred failure]: https://github.com/dahlia/optique/blob/1d6e459442e92f83006a7c34e4f11f5018c9ab39/packages/core/src/internal/failure.ts#L16-L39
