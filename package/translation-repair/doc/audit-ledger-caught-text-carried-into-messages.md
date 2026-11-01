# Audit ledger: caught text carried into messages

Refusals,
log lines and stored details that carried a caught error's own text,
which can quote a provider body or a document.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B34: marked refusals carrying a caught error's text

Status:
fixed 2026-09-30 (UTC) in `a1e4accbd`,
guard red first in `7360aae58`,
found the same day in the B33 census's output (`reason-census.txt` in the audit's scratch folder),
which lists five reasons built from `caughtValueText(error)` and one from `error.message`.
`ArtifactParseError` declares `messageNamesOnly`,
so `refusalText` repeats its message wherever a refusal is printed,
on the promise that the message holds only authored sentences,
counts,
names and what the operator typed.
The inventory in `message-names-only.unit.test.ts` checks what a marked class's own constructor interpolates,
and could not see a throw site handing it another error's text.
Six sites did.
Two of them quoted the stored value:
a pipeline digest refused by forwarding `assertPipelineDigest`'s `TypeError`,
which prints the value it received,
and a preparation identity refused by forwarding `PreparationIdentityError`,
whose first message printed the value.
Both values are hashes a run writes,
never corpus text,
but a tampered or foreign artifact holds whatever string it holds,
and the field readers' own region says they refuse "with the dotted path of the field rather than its content".
The comparison reader forwarded `ArtifactComparisonError`,
an unmarked class its throw sites handed finished sentences;
those sentences held names and numbers,
and nothing checked that they would keep to that.
The corpus verification wrapper forwarded `ArtifactPreparationMismatchError` for each lane's slice count,
from a check that could not fire:
the reader already refuses a lane whose ledger does not hold one row per prepared slice
and a lane count other than its ledger's length,
and the verification checks the preparation's count first.
The remaining two forwarded from narrowed catches:
the change-set reader from `AssemblyContractError`,
which is marked,
and the row relations reader from `DeliveryCoherenceError`,
which is marked,
and `WordingCoherenceError`,
which took finished sentences from its throw sites and so could not carry the marker.

The digest module's own words were wrong too:
the `TypeError`,
its `@throws`,
and `isDigestShaped`'s `@returns` said a digest is 64 lowercase hex characters,
and every digest starts with the scheme name `sha256-tree-v1:`;
the `PipelineDigest` type's TSDoc said a digest cannot be told from a git object id by shape,
which the scheme name makes false.

Fix:
the digest and identity readers refuse by shape
(`isDigestShaped`,
and a new `isPreparationIdentityShaped`)
in words built from the constants the checks read
(`PIPELINE_DIGEST_SHAPE`,
`PREPARATION_IDENTITY_SHAPE`),
then narrow.
`ArtifactComparisonError` takes a closed fault and writes its own sentence,
carries the marker,
and the comparison reader's catch narrows to it and rethrows anything else;
`comparisonRowDifferences` returns `ComparisonRowField`,
so the fields a sentence lists are names by type.
`WordingCoherenceError` writes its sentence from a slice index and a closed fault kind,
in `7360aae58`.
The corpus verification drops the wrapper and both lane checks.
The guard's new case finds every construction of a marked class whose arguments turn a caught error into text
(a template holding the binding,
`String` or `caughtValueText` of it,
or its `message` or `stack`),
reads which classes the enclosing catch narrows to,
and fails on one missing from `FORWARDING_SITES`
or listed as forwarding from a catch narrowed to anything but marked classes.
Its red run listed the comparison reader as narrowed to `LaneComparisonError`,
the live comparison's class rather than the one `compareLanes` throws;
the fix commit corrected the entry,
and a comment on the red commit records it (M77).

## B166: two lookups logged a caught value's own words, which can quote a provider body

Red in `290110436`,
fixed in `2d5141018`.

