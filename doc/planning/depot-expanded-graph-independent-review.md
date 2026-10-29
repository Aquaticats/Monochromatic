# Independent review of expanded Depot graph failures

## Purpose and scope

Review the original retained graph artifact ZIPs and source-bound command logs,
separating application assertions,
harness/tool prerequisites,
provider access,
and incomplete execution.
This is a read-only evidence review;
it does not dispatch or repair the measured workload.
The application source remains `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.

## Reviewed evidence

The [independent receipt](../audit/depot-expanded-graph-independent-review-2026-10-18.evidence.json)
records the exact original ZIP identities,
their byte counts and SHA-256 digests,
all ZIP CRC checks,
per-command member hashes,
source hashes,
and the derived browser-reporter views.
No cloud command or allocation was needed for this review.

- GitHub run `39140975786` contributes repeated GitHub-hosted and Depot GitHub runner samples.
  Native run `gb01kjx2qk` contributes the original repeated native samples.
  Their harness is `234ccc92dabb81e998e0a0bb5e838713924dd83f`.
- Native correction `fnvwqfg0bb` contributes repeated terminal samples at harness
  `72af18d5b130dabef7ff5b656ffdc1b58df4fcde`.
  The receipt keeps this cohort separate from the original native samples.
- The eight originals contain `1720` command logs whose bytes match the original `graph.json` hashes.
  Each graph schedules `222` command invocations,
  giving `1776` scheduled invocations in total.
  The difference is `56` explicitly unstarted commands across the GitHub-hosted repetitions.
  These are command counts,
  not test-case counts.
- Every original reports no changed tracked application files and no remaining owned driver groups.
  Those checks are compared directly with the retained graph content.

## Application gates and incomplete execution

### Lint warnings are deliberately blocking

The frozen `package/config/oxlint/src/config-base.ts:31` sets `denyWarnings: true`.
Its wrapper preserves the tool's unsuccessful exit.
Therefore an Oxlint summary with warnings and no errors can still be a failed configured gate.
These outcomes do not establish an infrastructure failure.
The actual findings,
including required test-artifact imports,
remain source-quality work rather than accepted tests.

### Fuzzer parity failures are actual reached assertions

The frozen matcher differential test compares the fork with its upstream oracle.
The Depot GitHub repetition one original member `modules-a-043.log`
contains a counterexample with inputs `["a", ""]`,
patterns `["!", "!"]`,
and `allPatterns: true`:
the fork returns `false` where the oracle returns `true`.

The p-map differential member `modules-b-059.log` reaches both configuration-error comparisons
and asynchronous trace comparisons.
The `[null, [], []]` counterexample differs in the iterable-input diagnostic;
other counterexamples involve concurrency,
skipping,
rejection,
and output traces.
It would be inaccurate to reduce every p-map failure to error wording.

The fork invariant groups also run and pass in these logs.
The differential and invariant boundaries must remain distinct.
Their frozen sources do not fix a seed,
and the actual logged seeds differ.
Matcher's gate also succeeds in some native samples.
Source and tool identity therefore do not make all generated cases identical.
No complete-graph performance ratio follows from these fuzzer timings.

### WireGuard input failures use injected fixtures

The `wg-allowedips` member `modules-a-082.log` reaches five failed child assertions
in `generateAllowedIpsWithLookup`:
the warning aggregation assertion and four invalid-input diagnostic assertions.
The source at `package/wg/core-allowedips/src/generate-input.unit.test.ts:236`
expects an ASN warning that is absent;
the assertions at lines `347`,
`367`,
`387`,
and `411` expect specific CIDR diagnostic text.
The lookup implementations are injected frozen fixtures.

The HTTP `500`,
missing `IPINFO_TOKEN`,
and `ENOTFOUND` messages belong to passing negative-fixture groups.
They are not independent evidence that Depot egress or a live ASN provider failed.

### Logger failures have a narrow disposable-file boundary

The logger original member `modules-b-038.log` reaches a batch-write Promise identity assertion
and a JSONL parse failure during the reentrant `toJSON` case.
The owning `package/module/logger/src/sink/file-batch.unit.test.ts`
creates a disposable directory and exercises escaped,
newline,
and Unicode messages.
Its concurrent-record integrity group also passes.

These observations establish failed library assertions and malformed data at that test's read boundary.
They do not isolate a provider filesystem cause,
data loss,
or a general failure of logger writes.

### The watch-condition failure is intermittent across providers

The frozen `conf-fork` watcher test waits for a change notification with a `25000` millisecond bound.
The expected notification condition fails in three of the eight original samples:
repetition two of GitHub-hosted,
Depot GitHub,
and the original native cohort.
Both corrected-native samples pass.

This supersedes the initial three-of-six observation made before the corrected originals arrived.
The successful correction does not isolate a fix for the watcher:
the measured container prerequisite changed,
and a controlled watcher-cause experiment was not performed.
No provider-specific reliability rate follows from these samples.

## Browser and report-collection boundaries

### The original native browser commands fail before tests

In the original native cohort,
Podman preparation and the browser/E2E commands stop at
`newuidmap: Target process ... is owned by a different user`
and `cannot set up namespace using /usr/bin/newuidmap: exit status 1`.
Those commands are attempted,
but their browser assertions are not exercised.

The correction changes the native Podman wrapper to invoke the real executable through
`sudo --non-interactive`.
The source delta and both source hashes are retained in the receipt.
Both corrected samples complete preparation and reach the browser reporters.
This qualifies the measured privileged path;
it does not prove the original unprivileged path works or that native Podman is generally unsupported.

### The reached WebKit and E2E results remain failed gates

All four GitHub Actions samples and both corrected-native samples report WebKit results of
`26` expected,
`1` skipped,
and `1` unexpected,
with no flaky result.
The unexpected case is
`package/module/logger/src/sink/opfs.browser.test.ts:41`:
`createOpfsSink().verify()` returns `false` where the assertion expects `true`.
That establishes the availability-probe result;
it does not identify the underlying OPFS cause.
The skipped OPFS root-discovery case remains skipped.

The Chromium and Firefox E2E commands in those six samples report no test cases and a discovery error:
`Cannot find package '@monochromatic-dev/module-or-throw'`
from `package-paused/webapp-edu/paper2vn/src/paper2vn.e2e.test.ts`.
The frozen root E2E configuration matches `**/*.e2e.test.ts` without excluding that paused owner.
This is a reached discovery failure,
not passing E2E coverage or a failed browser assertion.

### The collector merges stdout and stderr

Both GitHub-hosted WebKit original members `browser-005.log`
have npm update notices physically interleaved inside the JSON reporter document.
Parsing the original document fails at position `32768`.
The original ZIP CRC and the member's recorded hash still pass.

The owning collector in `.depot/research/graph/mise.toml:9`
spawns each task with `stdio: ['ignore', fd, fd]`.
Both child streams therefore write to the same command-log descriptor.
This is a report-collection boundary,
not evidence that Playwright emitted invalid JSON on its stdout stream.

The research-derived view removes only the five explicitly retained physical `npm notice` lines.
The receipt records the removed text,
the original parse error,
the original member SHA-256,
and the SHA-256 after that exact transformation.
It then parses the reporter and checks its actual cases.
The original bytes remain unchanged.
This supports the reported assertion classification;
it does not qualify a strict machine consumer of the original mixed stream.

## Budget and comparison limits

Both GitHub-hosted samples reach six command budgets and leave `28` commands unstarted.
The collector's source at `.depot/research/graph/mise.toml:9`
uses a `180000` millisecond default command budget,
bounded again by the campaign's remaining lifetime.
It sends `SIGTERM` to the owned process group,
then applies cleanup.
The outer recorded outcome is not necessarily a process terminated directly by `SIGTERM`;
the exact original status and signal are retained.

Neither Depot GitHub sample nor either native cohort has a budget or unstarted command.
That means their scheduled commands were attempted,
not that their assertions all passed.
The original native browser prerequisite and the reached source assertions remain separate failures.

The shared producer's `42` commands and the Rust branch's `2` commands succeed in every sample.
Their complete command boundaries support a separately qualified repeated comparison.
Rust assertion identities remain independently checked by the Rust evidence owner.
The graph's incomplete GitHub execution,
differing generated cases,
and failed acceptance gates prevent a complete-CI multiplier or full expanded-CI acceptance claim.

## Publication correction

The platform diagnostic publisher completed with exit `0`.
Its `cli-git` notice naming a future `665` second wait limit did not establish an exhausted wait.
The earlier status wording that it had given up was too strong and is withdrawn.
Direct remote discovery returned `d71c018380e55365bf9dd5131914c2be152f0808` for `refs/heads/main`;
the ancestry check for the diagnostic ledger commit exited `0`.
The [existing wait-notice diagnosis](../troubleshooting/cli-git-push-wait-notice.md)
explains the separate advisory and failure paths.
No concurrent push process was interrupted.

## Current status

The requested eight-original graph classification review is complete.
It made no measured-source changes and required no cloud allocations.
Targeted Markdown lint,
CommonMark rendering,
internal-link checks,
JSON parsing,
and JSON formatting checks pass.
The publication checks also validate the six reached WebKit reports,
the twelve empty E2E discovery reports,
the watch-condition count,
and the explicit derived-view boundary.
The whole-report sufficiency review remains separately queued after the parent's graph,
operating-system,
and semantic evidence queues close.
