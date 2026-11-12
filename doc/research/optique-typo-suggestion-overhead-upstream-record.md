# Optique typo-suggestion overhead: the upstream record

## Research status

- Status:
   complete
- Source reading recorded:
   2026-11-12T09:11:13-05:00
- Measurement recorded:
   2026-11-12T10:05:51-05:00
- Scope:
   how the eager "did you mean" cost entered `@optique/core`,
   how upstream handled it,
   and what upstream has released
- Product changes:
   none;
   this is an investigation

## Trigger

Optique's maintainer commented on commit `4879a44e6a2460ab3c2531744e24a1f2aef27aeb` of this repository
on 2026-11-12T03:14:38Z ([maintainer comment][]).
The comment says the overhead that commit describes led to [issue 988][],
that [pull request 992][] addressed it in 1.3.2,
that a structural change ([issue 989][],
[pull request 997][]) is merged into `main` and planned for 1.4.0,
and asks whether this project would reassess Optique on 1.3.6 or `main`.
It also says:
"I haven't rerun your workload,
so I can't say how much overhead remains."

## Questions

1.  Why has the structural change not been released,
    and is there really no pre-release of it?
2.  Why was the cost behind [issue 988][] introduced,
    why did upstream's tests not catch it,
    and why were such tests absent?
3.  Has upstream taken steps,
    or stated an intention,
    to prevent a repeat?
4.  Is good performance a goal of Optique?

## Sources and how they were read

- Upstream Git history:
   a full clone of `dahlia/optique` at `1d6e459442e92f83006a7c34e4f11f5018c9ab39`
   (2026-11-12T19:55:47+09:00),
   read with `git log`,
   `git show`,
   `git tag --contains` and `rg`.
- Upstream issues,
   pull requests,
   milestones and review comments:
   `gh issue view`,
   `gh pr view` and `gh api`.
- npm:
   the package document at `https://registry.npmjs.org/@optique/core`,
   fetched 2026-11-12.
- JSR:
   `https://jsr.io/@optique/core/meta.json`,
   fetched 2026-11-12.
- This repository:
   `git show` of the commits named in each finding.

Quotations are verbatim from those outputs.

## Checks of the maintainer's statements

Each statement in the [maintainer comment][] that the record can test holds:

- "This commit used v1.1.1":
   `pnpm-lock.yaml` at the parent of `4879a44e6a` resolves `@optique/core@1.1.1`.
- "your current lockfile has v1.3.6":
   `pnpm-lock.yaml` on `main` resolves `@optique/core@1.3.6`.
- "addressed by #992 in v1.3.2 on September 29":
   tag `1.3.2` is dated 2026-09-29T23:01:50+09:00 and contains commit
   `89fa47a96d61c4dd650baad10a97528311417a1f`.
- "This change is not in v1.3.x":
   `git branch --remotes --contains 868a9d77139433608178eb3453bf9aaafccd0066` lists only `origin/main`,
   and no tag contains that commit.

The comment leaves one thing out:
the structural change has been installable from npm and JSR since 2026-09-30,
as the finding for question 1 shows.

## Question 1: release state of the structural change

### It is not in any stable release

Commit `868a9d7` ([pull request 997][]) is on `main` only.
The newest stable release is 1.3.6,
tagged 2026-10-15T03:57:28+09:00.

### Upstream's recorded reason is a behavior change

[Issue 988][] limited the patch release to work that leaves behavior alone:
"Reduce this overhead without changing the public API,
parser control flow,
suggestion results,
or the invocation behavior of custom error callbacks."
It names its target as "`1.0-maintenance`,
for a patch release",
and hands the rest to [issue 989][],
whose target is "`main`,
for a future minor release".

The structural change does alter observable behavior.
Its changelog entry in upstream `CHANGES.md` reads:
"Custom mismatch error callbacks now run when their diagnostic is requested,
so callbacks used for side effects may run fewer times."

No written branch or backport policy was found.
A search of upstream's Markdown files,
leaving out the changelog and its fragments,
for `maintenance`,
`backport`,
`patch release` and `cherry` returned two unrelated lines in `docs/cookbook.md` and `docs/why.md`.
The split between patch and minor work is therefore upstream's practice as shown by these two issues,
not a rule this investigation could read.

### 1.4.0 has no date, and its pace matches earlier minors

Milestone "Optique 1.4" was created 2026-09-26T15:06:52Z.
It has no due date,
56 closed issues and 2 open ones:
1103 ("Add a default command concept to `runProgram()`")
and 1024 ("Move development tooling off Deno while preserving Deno and JSR support").

