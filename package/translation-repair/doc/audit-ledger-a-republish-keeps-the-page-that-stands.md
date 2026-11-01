# Audit ledger: a republish keeps the page that stands

Round w398 made a page the strict grammar refuses ship as the archive kept at the slices that break it (B1207),
and the re-check of that round found that a republish could overwrite a page that parses with such a page (B1236).
This round (w401,
2026-11-01 UTC) closes that gap in `page-republish.ts`,
and records the cache versions and the slips of the round.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1260: a republish never replaces a page that stands with a worse one

Red in `320df246bf`,
fixed in `8c607fdeb3`,
its lint findings fixed in `0bfa9e6a0`.

The owner's answer of 2026-09-27 (`design-commitments-shipping-and-the-archive.md`) is that a pass rewrites a stored
page to the running build's reading,
and that a page that does not parse stays as it was and is reported.
`rebuildPage` in `page-republish.ts` kept the old page only when `publishFixedPage` threw.
Since round w398 (B1207) `publishFixedPage` does not throw when the strict grammar refuses the composed page:
it ships the archive kept at the slices that break it (`page-grammar-archive-kept`),
or the composed page itself (`page-unparseable`).
A resume over a stored page that disagreed with the running build could therefore overwrite a page that parses with a
page that kept archive slices the old page had translated,
or with one the site build will not take.
B1236 recorded the gap from the re-check without reading the function;
this round read it and reproduced the overwrite.

The red evidence is on `93dba14869`,
through `republishSettledPages` over a settled artifact and a stored page that parses and disagrees.
Both cases returned `{ kind: 'republished', why: 'disagreed' }` where `left` was expected,
and the log read `publish: wrote 1 slice into a page of 81 characters` and
`publish: shipping with defect page-grammar-archive-kept` for the case of a page that parses with the archive kept,
and a page of 83 characters with `publish: shipping with defect page-unparseable` for the case of a page that does not
parse even so (corrected on 2026-11-01 UTC by round w400:
the entry had given 81 for both).
The two cases for a missing page passed on the parent,
as they are meant to (B1262).
The fixtures use no corpus text:
the refused wording is a curled JSX string literal of a cat-themed invention,
and the unparseable archive has a brace opened and never closed.

The fix is the `fallback` option of `publishFixedPage` (B1261).
`rebuildPage` asks for `refuse` when the stored page disagrees and `ship` when it is missing.
Under `refuse`,
`refuseFallbackPage` throws `FallbackPageRefusedError` when the parsing page carries a defect,
before the content checks,
the directory and the write.
The message names the defects (`page-grammar-archive-kept` or `page-unparseable`) and nothing of the page,
and `refusalText` prints it because the class is marked `messageNamesOnly`.
The existing `catch` of `rebuildPage` reports it as `left`.
A first publish (`pass-entry-persist.ts`,
`interim-page.ts`) passes no option and ships as before.

Where it shows:
a pass started in a runs directory that already holds settled artifacts whose stored page disagrees with the running
build,
which is a resume the operator opted into.
No recorded input is known to make a rebuilt page fall back;
the owner's answer is a rule,
so it holds without one.

## B1261: the seam that tells `rebuildPage` the page would fall back

Three shapes were weighed before anything was written.
Writing the page and restoring the old one when it fell back leaves a window in which the stored page is the worse one
and a crash keeps it,
so it was rejected.
A seam that composes and checks without writing,
called from `rebuildPage` before `publishFixedPage`,
would run `composeSourcePage` and `pageThatParses` twice and leave two readings that can drift.
An option on `publishFixedPage` reads the grammar once,
in the call that already does,
and throws before the first write;
it was chosen.
The default is `ship`,
so no caller that did not ask changes.

## B1262: a missing page ships its fallback

