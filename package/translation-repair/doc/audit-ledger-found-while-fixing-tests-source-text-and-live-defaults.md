# Audit ledger: found while fixing, tests, source text and live defaults

Defects met while fixing another finding,
in the tests and fixtures,
in what source files say,
and in parameters that fell back to the live transport or the corpus clone.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## X2: a scripted gather-stage test's seat rotation depends on its prompt text

Status:
fixed in `87acdb4d9`:
`attestCitedReferences` takes the optional `fanOut` `pairBlocksWithRoster` takes,
threaded to both of its gathers,
and the four scripted cases ask the whole bench.
Checked by shifting the hash's offset basis:
three shifted offsets failed the pre-fix tests (2,
4 and 2 failures),
the fixed tests passed under the two offsets run against them,
and the control survived every run.
The window's own tests pass under any offset,
rightly:
another offset is another valid rotation.
`stage-fanout-window.ts` picks the seats to ask by an FNV-1a hash of the prompt modulo the roster size,
so rewording a fixture changes which scripted seat is heard;
`reference-attest.unit.test.ts` and `reference-attest-confirm.unit.test.ts`
failed against a passing HEAD on a reworded fixture
until phrases matching HEAD's rotation were chosen.
A test that depends on which seat is heard should pin the rotation rather than inherit it from its text.

## X3: comments and TSDoc quoting corpus text or handles in source files

Status:
the method quote fixed in `a96bf1f4a`,
which also replaced a TSDoc example naming the method;
the rest waits on the owner's end-of-project sanitization.
`reference-attest-match.ts:18-19,29-30,48-50`,
`translate-suicide-drop.ts:4-21` (quotes a method,
a date and handles),
`rendering-glossary-phrasing.ts:56,76-79`,
`archive-original-note.ts:37-53`,
`markdown-blocks.ts:13`,
`bilingual-pair-bound.ts:25`,
`bilingual-line-clause.ts:19`,
`corpus-run/run-config.ts:106`,
`image-asset.ts:45`,
`image-reading-stage.ts:98`;
and in untouched tests,
names in comments and test names
(XingZ60 32,
hakureico 23,
Toka_ls 6,
gqt 6,
Yumao 5,
aiyysk 3,
lintong 3,
羽毛 3,
Ling 2,
Hanasaka 1),
Yumao and 羽毛 as string literals.
The owner defers sanitization to the project's end;
the method quote in `translate-suicide-drop.ts` is the one the reader-protection rule covers now.

## X9: code indented against its nesting, which no check reads

Status:
fixed in `632ef5fbc` and `f7b766fa4`;
enforcement is issue #577.
`runCriticBenchmark`'s inner attempt,
its `try` body and its retry block sat two to three levels too deep
for about 190 lines of `benchmark.ts`;
a test body in `repair-translation.unit.test.ts` dropped two spaces for about 60 lines,
and the stall case in `stream-idle-guard.unit.test.ts` left an argument and its assertions too shallow.
All were lint-clean:
the dprint TypeScript plugin was retired for `oxlint-plugin-stylistic`,
which has no indentation rule.
A heuristic scan of the 1666 source files flagged 35 lines,
the rest being template-literal ends
(`~/temp/agent/audit-repair/indent-scan.mjs`).
The trailing-comma drift in some test files is not a finding:
`package/config/oxlint/src/overrides.ts` lets tests lay out calls freely and no config enables `comma-dangle`.

## X14: no test drives a pass seam