Earlier minor releases were tagged on 2026-04-14 (1.0.0),
2026-06-16 (1.1.0),
2026-07-21 (1.2.0)
and 2026-09-15 (1.3.0),
gaps of 63,
35
and 56 days.
On 2026-11-12,
58 days have passed since 1.3.0.
The record shows no delay specific to this change.

### Pre-releases exist and carry the change

Upstream's `.github/workflows/main.yaml` publishes every non-tag build to npm under the `dev` dist-tag
(`pnpm publish ... --tag dev`),
with the version `<version>-dev.<run number>+<commit>`.
npm lists 79 builds named `1.4.0-dev.N`,
from `1.4.0-dev.2600` (2026-09-26T13:48:06Z) to `1.4.0-dev.2971` (2026-11-12T11:18:53Z),
and its `dev` dist-tag points at the last.
JSR lists 92 and keeps the source commit in each version string.

The JSR names place the change:

- `1.4.0-dev.2664+6f258296`,
   published 2026-09-29T16:45:35Z,
   is the commit [pull request 997][] was based on;
- `1.4.0-dev.2665+868a9d77`,
   published 2026-09-30T01:43:17Z,
   is the head commit of that pull request,
   built before it merged;
- `1.4.0-dev.2667+68b5edd1`,
   published 2026-09-30T02:03:47Z,
   is its merge commit on `main`.

So a build with the structural change was on both registries within minutes of the merge at 2026-09-30T01:59:20Z.
There is no build named alpha or beta.
A search of upstream's README files and `docs/` for `unstable`,
`pre-release`,
`@dev`,
`dist-tag`,
`--tag dev`,
`dev build`,
`nightly` and `canary` found no instructions for installing these builds;
the only hits describe a semantic-version value parser.

## Question 2: where the cost came from, and why tests did not catch it

### The mechanism

`object()` offers each token to each option parser before the positional parser.
An option parser that does not match returns a failure carrying an error message.
Before 0.7.0 that message was a short template.
Commit `2a772953f3155123f0d1c02dfbc4685c1d133167`,
whose subject is `Automatic "Did you mean?" suggestions for typos`,
dated 2025-11-17T07:21:11+09:00,
replaced it,
in `option()`,
`flag()`,
`command()`,
`object()`,
`or()` and `longestMatch()`,
with a call to `createErrorWithSuggestions()`.
That function collects every option name from the parser's usage tree into a set
and runs `levenshteinDistance()` between the token and each name.

Two structural facts made that expensive:

- failures built their message at the moment of the mismatch,
   whether or not anything would read it,
   which predates the suggestion feature;
- a mismatch is ordinary control flow for a positional token,
   so the work ran once per token per option parser.

[Issue 988][] measured the product:
thirty options and a thousand paths made one successful parse invoke the mismatch path thirty thousand times.

A custom message did not avoid it,
as [issue 988][] says:
"A custom `errors.noMatch` message does not skip this computation,
including when the message is static."
This repository's commit `1e53f52e02989b152576c0683f6f228b24c40140` recorded the same on 2026-07-15.

### The design note that preceded it

The feature has a design note,
[issue 38][],
opened by the maintainer on 2025-11-16T20:11:36Z.
Its section "Performance considerations" says the edit-distance cost
"should not cause performance issues because":

1.  "Option names are typically short (< 20 characters)"
2.  "Most CLIs have fewer than 100 options"
3.  "The algorithm includes early termination when an exact match is found"
4.  "The distance calculation can skip candidates with very different lengths"

and concludes:
"For a typical CLI with 50 options,
the worst-case overhead is less than 1ms on modern hardware."

Set beside the code in commit `2a77295`,
that analysis differs from what shipped in three ways:

- It models the input as a mistyped option name.
   The code runs on any token an option parser rejects,
   which includes every positional argument,
   and a file path is far longer than an option name.
- It counts one search per error.
   The code searches once per token per option parser,
   during parses that succeed.
- Its fourth reason was not implemented.
   `findSimilar()` in that commit computes the full distance for every candidate;
   it has no length comparison.
   [Pull request 992][] added exactly that check ten months later:
   "reject candidates whose normalized length difference already exceeds the distance or ratio threshold".

The third reason,
early termination,
was implemented,
but only after the distance to that candidate had been computed.

### How it was made and reviewed

- The commit's author date is 2025-11-17T07:20:59+09:00,
   2 hours 9 minutes after [issue 38][] was opened;
   the issue closed 34 seconds after that.
- The commit carries the trailer `Co-Authored-By: Claude <noreply@anthropic.com>`.
- GitHub lists no pull request for the commit
   (`gh api repos/dahlia/optique/commits/2a77295.../pulls` returns an empty list),
   so no review record exists.