There is no old page to keep,
and "a run always ships" decides (the owner,
2026-09-27).
The page the fallback gives is the archive kept at the slices that break it,
or the composed page with `page-unparseable`,
shipped as a first publish ships it,
with the defect named on the log line `publish: shipping with defect <check>`.
`page-republish.unit.test.ts` pins both,
with the bytes of the page written.
A limit:
the outcome of the row reads `republished` and does not say the page fell back.
The log line and `verify-published` (B1231) do.

## B1263: the cache versions

The rule is step 4 of `mistake-prevention.md` (M25,
M28).
The audit
(`slice-cache:cache-account-audit -- --runs-under` the agent folder,
run on the tree of this round after its three commits)
lists the eight constants at the values of B1228
(consolidate 44,
consistency 10,
lane contest 19,
refine 20,
repair slice 51,
page title 9,
pairing 3
and translate slice 37)
and 14,049 slice-cache records under 430 runs directories,
the newest 2026-10-31T10:04Z (`XingZ6023`),
with 5 of 8 versions set after it.
Three directories could not be listed (`EACCES`,
all `pgvector` scratch folders of 2026-09-03),
which leaves their runs out of the count.

- No version moves.
  The change decides which page a resume writes into the published tree.
  No key reads that page:
  the key hashes the version,
  the shape of the run and the texts a stage is given,
  and no stage asks or accepts anything differently.
- The three source commits of this round (`8c607fdeb3`,
  `0bfa9e6a0` and `0530b95c54`) appear in the audit as source
  commits no account names,
  with every version listed as one they ride inside
  (corrected on 2026-11-01 UTC by round w400:
  the entry named two and said the audit ran after three commits;
  `0530b95c54` is the third,
  and it touches `provider` as well as a test of `corpus-pass`).
  The red test commit holds no source and is not listed.
  The first two touch `page-check` alone,
  and `rg translation-repair-page-check package --glob package.json` lists `slice-cache`,
  `probe`,
  `corpus-pass` and `run` as its only dependents,
  none of which is a package the sheets and floors of a stage import.
  `0530b95c54` renames a parameter of `markingAfterExchange` (B1264) and changes no behaviour:
  the router calls the function with the same arguments and rebuilds the same error,
  and a name of a parameter is in no text a key hashes,
  so the reason that holds for it is that no value any key reads changed,
  not that its package is outside the stages' imports.
- The five versions set after the newest record have no record at their numbers,
  so even a change that did reach a key would ride inside them.

## B1264: the slips of the round and what stays open