Found on 2026-10-05 by the audit of B134.
`workTitleLookupLines` (`work-title-lookup.ts`) and `citedReferenceBlock` (`cited-reference-lookup.ts`)
logged `String(error,)` of whatever their per-title and per-page catches caught.
An endpoint body that is not JSON fails `response.json()` with V8's `SyntaxError`,
whose message quotes ten characters of it:
on the build of `e300e6c33` a stub body of cat prose reached both warnings quoted (the fixing agent's measurement).
The package's rule for a logged failure is `refusalText`,
which repeats a message only where its class declares it free of quoted text.

The fix:
both catches log `refusalText` of the caught value,
so a parse failure logs `refused by SyntaxError`
and `CacheFileUnreadableError` still logs its filesystem code and path.
`WorkTitleLookupError` and `CitedReferenceFetchError` forward a message and cannot be marked,
so their status code now logs as the class name alone;
a field or a marked sentence of their own would bring it back.
B180 marks both classes,
and the status is logged again.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B232: a model reply's opening reached log lines and stored details through the parse refusal

Red in `afee9dcda`,
fixed in `4c26a48a0`.

Found by a census of every place a caught value becomes text,
and merged on 2026-10-06 (UTC).
`parseModelJson` (`model-content.ts`) returned `String(error,)` of its `JSON.parse` failure as `detail`,
and `readJsonPastFalseStart`,
`readJsonBeforeTrailingText` and `readPayload` (`stream-delta-scan.ts`) logged or kept the same.
V8's `SyntaxError` quotes ten characters of the text it refused:
a model's reply,
a provider's body or a stream frame.
The detail reached the schema-mismatch detail of `chat-json-outcome.ts` and the `stage-call.ts` lines,
while `lookup-cache.ts` already said this detail must never be repeated.
The red cases are the parse refusal of a reply opening with a ten-character word,
asserted against `refused by SyntaxError`,
and the package case of the scan B237 records.

The fix:
`parseModelJson` returns `refusalText` of the failure,
and the three readers go through `parseModelJson` where each caught a parse itself.
The parser is still a bare try around `JSON.parse`,
so the stream path does no more work per frame.
The reply's own opening and the stop reason stay in the outcome;
V8's reason and position are lost.

The lead ran 35 named test files before the fix was committed,
the scan among them,
with the 37 source scans and a clean type check.
The whole suite's runs on this change are recorded in B235.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B233: a failed model exchange was logged and recorded by its own message

Red in `afee9dcda`,
fixed in `4c26a48a0`.

Found by the census of B232.
Twenty-two functions turned a caught exchange failure into text with `String(error,)` or `caughtValueText`,
among them `attemptStageCall`,
`attemptDecisionCall`,
`reaskElsewhereNudged`,
`reuseTwinOrBuy`,
`readImagePair` (a rejected `allSettled` reader),
`runPick`,
`readDryness`,
`gateProvider`,
`reportModelHealth`,
the coverage probe,
`runRepairBenchmark` (two log lines and the stored `detail`),
and the abort-path lines of `repair-refine-step.ts`,
`repair-slice-buy.ts` and `translate-slice-attempt.ts`.
Three kinds of message can ride there.
A runtime `fetch` rejection for a header it cannot send quotes the header's value,
which is the key
(measured as a `TypeError` in the model-provider probe of 2026-10-05,
by the agent's report);
V8's JSON parse failure quotes the opening of the text it refused;
and `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19) excerpts the provider's reply body on purpose.
The red cases throw a `TypeError` quoting a stand-in key (`quoting-failure.test-fixture.ts`,
invented)
and a `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19) with a body,
and the logged lines held both.

The fix:
`exchangeFailureText` (`exchange-failure-text.ts`) prints a marked class's own sentence,
`refused by SyntheticHttpError with HTTP <status>` (renamed `ProviderHttpError` on 2026-10-19)
for a provider's status failure,
and the class name for anything else.
Every site named here uses it.
The lead's edit on the agent's change:
the model-health probe names an unreachable model's failure class once,
where it read `UNREACHABLE (TypeError): refused by TypeError`.

What was lost:
no log line carried a provider's own account of a refusal any more,
only its status,
and two comments (`completion-shape.ts`,
`request-size-refusal.ts`) that said the account stays on `bodyExcerpt` "for the log" became untrue for every class.
That account is the evidence a model is dropped on.
B266 and B268 record the follow-up that put it back by a typed read.

Not cased:
the abort-path lines,
the provider gate and the two probe commands have no behavioural case and rest on the scan (the agent's report).

Open to the owner's veto:
a separate `exchangeFailureText` where a wider `refusalText` is the alternative;
the picture reader's two lines left printing the local reader's failure text at this commit
(held in the scan with that reason,
and closed by B269).

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B234: the retry ladder logged a raw transport rejection and wrapped a non-Error without its cause

Red in `afee9dcda`,
fixed in `4c26a48a0`.

Found by the census of B232.
`exchangeWithRetry` (`transient-retry.ts`) logged `String(thrown,)` on every retry of a thrown transport failure,
the one place the unsendable-header rejection reaches before any marked class wraps it,
and `attemptExchange` turned a thrown value that is no `Error` into `new Error(String(error,),)` with no `cause`.
The red cases throw the quoting `TypeError`,
and a bare string,
and read the retry line and the wrapped error.

The fix:
the retry line goes through `exchangeFailureText`,
and the wrapper states `the transport threw a value that is not an Error` with the value as `cause`.

Open to the owner's veto:
the wrapper sentence no longer carries the thrown text.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B237: no scan held the rule that a caught value never becomes text whole

Scan and red cases in `afee9dcda`,
wired and green in `4c26a48a0`.

Found by the census of B232.
Nothing read a caught value turned into text whole:
`caught-errors-kept.unit.test.ts` accepts any use of the binding,
and the inventory in `message-names-only.unit.test.ts` reads only the constructors of marked classes
and the sites that hand them a caught text.

The fix:
`caught-value-text.unit.test.ts` flags the forms listed in its module comment
(the argument of `String`,
`caughtValueText`,
`JSON.stringify` or a logger,
a template,
`.message`,
`.stack`,
destructuring),
keyed by file and enclosing function with a reason for each held function,
over a fixture case that finds every form.
Run on the agent's base it found 22 functions fixed by B232 to B236 and 22 held (the agent's count);
on the lead's red tree its package case listed 22 functions outside the held list.
The lead wired it into the `source-scans` task,
which ran 37 files from then.

Not reached,
by the agent's account:
a copy into another binding,
a value carried across a function under another name,
and text reaching a sink through a callback.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B269: the local picture reader's failure line repeated the tool's output and paths

Red in `54b3d2535`,
fixed in `a29a9dc42`.

Found by the census of B232,
which held the two lines with a note,
and fixed by the follow-up B266 records.
`readImageWithOcr` and `askDeterministicReader` printed the local reader's whole failure text,
which holds the command line,
the scratch paths and standard error,
against their own comment and `decoderFailureLine`.

The fix:
`LocalProgramFailedError` (new `local-program-failure.ts`) states an exit code,
a filesystem code or that the program is missing,
and `readerFailureText` names the class and the code;
the two held entries leave the scan of B237.
The red cases are in the `image-ocr` and `image-reading-pair` test files.

Open to the owner's veto:
`LocalProgramFailedError` for the local reader.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B270: the refusal printer's frames could include a line of the message, and one scan read a property as a use

Red in `54b3d2535`,
fixed in `a29a9dc42`.

Found by the follow-up B266 records.
`framesOf` (`corpus-run/cli-refusal.ts`) kept every stack line starting with `at `,
so a message holding a line break followed by `at ` printed that line of itself among the frames.
`readClause` of `caught-errors-kept.unit.test.ts` counted a property or a key spelled like the binding as a use of it.
The `refusal` parameter of `defectOf` was typed as any `Error` class,
while every caller passes a marked one.

The fix:
the printer cuts the frames off where the stack's header ends;
the scan skips name positions;
and `defectOf` takes only a class that declares its message safe,
so the compiler holds it.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".