- It changed 18 files with 2,040 insertions.

### Why the tests did not catch it

The commit added `suggestion.test.ts` (622 lines) and `parser-error-suggestions.test.ts` (450 lines),
plus smaller additions to six other test files.
Every test name in `parser-error-suggestions.test.ts` describes a parse that fails
and asserts what its message says,
for example "should suggest similar option on typo" and "should limit suggestion count to 3".
`suggestion.test.ts` tests the distance function,
the similarity search and message formatting on fixed inputs.

No test in that commit parses successfully with positional arguments and checks what work was done,
and none times or counts anything.
The property that broke,
that a successful parse does no diagnostic work,
was never stated as a requirement,
so nothing asserted it.

At tag `1.3.1`,
the last release before the patch,
a search of `packages/core/src/*.test.ts` for a thousand-element input,
`performance`,
`throughput` or `scaling` found no test of parse cost.
The hits are timeouts,
numeric literals,
unrelated names,
and one block in `dependency-extra.test.ts` named "Performance and stress tests":
eight tests that check the `dependency()` combinator gives correct results under repeated or large use,
none of which times anything or asserts how much work was done.

### How long it shipped

Tag `0.7.0` (2025-11-25T23:46:51+09:00) is the first release containing the commit.
Tags `1.0.11`,
`1.1.9`,
`1.2.9` and `1.3.2`,
all dated 2026-09-29,
are the first containing the patch.
That is 308 days,
and 89 tags contain the feature without the patch.

### Nobody reported it upstream

A search of upstream's issues,
open and closed,
for `performance`,
`slow`,
`benchmark`,
`levenshtein`,
`overhead` and `latency` returns nothing about parse cost before [issue 988][].
No report from this project is on upstream's tracker:
`gh issue list --repo dahlia/optique --state all --author Aquaticat` returns no issue,
and a search of upstream's issues for `Monochromatic` returns only the maintainer's own four.
The owner confirmed on 2026-11-12 that nobody reported it by any channel:
asked whether anyone for this project told the maintainer before 2026-09-28,
the answer was "No,
nobody reported it".
This project measured the cost on 2026-07-15 and replaced the parser the same day.
It recorded the finding in its commit messages and,
from 2026-08-14,
in `doc/handover/cli-git-cac-migration.md`
and `doc/audit/tech-cli-git-command-line-argument-parser-replac-vet-2026-08-14.md`;
no file under `doc/troubleshooting/` described it until 2026-11-12.

Upstream found it by reading this repository.
The maintainer opened issues 988,
989,
990 and 991 within 24 seconds of each other on 2026-09-28,
from 14:08:55Z to 14:09:19Z,
74 days after commit `4879a44e6a`,
and each cites this repository's commits as its source.
Issues 990 and 991 concern the two parsing defects commit `1e53f52e02` describes:
an undeclared `--name=value` token accepted as a positional argument,
and a known value option with its value missing captured by `passThrough()` as a successful parse.

Why the tests for this were absent for 308 days therefore has a plain answer in the record:
no test stated the property,
and no report said it had broken.
Whether upstream ever profiled a parse with many positional arguments before 2026-09-28 is not in the record.

## Question 3: steps upstream has taken or announced

### Taken: regression tests for this mechanism

[Pull request 992][] added tests under "eager suggestion compatibility" and "findSimilar compatibility".
They check that a literal `noMatch` message skips the candidate search
and that the faster search returns what the original algorithm returned.

[Pull request 997][] added `packages/core/src/deferred-diagnostics.test.ts`,
nineteen tests.
They count calls rather than time,
for example "does not call noMatch for discarded failures",
"does not scan suggestions for an unread merge failure"
and "defers default suggestions in discarded alternatives".
These would fail if a later change made discarded failures build their diagnostics again.
They do not bound how the cost of a parse grows with the number of arguments.

### Taken: a startup benchmark, which measures something else

[Issue 1083][] ("Establish reproducible startup and install baselines",
opened 2026-10-18,
closed 2026-11-02) added `benchmarks/startup/` and a CI workflow,
`.github/workflows/startup-baseline.yaml`.
Its README states its limit in its opening paragraph:
"It does not measure parser throughput."
No argument list in its manifest has more than three tokens (`start alpha beta` is one).
CI "does not compare timing with a budget",
and the issue says of itself:
"This issue adds evidence and reporting,
not a speed target or loading API."

The issue names its cause,
and it is not this incident:
a third-party comparison of TypeScript CLI libraries that measured whole-process startup.

[Issue 1084][] ("Reduce measured startup and packaging costs") is open on the 2.0 milestone.
It concerns module loading and package contents.

### Not found: anything aimed at a repeat