Status:
fixed;
every pass seam is driven by a test
(contest,
consolidation,
insertion admission,
preparation,
lanes and pictures).
The insertion admission seam (`admitPassInsertions`,
X12) is driven the same way in `2fbb7ec24`,
and the preparation seam (`runPassPreparation`,
X12) in `f891b358e` for the pairing
and `d00123df3` for the archive review.
`runPassContest`,
`runPassConsolidation`,
`readSeatedPictures` and `runPassEntry` wire each phase's hook
and bench into its driver,
and none had a test,
so a seam dropping its hook survived every guard
(the picture seam mutant in X12).
`76142da2c` exports the contest and consolidation seams,
and `6371fd9ff` drives each against a client
whose view turns Synthetic dry after the phase's own reading,
under a hold:
only a wired hook keeps the seat the dry-out took from being asked,
and a control whose view never changes shows that seat is asked otherwise.
Mutation checked with a control:
both seam mutants caught.
Lanes:
the hooks were spread into `runDocumentLanes` inside `runPassEntry`,
which no test drives.
`f2c329781` gives them a seam,
`runPassLanes` (`corpus-run/pass-lanes.ts`),
which takes the `readLanesSeats`
result whole,
and `30e57c62b` drives it on a real reading:
no lane call may ask a seat the dry-out took
from either lane bench.
Mutation checked with a control,
then the full suite (0 FAIL,
1280 PASS):
the seam dropping both hooks,
the translate re-seat alone and the per-slice hook alone were each caught.
Pictures:
`readSeatedPictures` reached the pinned corpus (`gatherEntryPictures`)
and shelled out to `dwebp` and `tesseract` (`readImageWithOcr`) with no way to hand either in,
and its one seam,
`visualEvidenceReader`,
replaces the whole reading,
hook and all.
`36aeeaa6c` adds an optional `pictureSources` input (`PassPictureSources`) replacing only those two,
and `f865f524a` drives the entry's picture reader over stand-in bytes and OCR.
The first mutation run caught three mutants and left two alive (M42 again):
the stage seating the roster instead of the readers,
since under a constant hold the hook re-seats every picture
and the first seating is never asked,
and the OCR ignoring the stand-in,
since the readers are asked either way.
`35684e182` adds a case with no hold and requires the stand-in OCR to have read;
the rerun,
with a control,
caught all five,
and the full suite passed (0 FAIL,
1281 PASS).

## X17: the rendered-sheets fixture claims every model-facing sheet and holds fifteen