Three cases of `claude-schemas.unit.test.ts` of `corpus-pass` were red on `93dba14869`,
and failed the same when the file ran alone
(the rotation sweep of round w399,
whose lone run's output is outside the repository).
They list the files that declare an answer schema,
7 more than the list holds,
the shapes the search leaves unread
(`editor-fidelity-probe.ts:91` and `corpus-pass.ts:170`),
and the functions that hand a sender a builder call made from their own parameters:
`markingAfterExchange`,
then `createRoutingClient`,
`runClientWithFloors`,
`runClientFrom`,
`createRunClientWithFloors`,
`createRunClient`,
`probeRunClientFrom`,
`cardsClientFrom`
and `newCardsClient`.

The cause is the test's reading and not a change of behaviour.
`77e37c62f1` moved the router's `chooseProviderMarked` into `markingAfterExchange`
and gave it a parameter named `choose`,
which it calls with its other parameters.
A name a file neither declares nor imports stands,
in the search,
for the function of that name
(`standsForKnown` in `claude-schema-usage.test-fixture.ts`),
and the family declares a forwarder `choose` in `pass-consistency-carry-choose.ts`.
So the helper was read as a forwarder,
the files that call it as the files that write its arguments,
and each caller of those in turn.
The helper reaches no sender and no builder.
It only rebuilds an error after the router's decision.
At `abb1e953bc` the same code was `chooseProviderMarked`,
which called `chooseProvider` by a name of its own and had no parameter that was a function,
so the cases passed.

The fix is in the code shape.
The parameter is named `decision`
(`provider-router-exchange-mark.ts`,
its call in `provider-router.ts` and its test),
and the 3 cases pass again with nothing added to the lists and nothing narrowed.
A case of the same test pins the reading both ways:
a helper that calls its parameter `choose` is listed beside the forwarder `choose`,
and the same helper with `decision` is not.
That case records the reading,
and the lists the three failing cases of `claude-schemas.unit.test.ts` compare name a new collision when one happens.
The search stays fail-closed.
It cannot tell a parameter that holds the forwarder from one that holds another function,
and a file that really sends an answer schema through a function it takes as a parameter still fails.

Slips of the round:

- The first commit of the fix shipped with 3 oxlint warnings and 5 errors:
  two `chain-per-line` findings,
  one `invocation-depth-per-line` (a hoist in the red commit's helper),
  four `TS2322` for a test `fn` without `async`,
  and `index.ts` at 304 of 300 lines.
  The lint ran after the commit was started and was not read before it.
  `0bfa9e6a0` fixes them,
  and the three new exports sit in `page-guard-reports.ts`,
  the module that already exports `pageThatParses`.
- The whole source-scan suite,
  run once on the docs commit,
  found 3 things the oxlint run could not:
  `FallbackPageRefusedError` was not in the marked-class inventory of `message-names-only.unit.test.ts`
  and `checks.join` was not among its named parts,
  a `@returns` of the new test helper opened with an article,
  and the new test file held two top-level awaited describes.
  The class and the part are listed,
  the article is gone and the describes are one.
  The 3 files then passed alone,
  with oxlint and `lint:types` on `page-check` and `source-scan`.
- The whole source-scan suite ran a second time,
  for the `claude-schemas` fix,
  and found the word "above" in this entry's own text as a reference by position.
  It is worded by what it names now.
- `lint:types` of the four dependents first ran with no `dist` of their own,
  and printed `TS2307` and the implicit `any` it causes;
  it passed after their `build`.
- Waits:
  a monitor on a PID taken from `pgrep` and one that matched a process by name,
  both stopped,
  and several placeholder monitors.
- One read of a harness task output file under the session's `/tmp` folder (empty),
  `rg` with `--max-count` twice and a `git log` with a count cap,
  a `tail --lines` on this file,
  a `cut` in a wait condition,
  four Edit calls to this file in one block,
  an oxlint run started while the fix commit's hook was still running,
  and one read of a 406 KB audit file that the tool refused and a second read of a slice of it.

Open:

- The content checks of the publisher (`page-agreement`,
  `front-matter`,
  `archive-original`,
  `destinations` and the others) still ship their defects over a standing page on a resume.
  The owner's answer names a page that does not parse,
  so this round did not extend it.
  Whether a defect other than the grammar's should also keep the old page is the owner's to say.
  The defects of `composed.defects`,
  such as `page-source-unparseable`,
  are among them:
  `refuseFallbackPage` reads only the defects of the grammar's fallback,
  so a republish over a standing page that composed with one of them still writes (added on 2026-11-01 UTC by round w400;
  the header list of `page-republish.ts` names them as well).
- On a refusal the log still carries the line of `pageThatParses` that begins `the page does not parse`
  and ends `keeping the archive at the slices that break it`,
  which reads as a promise the republish then does not keep;
  the line after it,
  `REPUBLISH LEFT`,
  says the page stayed.
  Under `refuse` the revert search of `pageThatParses` also runs to its end before the refusal,
  and its reads and its log lines are spent on a page nothing writes (added on 2026-11-01 UTC by round w400).
- `restoreTypography` logs the first characters of a text it refuses as prose
  (`[typography-prose-mask] noteRefusedMask`),
  which in a real run is corpus text in a log line.
  It was already so and is not changed here.
- `doc/audit/standards-review-gate-shadow.md` is modified in this tree by a hook that is not part of this round,
  and is not committed.