The following were searched for and not found:

- a test or benchmark that grows the argument count and checks how parse cost grows;
- a stated complexity bound for parsing;
- a written review of how the cost went unnoticed;
- any change to how a feature's performance assumptions get checked before it ships.

The searches were upstream's issues for the terms listed under "Nobody reported it upstream",
GitHub discussions for `suggestion overhead`,
`performance` and `slow` (ten results,
all release announcements or unrelated requests),
and the repository's Markdown files.
A web search for a public statement returned upstream's own documentation and unrelated projects.
An absence in those searches does not show that no such intention exists;
it shows that none is written where these searches reach.

### How the two fixes were reviewed

- [Pull request 992][] was opened 2026-09-28T14:56:05Z and merged 5 minutes 39 seconds later.
   Its review list is empty.
   Two automated reviewers ran:
   Codex reported "Didn't find any major issues",
   and CodeRabbit reported "No actionable comments were generated".
- [Pull request 997][] was opened 2026-09-30T01:46:41Z and merged 12 minutes 39 seconds later,
   changing twelve files with 875 insertions.
   Codex did not review it:
   "You have reached your Codex usage limits for code reviews."
   CodeRabbit raised one comment,
   the maintainer answered it at 01:56:05Z,
   and the bot withdrew it at 02:05:12Z,
   after the merge.
   A second requested review did not finish:
   "Pull request base or head changed."

Both pull requests record local measurements in their descriptions.
[Pull request 992][] reports
"median parse times of 1,736 ms before and 167 ms after (15 runs after 5 warmups).
Short-path default/callback runs showed no improvement."
[Pull request 997][] reports
"discarded callback calls fell from 30,000 to 0 and median parse time fell from 112 ms to 65 ms."
Neither benchmark script is in the repository.

## Question 4: is performance a goal of Optique?

### Stated

No.
A search of `README.md`,
`docs/why.md`,
`docs/index.md` and the READMEs of `packages/core` and `packages/run` for `performance`,
`performant`,
`fast`,
`faster`,
`overhead`,
`lightweight`,
`startup`,
`speed`,
`slow`,
`benchmark` and `efficient` returned no line.
The same search for `type-safe`,
a term those files must contain,
matched in `README.md`,
`docs/why.md` and `docs/index.md`,
so the search can see a stated goal when one is there.
Both packages describe themselves as "Type-safe combinatorial command-line interface parser",
the repository as "Type-safe combinatorial CLI parser for TypeScript",
and its topics are `cli`,
`getopt`,
`parser-combinators` and `typescript`.
Across all of `docs/`,
`performance`,
`performant` and `benchmark` occur on six lines,
about shell completion,
configuration loading and custom parsers.

Where upstream writes about speed,
it declines to set a target.
[Issue 1083][] calls itself
"not a speed target".
[Issue 1084][] says
"The user-facing goal is cheaper short-lived invocations and smaller production installations
where measurements show avoidable work.
No particular bottleneck,
percentage improvement,
or new public abstraction is assumed."

### Shown by conduct

Upstream treats a known performance defect as a bug worth fast work:

- [issue 988][] carries the `bug` label;
- the patch reached four maintained release lines within 24 hours of the issue being opened;
- the structural follow-up merged the day after that release;
- the maintainer then wrote to this project,
   44 days after 1.3.2,
   asking for a reassessment and a reproducer.

Upstream does not treat performance as a property to be guarded before a report:
the 2025 design note gives an estimate and records no measurement,
the measurement scripts for both fixes were not kept,
and the one maintained benchmark excludes parsing.

The fair summary is that performance is a concern upstream responds to,
not a goal it measures or tests for.

## The owned parser is not a like-for-like comparison

The owner,
on 2026-11-12:
"The ts repo parser for git wrapper had many bugs.
This isn't an apples to apples comparison."

Any figure that sets Optique beside the parser that replaced it carries that limit,
including the 4.24 s and 0.89 s in commit `4879a44e6a`:

- The replacement was a different parser with different behavior,
   not a faster implementation of the same one.
   Commit `1e53f52e02` says its token handling
   "was taken from probing @optique/core 1.1.1 rather than from reading it",
   and that two inputs Optique accepted now raise an error.
   [Issue 988][] makes the same point from upstream's side:
   "the replacement also changes parsing behavior."
- It needed a fix within minutes.
   Commit `6ef7bee28b` ("keep undeclared joined git options working") is dated 2026-07-15T20:06:08-04:00,
   under thirteen minutes after the replacement landed.
- It no longer exists.
   Commit `642b6d72cb` deleted the TypeScript Git policy wrapper on 2026-10-13T17:15:06-04:00.