Status:
fixed in `9ebc80985` (production names `4429fd4b3` and `b98638490`,
spelling `ffc03346d`),
2026-09-28;
found wiring H16.
`rendered-sheets.test-fixture.ts` said it renders "EVERY MODEL-FACING SHEET" so a guard about what the models read
checks every sheet at once;
it held fifteen:
of the 24 `build…Messages` builders the package defines it rendered 14,
and it rendered none of the picture readers,
the other selection slates,
their decline texts or the typed decision.
The fixture now renders all of them,
split into four sibling fixtures under max-lines.
Rendering them from production text took names for four texts built inline where they are sent
(the picture reader's request,
the translate challenge task,
the archive slate's task and decline),
each moved byte for byte,
and exports for the rest.
The first read found one defect:
the picture readers' instruction said "summarise",
which en_CA writes "summarize";
the picture reading key hashes the instruction,
so no version moves.
The full suite then passed on `ffc03346d` (0 FAIL).
Prevention:
`rendered-sheets-census.unit.test.ts` scans `src` for every exported `build…Messages` function
and fails when no fixture renders it (over the old fixture it lists ten).
Sheets built without that naming
(the slates' tasks,
criteria and decline texts,
the picture readers,
the typed decision)
are not in the census;
a new one of those still needs its own entry.

## X19: a unit test bought a live web search and wrote the real lookup cache

Status:
fixed in `d82dfe559`,
2026-09-28.
`preparePassEntry` read three things outside the pipeline itself:
the work-title lookup and the cited references,
each with the Exa key from `process.env` and a cache under `~/.cache`,
and the corpus names at the pin.
The unit suite runs under the root `mise.toml`,
which decrypts `.env.local.json`,
so the key is set in every test process (checked as a boolean:
`TRANSLATION_REPAIR_EXA_API_KEY` is non-empty).
H16's red guard (`31c7d3f00`) prepared an original naming an invented title through that path:
the preparation bought one Exa search for it
and wrote the answer to `~/.cache/translation-repair/lookup/84fcabcd8c13….json` at 19:42 UTC.
The five other lookup records whose titles a fixture also names are corpus titles live runs bought.
The three reads now come through `PassOutsideReads` (`corpus-run/pass-outside-reads.ts`),
required by `preparePassEntry` and `runPassPreparation`;
the run hands over `RUN_OUTSIDE_READS`,
and every test hands over `NO_OUTSIDE_READS`
(`pass-outside-reads.test-fixture.ts`) or a reader of its own,
so a test that leaves the seam out does not compile.
`pass-outside-reads.unit.test.ts` drives the wiring:
each reader asked once about the original,
and what each returns reaching the sheets.
The footnote-lifecycle test had worked around the same hazard by asserting its original names no title;
that assertion gave way to the fixture.
The stray record could not be removed from this session (the removal was refused);
it holds only the invented title's search,
and the owner can delete it.
The fix left one caller supplying the run's readers itself:
`runEntryPipeline` handed `RUN_OUTSIDE_READS` to `runPassPreparation`,
so `settleEntry` took no readers
and `pass-entry.unit.test.ts` still prepared every entry through the key,
the real caches and the corpus clone,
until 2026-09-29 (M68).

## X23: the run client and the provider gate read the process's keys by default

Status:
fixed 2026-09-29 for the provider gate (`4138c5ade`) and the run client (`3510c8336`);
the provider clients' own transport defaults went in X24.
`configureProviders` read the four provider keys,
the Hyper pace and the Bedrock ledger's place from `process.env`,
and `createRunClient` and `assertRequiredProvidersReady` took an optional transport defaulting to the live one.
`run-config.unit.test.ts` wrote one or two keys into `process.env` and built the client on the rest,
so every one of its client cases ran on the OpenRouter and Bedrock keys the suite inherits from `mise`
and the real Bedrock ledger's path;
its refusal cases had to clear each new key by hand,
and its own comments record two landings,
on 2026-09-03 and 2026-09-07,
that turned four of them into builds.
The format 2 baseline shows it:
of the four unconfigured-provider arms in `createRunClient`,
the Bedrock and OpenRouter ones never ran in the whole suite (`census-qilSwP`).
The wiring cases hand over a transport,
so none reached a provider;
of the 189,941 lines in the live Bedrock ledger,
none has a prompt under 200 tokens,
the size of a fixture's message
(`x23-ledger-tiny.mjs` in the audit's scratch folder,
read 2026-09-29).
`configureProviders` now requires `env` and `transport`;
`runClientFrom` builds the run client over both,
and `createRunClient`,
which every runner calls,
hands it `process.env` and `fetchTransport`.
The client cases build on environments they hand over,
the Bedrock one over a ledger in a throwaway directory,
and the one case of `createRunClient` clears every key and expects the refusal.
The stand-in caller that refused by name for each absent provider went with it:
an absent provider's meter is `UNCONFIGURED_METER`,
which reads dry (`provider-meters.ts`),
and the route never picks a dry provider (`routeProviderFor` in `budget-routing.ts`),
so no call could reach one;
the router now takes only the configured callers and asserts one is present where it calls.
Prevention is M43's:
a builder that reads a key,
a cache or a transport takes it as a required parameter,
and the process's own is named once,
in the function runners call.

## X24: parameters that fell back to the live transport or the corpus clone

Status:
fixed 2026-09-29.
A census of every parameter in package source whose default reaches past the process
(`x24-default-callers.mjs` in the audit's scratch folder,
beside a search for defaults naming `fetch`,
`fetchTransport`,
`process.env`,
`homedir()`,
a cache directory or a corpus reader)
found three kinds.
The five provider clients (`createSyntheticClient`,
`createHyperClient`,
`createDecisionsClient`,
`createOpenRouterClient`,
`createBedrockClient`) defaulted their transport to the live one;
every test handed one over,
and the one runner that did not,
`budget-sample.ts`,
meant the live one.
Six corpus readers defaulted to the clone:
`readCorpusNames` its lister and reader,
`loadEntry` its source reader (M70),
and `censusEntry`,
`gatherControlCases`,
`gatherRelabelCases` and `sampleBenchSlices` their pin;
every test handed over a fixture or a throwaway clone,
and the runners that left them out meant the run's pin.
Each is now required:
the runners hand over `fetchTransport`,
`readCorpusFile` or `RUN_CORPUS_PIN`,
`slice-census.ts` hands over the run's pin for an entry whose artifact predates the recipe,
where it had left the pin out,
and `draw-entry-load.unit.test.ts` hands its refusal cases a reader that refuses any corpus read.
The third kind was read and kept:
the operator dials `resolveSpendCeilingUsd`,
`resolveHardCapMinutes`,
`resolveStragglerGraceMs` and `resolveWriterGraceMs` default `raw` to their variable,
which is configuration a run is launched with rather than a key,
a cache or the network,
and their tests hand `raw` over.
Library code reading a key from `process.env` in a function body turned up only in runners
(`model-catalog.ts`,
`roster-card.ts`),
which the runner batch of T8 reads.