So the owned parser's speed shows how little work the task needs,
not that Optique could have been that fast while doing everything Optique does,
and not that the replacement was correct.
This document counts neither the owned parser's defects nor Optique's;
it reads the repository's record of the three facts listed here and no further.

## Measurement of this repository's old parser

Measured 2026-11-12,
from 14:46:01Z to 15:05:51Z.
This is the measurement the [maintainer comment][] says upstream has not made.

### What was run

- Subject:
   `parseCommitRegion` from `package/git-policy/cli/src/parser/commit.ts` at commit `1e53f52e02`,
   the parent of `4879a44e6a`,
   with the files it imports,
   unchanged.
- Arms:
   `@optique/core` 1.1.1,
   1.3.1,
   1.3.2,
   1.3.6,
   and the pre-release builds `1.4.0-dev.2664`,
   `1.4.0-dev.2665`,
   `1.4.0-dev.2667`
   and `1.4.0-dev.2971`,
   each unpacked from its npm tarball.
   The installed files confirm what the JSR version strings say:
   `1.4.0-dev.2664` has no `dist/internal/failure.js`,
   the other three have it,
   and `1.4.0-dev.2665` and `1.4.0-dev.2667` differ only inside `package.json`,
   in the `version` field and the pinned version of `@optique/env`.
- Floor:
   the owned parser from commit `4879a44e6a`,
   under the limit the section "The owned parser is not a like-for-like comparison" states.
- Input:
   `['--message', 'x', ...paths]` with no `--` separator.
   The real path list is the first thousand tracked files at commit `1e53f52e02`,
   from 9 to 104 characters,
   median 55.
   The short list holds names of 1 to 3 characters.
- Method:
   one fresh process per arm and input,
   2 warmup parses and 7 timed ones,
   repeated over 7 rounds with the arm order rotated,
   so each figure is the median of 49 timed parses.
- Where:
   a container from `docker.io/cachyos/cachyos-v4:latest` with Node v26.13.1,
   no mounts,
   `--cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10`,
   on a host whose one-minute load average,
   recorded with every run,
   ranged from 29 to 56.

A Pi agent built and ran the harness.
Its medians for the thousand-path input were recomputed here from its raw records and match,
and a separate timing loop written for this check,
run once per arm in the same image,
gave 2,793 ms on 1.1.1,
181 ms on 1.3.6,
89 ms on `1.4.0-dev.2971`
and 0.13 ms for the owned parser.
The other inputs come from the agent's summary and were not recomputed.

### Run-to-run spread

Eight copies of one unchanged arm,
interleaved over six rounds,
differed in their medians by 3.6% on 1.3.6 and 11.0% on `1.4.0-dev.2971` at a thousand real paths,
and by up to 16.9% at one path.
A difference smaller than that is noise.

### A thousand real paths

Median,
with minimum and maximum:

- 1.1.1:
   2,695.7 ms (2,574.6 to 3,898.9)
- 1.3.1:
   2,706.7 ms (2,570.5 to 2,985.2)
- 1.3.2:
   180.0 ms (166.3 to 206.0)
- 1.3.6:
   184.3 ms (172.8 to 228.5)
- `1.4.0-dev.2664`,
   `main` just before [pull request 997][]:
   229.0 ms (206.6 to 294.6)
- `1.4.0-dev.2665`,
   that pull request's head:
   90.2 ms (80.5 to 115.0)
- `1.4.0-dev.2667`,
   its merge:
   96.8 ms (84.4 to 114.8)
- `1.4.0-dev.2971`,
   the current `dev` build:
   95.0 ms (83.6 to 242.6)
- owned parser:
   0.18 ms (0.09 to 1.32)

What this shows:

- The slowness reproduces on 1.1.1,
   so the probe can see it.
   The 2.56 s in commit `4879a44e6a` was measured inside the whole wrapper by a method that commit does not record;
   the two figures agree in size and are not the same measurement.
- [Pull request 992][] removed 93% of the time,
   between 1.3.1 and 1.3.2.
- 1.3.6 still takes 184 ms for this parse,
   0.18 ms per path.
- The 1.4 line was slower than the 1.3 line at the same patch level.
   `1.4.0-dev.2664` was built on 2026-09-29 from a commit that contains tag 1.3.2 and no later 1.3 tag,
   and took 229.0 ms where 1.3.2 took 180.0 ms:
   27% more,
   which is beyond the spread.
   An earlier wording here compared it with 1.3.6 and said `main` "had become" slower than it;
   1.3.6 was tagged on 2026-10-15,
   after that build,
   so that comparison was out of order.
   [Pull request 997][] then cut the 1.4 line's time by 61%.
- The current pre-release takes 95 ms,
   about half of 1.3.6.
- The parse grows with the number of paths in every Optique arm.
   On 1.3.6 it took 1.0 ms for one path,
   2.5 ms for ten,
   23.4 ms for a hundred
   and 184.3 ms for a thousand.

### A thousand short paths

- 1.1.1:
   303.9 ms
- 1.3.1:
   300.3 ms
- 1.3.2:
   185.3 ms
- 1.3.6:
   184.0 ms
- `1.4.0-dev.2664`:
   228.6 ms
- `1.4.0-dev.2665`:
   91.8 ms
- `1.4.0-dev.2667`:
   90.1 ms
- `1.4.0-dev.2971`:
   89.7 ms
- owned parser:
   0.17 ms

[Pull request 992][] says short paths "showed no improvement".
On this parser they improved by 38% between 1.3.1 and 1.3.2.
The likely reason is that this parser's option names are long,
so the new length check rejects them against a token of one to three characters;
that reading comes from the patch's code and was not tested separately.

From 1.3.2 through 1.3.6,
short and long paths cost the same,
about 0.18 ms each,
and on the pre-releases with [pull request 997][] about 0.09 ms each.
What remains no longer depends on the token's length.

### Upstream's own reproduction

The script in [issue 988][],
thirty options and a thousand long paths:

- 1.1.1:
   964.3 ms
- 1.3.1:
   952.0 ms
- 1.3.2:
   111.1 ms
- 1.3.6:
   109.3 ms
- `1.4.0-dev.2664`:
   147.8 ms
- `1.4.0-dev.2665`:
   83.0 ms
- `1.4.0-dev.2667`:
   80.3 ms
- `1.4.0-dev.2971`:
   86.2 ms

Upstream reported 1,736 ms to 167 ms for [pull request 992][] and 112 ms to 65 ms for [pull request 997][].
The ratios here,
8.6 from 1.3.1 to 1.3.2 and 1.8 from `1.4.0-dev.2664` to `1.4.0-dev.2665`,
are close to upstream's 10.4 and 1.7;
the absolute values differ with the machine and the two-CPU cap.

### Control: paths after the separator

With `['--message', 'x', '--', ...paths]`,
`parseCommitRegion` cuts the list at the separator and hands Optique two tokens,
`--message` and `x`.
Across every path count and both path lists,
the Optique arms' medians ran from 0.39 to 0.97 ms,
with no version standing apart.
The owned parser took 0.03 to 0.05 ms up to a hundred paths and 0.16 to 0.17 ms at a thousand,
since it reads the paths itself.
No version changes that fixed cost.

### Where the remaining time goes

`node --cpu-prof` on the thousand-real-path input,
as shares of the time inside the parse:

- 1.1.1:
   `levenshteinDistance` 93.1%;
   everything in `suggestion.js` 96.4%.
- 1.3.6:
   `traverseUsage` 23.7%,
   `findSimilar` 21.0%,
   `createErrorWithSuggestions` 20.1%,
   `set` in `input-trace.js` 10.5%,
   `levenshteinDistance` 5.6%,
   `extractOptionNames` 3.6%.
   By file,
   `suggestion.js` 46.6% and `usage.js` 27.3%.
- `1.4.0-dev.2971`:
   no suggestion code at all.
   `set` in `input-trace.js` 22.9%,
   `withParserOptionScope` 9.1%,
   `createDeferredFailure` 8.3%,
   `withOptionScopeChild` 7.7%,
   `forkOptionScope` 6.8%.
   By file,
   `short-option.js` 31.3%,
   `input-trace.js` 23.3%,
   `constructs.js` 14.2%.

So on 1.3.6 the edit distance is nearly gone,
and about three quarters of the parse is still spent collecting option names
and searching them for suggestions nobody reads.
On the pre-release that work is gone,
and what remains is bookkeeping done each time a token is offered to an option parser.

### Results are the same on every version

For every timed input,
every Optique arm returned the same `CommitRegion` record.
Fifteen further option-shaped inputs also gave the same record on every Optique arm.

The library itself did change at 1.3.2,
as issues 990 and 991 describe:
run directly,
`['--unknown=v', 'a.txt']` gives `paths` of both tokens on 1.1.1 and 1.3.1,
and `paths` of `a.txt` alone from 1.3.2 on;
`['-m']` succeeds on 1.1.1 and 1.3.1 and fails from 1.3.2 on.
The old parser's record hides both changes,
because it never read Optique's positional list and turned a failed parse into an all-false record.

The owned parser differs on four of the fifteen inputs:
it throws for an undeclared `--name=value` token and for a value option missing its value,
where the old parser returned a record.
That is the behavior change commit `1e53f52e02` announced.

### What this does and does not support

- It answers the maintainer's question for this parser:
   on 1.3.6 a thousand-path parse costs 184 ms,
   and on the current pre-release 95 ms,
   against 2,696 ms on the version this project left.
- It does not say what the old wrapper would have cost in total on 1.3.6.
   The wrapper no longer exists,
   and only its parse was timed.
- It does not rank Optique against the owned parser.
   The 0.18 ms shows that reading a thousand paths needs almost no work;
   the two parsers do different things.
- The harness lives in `${HOME}/temp/agent/optique-measure-2026-11-12/`,
   which is scratch and not kept.
   `doc/troubleshooting/optique-eager-typo-suggestions-on-positional-arguments.md` holds a standalone script
   that shows the same effect with nothing but `@optique/core`.

## Answered here before asking upstream

The owner's instruction on 2026-11-12 was to resolve what analysis,
measurement and source reading can resolve before any question goes to the maintainer.
This section holds those answers.
Scaling experiments,
the attribution of the 1.4 line's extra time,
and an audit of `passThrough()` were still running when it was written.

### The differences, with each process as one observation

The medians under "Measurement of this repository's old parser" pool 49 timed parses per arm.
Those parses are not independent:
seven fresh processes each contributed seven.
Taking each process's median as one observation,
and comparing two arms round by round,
gives seven paired ratios per comparison.
With seven pairs,
all seven falling on one side has a two-sided sign-test probability of 0.0156 if the arms did not differ,
and that is the smallest value seven pairs can give.

`1.4.0-dev.2665` and `1.4.0-dev.2667` are the same code,
which makes them a control for what "no difference" looks like:
their paired ratios ran from 0.852 to 1.172 on the thousand-real-path input,
five rounds one way and two the other.
A ratio inside that band,
without all seven rounds agreeing,
is not evidence of a difference.

On the thousand-real-path input,
later arm over earlier arm,
median ratio with its lowest and highest:

- 1.1.1 to 1.3.1:
   1.004 (0.831 to 1.037),
   four rounds slower and three faster.
   No difference.
- 1.3.1 to 1.3.2:
   0.065 (0.061 to 0.070),
   all seven faster.
- 1.3.2 to 1.3.6:
   1.044 (0.974 to 1.202),
   four slower and three faster.
   No difference.
- 1.3.2 to `1.4.0-dev.2664`:
   1.241 (1.144 to 1.430),
   all seven slower.
- `1.4.0-dev.2664` to `1.4.0-dev.2665`:
   0.399 (0.371 to 0.473),
   all seven faster.
- `1.4.0-dev.2667` to `1.4.0-dev.2971`:
   1.017 (0.885 to 1.087),
   four slower and three faster.
   No difference.
- 1.3.6 to `1.4.0-dev.2971`:
   0.510 (0.459 to 0.608),
   all seven faster.

So the 1.4 line's extra time over 1.3.2 is real in direction,
since every round agrees,
and uncertain in size:
24% at the median,
between 14% and 43% in single rounds.
The same holds on the other two inputs:
1.209 (1.148 to 1.366) for a thousand short paths
and 1.331 (1.270 to 1.454) for upstream's own reproduction,
all seven rounds slower in each.

### When 1.4.0 is likely

An estimate,
not a forecast from a measured rate:
the work it rests on has not started.

What the record shows:

- Upstream tags a minor release when its milestone has no open issue left.
   1.3.0 was tagged minutes after its milestone's last closure.
   1.2.0 was tagged with one issue open,
   which closed 84 minutes later.
- The milestone keeps growing until then.
   Thirty-two of the 1.3 milestone's 67 items were created in the fourteen days before its tag,
   and four items were added to the 1.4 milestone between 2026-11-10 and 2026-11-12.
- Milestone "Optique 1.4" has 58 items and two open.
   Issue 1103,
   a feature,
   was opened on 2026-11-02 and has had no activity since.
   Issue 1024 moves the project's development tooling off Deno in six steps;
   no commit or pull request for it exists,
   and 26 lines of `mise.toml` still mention `deno`.
- The maintainer opened that same migration issue on five projects within an hour on 2026-10-09.
   Two are finished:
   `dahlia/logtape` and `dahlia/upyo` both closed theirs on 2026-10-19,
   ten days later.
   On `fedify-dev/fedify` the seven migration pull requests merged between 2026-10-27 and 2026-11-11,
   sixteen days from the first being opened to the last being merged.
   Optique's has not started.
   The fifth,
   on `fedify-dev/botkit`,
   is open and was not examined.
- Commits on Optique's `main` fell to five and then one per week in the two weeks around the start of the Fedify
   migration,
   and returned to nineteen and sixteen in the two weeks since.

Reading:
the same person does these migrations one project at a time,
finished the largest on 2026-11-11,
and takes from under ten to sixteen days for one.
If Optique's starts within the next week and nothing else is added that blocks the milestone,
1.4.0 would fall between about 2026-11-20 and 2026-12-20,
most likely in the first half of December.
It could come any day if issue 1024 is moved to a later milestone,
and later than that range if the migration waits behind other work.
The gaps between earlier minor releases (63,
35 and 56 days) say nothing more:
58 days have already passed.

### Facts read from source and record

- `object()` already tried each field parser for each token when the feature was written.
   The loop is at `packages/core/src/constructs.ts` line 1331 in commit `2a77295`,
   after the priority sort at line 1308.
   So the path from a positional token to every option parser's no-match branch existed in the code
   the suggestion call was added to.
- The documented ways to turn suggestions off did not turn the search off.
   Commit `4e77bb41`,
   the same day as the feature,
   added to `docs/concepts/messages.md` an example headed
   "Disable suggestions by ignoring the suggestions parameter"
   and another headed "Disable suggestions entirely" (`suggestions: () => []`).
   Both change what is shown.
   In both,
   the names are collected and searched before the callback runs,
   as the source excerpts in the troubleshooting document show.
- Upstream's documentation does not describe the cost that remains on the 1.3 line.
   A search of `docs/` and the core README on `1.3-maintenance` and on `main`
   for text tying suggestions or `noMatch` to cost,
   overhead,
   performance or many positional arguments returned nothing.
- No issue on upstream's tracker proposes dispatching a token to the parsers that could accept it,
   or a benchmark of parsing.
   A search for `dispatch`,
   `throughput`,
   `complexity`,
   `parse benchmark` and `quadratic` returned only unrelated uses of those words.
- Upstream has published no security advisory
   (`gh api repos/dahlia/optique/security-advisories` returns an empty list),
   so issues 990 and 991 were announced through the changelog alone.
- Whether other projects met the same cost could not be established.
   A GitHub-wide issue search for `optique Levenshtein` returns nothing,
   but the same words scoped to upstream's repository find issues 38 and 989,
   so the wide search returns less than exists and its silence means nothing.

## Independent review

A reviewer from another model vendor,
qwen4-max-preview run through Pi,
checked this document and the troubleshooting document against their sources on 2026-11-12,
reading them as they stood at commit `b104ae74cd`.
It recomputed every figure from the raw records and reread every quoted source.

It reported every quotation verbatim,
every code excerpt matching its tag and lines,
and every millisecond figure in the measurement lists reproducing from the raw records.
It found seventeen points in the prose around them,
none in the data.
Each was checked against the records before being acted on.
The ones that were errors of fact and are now corrected:
the load range,
the token count and ranges of the control with paths after the separator,
what the duplicate search returns,
the mention of Optique under `.out-of-scope/`,
a test block named for performance at tag 1.3.1,
the second field in which two pre-release builds differ,
which pushes publish a pre-release,
and a second document that had recorded the cost in August.
Four were over-wide wordings and are narrowed.
One,
that a ratio of 8.6 should read 8.7,
compared a different pair of versions from the one meant;
the text now names the pair.
The remaining four were matters of wording:
three are reworded,
and one sat in a line of the draft reply that had already been removed.

The sections "Answered here before asking upstream" and everything dated after that commit were not part of that review.

## What the record cannot answer

These need the maintainer's own account:

- why the fourth mitigation in [issue 38][] was not implemented in 2025;
- whether the "less than 1ms" estimate was ever measured;
- whether parse cost over many positional arguments had been profiled before 2026-09-28;
- whether a scaling test or a parse benchmark is planned;
- whether the cost that remains after [pull request 997][],
   where every token is still offered to every option parser,
   is the intended end state;
- why the 1.4 line was slower than 1.3.2 before that pull request;
- when 1.4.0 is expected.

[maintainer comment]: https://github.com/Aquaticats/Monochromatic/commit/4879a44e6a2460ab3c2531744e24a1f2aef27aeb#commitcomment-209502949
[issue 38]: https://github.com/dahlia/optique/issues/38
[issue 988]: https://github.com/dahlia/optique/issues/988
[issue 989]: https://github.com/dahlia/optique/issues/989
[pull request 992]: https://github.com/dahlia/optique/pull/992
[pull request 997]: https://github.com/dahlia/optique/pull/997
[issue 1083]: https://github.com/dahlia/optique/issues/1083
[issue 1084]: https://github.com/dahlia/optique/issues/1084
