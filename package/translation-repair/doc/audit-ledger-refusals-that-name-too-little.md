# Audit ledger: refusals that name too little

Refusals and failure lines that named too little to act on,
or named the wrong cause.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B69: the prompt payload store's refusals named nothing that refused

Found by the fifteenth coverage batch,
reading `prompt-payload-store.ts` beside the cold stretches of its refusals.
Every refusal the store raised read `prompt payload read failed for <digest>`
(or `write`),
whichever of its checks refused:
a record that could not be opened,
text that was not JSON,
another format version,
each field of the stored reply,
or a write the filesystem refused.
The error declares its message names-only,
so that message is all a `TALLY` line prints (`refusalText`),
and the cause it carried for a filesystem or parse failure was never shown.
An operator could not tell a corrupted record from a format change from a full disk,
and the one refusal case
(reply text 7)
asserted the class alone,
so it passed whichever check fired.

Fixed (`2d3b02c45` red,
`a56cde819`):
the error carries a `reason`,
in its message after the digest and as a field,
naming a JSON path in the record,
the envelope,
the format version this build reads
(never the stored one),
or a filesystem code from `failureName`.
Its TSDoc says what to do about one:
where the record lives,
that a read refusal stops every later run too,
that deleting the record asks the provider again at that call's cost,
and that a write refusal lost a reply already bought;
the README points there beside its other `INCOMPLETE` sentence.
The parse has a narrow `try` of its own,
through `requireJsonSyntaxRefusal`,
so the wide catch that rethrew the store's own refusal went.
Taking the reason out of the message again failed the 14 red cases and nothing else.
Two red expectations were corrected in the fix commit:
the not-JSON reason no longer names `SyntaxError`,
the only class that narrowing lets through,
and the not-an-object record became a JSON string,
since `isJsonRecord` lets an array through to the version check (M58;
the red commit carries a correcting comment).
`f7838ffb3` then asserts each refusal as `tallyErrorText` prints it,
whole,
so the reason is shown to survive the line's cap.

Recurrence:
a marked class names what refused,
since its message is all an operator sees;
one sentence for several checks also lets a case pass on the wrong check.

## B155: a decisions reply refusal carried an undefined cause, and a mistyped field read as missing

Red in `df0bcb85a`,
fixed in `ba243b3c3`.

Found 2026-10-05 (UTC),
reading the MiMo trial's production diff once the trial was merged.
The trial gave `DecisionReplyShapeError` a `cause`
and passed `{ cause, }` to `Error` whether or not one was supplied,
so every refusal but the parse failure carried an own `cause` of `undefined`.
Beside it,
`readDecisionReplyBody` reported an `answers` that is an array,
or a `model` that is a number,
as missing.

The fix:
the constructor spreads `cause` only when one is supplied,
as `CreditsShapeError` does,
and a present but mistyped field reads `answers is not an object` or `model is not a string`.
The cases assert each refusal's class,
whole message and cause.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B156: the listing catch read a fault of the process as a corpus refusal

Red in `df0bcb85a`,
fixed in `ba243b3c3`.

Found 2026-10-05 (UTC),
checking the trial's decision for the owner on `gitOutput` (`corpus-source.ts`,
`33fc7a459`).
The trial wrapped every throwable of the listing call as `CorpusReadError`,
on the reason that nano-spawn's one raw throw is an `options.input` write the call never makes,
and cited a scratch probe.
nano-spawn 2.1.0 also throws raw before any child exists:
`source/options.js` reads `process.cwd()`,
which throws `ENOENT` once the working directory is removed,
and `source/context.js` throws a `TypeError` for a command part that is no string.
Wrapped,
both read as a corpus failure whose message advises a check of the clone.

The fix:
the catch wraps a `SubprocessError` alone and lets anything else propagate as itself,
as it did at `28303c42a`,
so the error's own class and message name the fault;
the comment states the fact and names the nano-spawn files and the case that hold it.
A case removes the working directory under a child process and reads the listing's failure back.
The text and byte readers go through `execFile`,
and with the working directory removed both still read.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B167: a Bedrock ledger line that would not read named the line and not the file

Red in `290110436`,
fixed in `2d5141018`.

Found on 2026-10-05 by the audit of B134.
`BedrockLedgerShapeError` (`bedrock-ledger.ts`) is marked `messageNamesOnly`
and said `Bedrock spend ledger line 2 violated expectations: not valid JSON (SyntaxError)`.
The ledger is repaired by hand,
since a line is never skipped,
and `TRANSLATION_REPAIR_BEDROCK_LEDGER` can move it off its default,
so the refusal did not say which file to open.

The fix:
the class takes the ledger's path and names it before the line,
and every throw site in `entryOf` passes it;
`path` was already a named part of the marker inventory.
The torn-line case in `bedrock-ledger.unit.test.ts` now asserts the whole refusal,
and the case pinning each field's refusal carries each file's path.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries":
a marked class names what refused,
and for a file repaired by hand that is the file.

## B180: two lookups logged their failure by class name alone, losing the HTTP status

Red in `e3c217fb0`,
fixed in `ab11a69d6`.

Found on 2026-10-05 (UTC),
as the item ledger B166 left open.
`WorkTitleLookupError` (`work-title-search.ts`) and `CitedReferenceFetchError` (`cited-reference-fetch.ts`)
forwarded a `message` that held the endpoint's body,
so they could not carry `messageNamesOnly`,
and `workTitleLookupLines` and `citedReferenceBlock` log `refusalText` of what they catch:
a 401 from the search endpoint and a 429 from the contents endpoint both logged `refused by` and the class name,
with no status.

The fix:
both classes take a `detail` phrase each throw site authors,
declare `messageNamesOnly`,
and build `search ${detail}` and `contents ${detail}`;
the non-2xx throw hands them `responded` and the status,
and no longer reads the response body or repeats the query or the url,
which the lookups' own log lines still name.
Both names joined `MARKED_CLASSES`;
`detail` and `String(status,)` were already named parts.
Each lookup's test file asserts the whole logged line for a refused answer with a prose body.

Open to the owner's veto:
the thrown message no longer carries the query or the url,
which only the lookups' own log lines now name.

Open when this entry was written,
closed in B220:
a `fetch` that rejects,
for a name that does not resolve or a connection reset,
still logs as `refused by TypeError`.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries":
a class that cannot be marked loses its status in every log line that goes through `refusalText`,
so a class a log line must explain is built from named parts.

## B220: a request the transport rejected left the lookups as the transport's own error

Red in `63567b4f5`,
fixed in `843bd6ce0`.

Found on 2026-10-05 (UTC),
as the item ledger B180 left open.
`searchWorkTitle` (`work-title-search.ts`) and `fetchCitedReference` (`cited-reference-fetch.ts`) awaited `fetch` bare.
A rejection passed on as the runtime's `TypeError`,
which is unmarked,
so the lookups' log lines read `refused by TypeError`,
naming neither the endpoint nor what failed.

The fixing agent's probe of the runtime,
on local addresses and an invented header value:
a name that does not resolve rejects with a cause carrying a system code;
a refused connection rejects with a cause carrying none;
and a header value holding a line break rejects with no cause and a message that quotes the value.
So a system code cannot tell a network failure from a request the transport will not send,
and the rejection's message cannot be repeated,
since the header is the key.

The fix:
a rejection before any answer is rethrown as the lookup's own marked class,
`WorkTitleLookupError` or `CitedReferenceFetchError`,
with the rejection as its cause and never in its message.
The detail names the endpoint and says the request failed before any answer came back:
either the network could not be reached,
or the transport refused to send the request,
as it does for a key or header holding a line break;
and it says to check the connection and the key.
The agent's first wording said the network had failed,
which is untrue of the second case,
and the lead sent it back.
A connection that fails while the body is read says the network was lost while the endpoint was answering.
The caller's abort passes on as it came,
and so does a body that is not JSON,
whose parser message quotes the text it refused (ledger B166).

Cases in `work-title-lookup.unit.test.ts` and `cited-reference-fetch.unit.test.ts`:
the rejecting transport,
the header value holding a line break,
the body cut while it is read,
and in the first file the log line.
Seven of them failed on the lead's red tree,
where the header case showed the transport's own text,
the invented value in it,
as the refusal.
A fixture file holds the response whose body fails,
which both test files share.
One case name and two TSDoc summaries are the lead's rewording,
to say what the final detail says.

Open to the owner's veto:
a body that is not JSON stays the parser's unmarked error,
logged by class alone.

Open:
`work-title-search.ts` still stands in the list of modules without a test of their own,
its cases in the lookup's test file.
Still so at `94de8258e` (`own-unit-tests.unit.test.ts`),
with no follow-up launched for it.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries":
an authored account says only what holds for every cause it covers,
and names the possibilities where it cannot tell them apart.

## B235: a cut stream's message repeated its cause and stayed withheld from every refusal printer

Red in `afee9dcda`,
fixed in `4c26a48a0`,
with one stale case repaired in `495f040b6`.

Found by the census of B232.
`StreamCutShortError` (`stream-cut.ts`) interpolated `String(cause,)`,
so it stayed withheld from `messageNamesOnly`,
and every `refusalText` caller printed `refused by StreamCutShortError`
while a `String` of it printed whatever the stream's failure said.
`stopReading` (`stream-drain.ts`) logged the cancel failure by `String` too.
The red cases cut a stream with a quoting cause and a stalled cause,
and cancel a body whose cancel throws the quoting failure.

The fix:
the class declares `messageNamesOnly`,
the inventory names the three parts it builds from,
and the cancel line uses `refusalText`.
The agent's patch stated the cause through `refusalText`;
the lead changed that to `exchangeFailureText`,
so a cause that is a provider's status failure keeps its HTTP status,
and a case asserts it.

The fix left the suite red at one case.
`stream-idle-guard.unit.test.ts`,
"reports caller steering as itself,
not as a stall",
still asserted that the cut's message contains the steering error's own words.
The lead had run 35 named test files before the commit and that file was not among them (M122).
The whole-suite run on `4c26a48a0` in a side checkout found it:
1,624 `[PASS]` lines and 3 `[FAIL]` lines,
all three that one case and its two enclosing suites,
with the lint finding nothing and the 37 source scans passing.
`495f040b6` makes the case assert what it is named for:
the thrown value is a `StreamCutShortError`,
its `cause` is the caller's own error by identity,
and its whole text is `StreamCutShortError: hf:whiskers: stream cut after 0 characters (refused by Error)`.
The whole suite then ran on `8ce45ae86`,
which holds this fix and that of B238 to B246:
1,628 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 37 source scans passing.

Open to the owner's veto:
marking `StreamCutShortError`.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B236: the window trial's two refusals were `RangeError`s the pick could only name by class

Red in `afee9dcda`,
fixed in `4c26a48a0`.

Found by the census of B232.
`runPick` (`window-trial-pick.ts`) logged `String(error,)` of anything the arms raised,
and `runSliceArms` raised two `RangeError`s whose sentences were its only account of why a slice was refused.
Naming the class alone would have lost which refusal it was.
The red case refuses a lone slice and expects the sentence in the warning.

The fix:
`TrialSliceRefusalError` (`window-trial-refusal.ts`) is marked and built from an entry id and a slice index,
the pick logs it through `exchangeFailureText`,
and the sentences are unchanged.
One test file the agent's patch no longer applied to,
`window-trial-slice.unit.test.ts`,
was merged by hand by the lead.

Open to the owner's veto:
the class change of the two refusals,
which were `RangeError`.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B267: a request the runtime refused to send reached the lost-voice line as its class alone

Red in `54b3d2535`,
fixed in `a29a9dc42`.

Found by the follow-up B266 records.
After B233 a log line named a `fetch` rejection by class,
so the lost-voice line said `refused by TypeError` for an unreachable network and for an unsendable header alike.

The fix:
the transport catches the rejection where it is raised,
with no abort standing,
and throws the marked `TransportRequestFailedError`,
whose account names the possibilities and picks none,
with the runtime's rejection kept as its `cause`.
An abort passes on unchanged.
The retry ladder retries the new class as it retried the runtime's own rejection,
read by the lead in `transient-retry.ts`:
anything thrown that is not a self-ended stream,
a bound cut or an unretried status is retried.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B268: the provider's own account of a refusal reached no log line

Red in `54b3d2535`,
fixed in `a29a9dc42`.

Found by the follow-up B266 records,
and caused by B233's fix.
`exchangeFailureText` dropped the body excerpt,
and the two comments saying a withheld excerpt stays on `bodyExcerpt` "for the log" were untrue:
nothing logged it.
The only evidence of why a provider refused one request and not another,
a moderation refusal,
a context limit or a data-policy mismatch,
was in no log line.
The red cases throw a `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
with a body through each reachable log site and expect the words:
the lost-voice lines of the two stage calls,
the retry line,
the re-ask warning,
the abandoned buy,
the unreadable budget view and a thrown repair,
where the stored record keeps class and status alone.

The fix:
`exchangeFailureLogText` is `exchangeFailureText` followed,
for any provider status failure whose reply excerpt shows a reader something,
by `(the provider said: "<words>")`,
quoted onto one line.
Every site that logs a failed exchange uses it;
every stored record,
finding and thrown message keeps `exchangeFailureText`.
The two comments say where the log reads the field.
A new scan,
`body-excerpt-readers.unit.test.ts`,
holds the field to five named files;
the lead wired it into the `source-scans` task,
which ran 39 files from then.
The lead's edit:
the window trial's refusal line uses the log form,
in a file the agent was told to leave.

Not cased:
no case reaches the log line of `image-reading-pair`,
`required-providers`,
`model-health`,
`coverage-probe`,
the window trial or the three abort-path warnings with a provider status failure.

Open to the owner's veto:
the provider's words printed whole by `model-health`,
where a cut at 300 units stood;
`jsonLine` exported from `resolution-sheet-evidence.ts` for the quoting.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B371: exit 128 read as a clone git could not open

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its fifth finding.
By the agent's probe with a corrupt commit object,
`show`,
the commit probe and `ls-tree` all exit 128 with `inflate: data stream error`,
while `rev-parse --git-dir` exits 0 there,
and 128 for a plain or a missing directory.
The reader took the probe's 128 as a clone git could not open,
and the refusal told the operator to check that the directory is a repository.
By the agent's run on the port build,
the case naming a commit git cannot inflate in a clone it opens fails,
reading `unreadable-clone` where it expects `other`.

The fix:
on a commit probe's exit of 128,
`probeGitDir` asks for the git directory:
128 means `unopened`;
0 means `unasked` keeping the 128 failure,
so the kind is `other`;
and any other failure means `unasked` with both failures.
The `unasked` arm carries its failures,
and the TSDoc of `unreadable-clone`,
`unopened`,
`unasked`,
`GIT_FATAL` and the probe says what 128 means.
The `AggregateError` now reads "the corpus read failed,
and the probe of its commit got no answer about it",
since its old wording,
that the probe got no exit status it reads,
became false;
two stand-in git cases changed on that sentence alone.

Open to the owner's veto:
The remedy wordings are unchanged,
and a clone git opens but cannot read is now `other`,
whose remedy is to run the same git read by hand;
the alternative is the old kind,
`unreadable-clone`.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B373: a cut failure text read as whole

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its seventh finding.
`cappedFailureText` cut a corpus read's opening at its cap,
then appended the kind and the remedy with no mark,
so a cut path read as the whole path.
By the agent's run on the port build,
the two cases with a long id,
in `tally-error-text` and `sentinel-probe-line`,
fail,
since each now expects the cut to end in the mark.

The fix:
an opening longer than the cap is cut one unit short of it and ends in `CUT_MARK`,
the ellipsis character,
before the kind and the remedy,
and the TSDoc of `TALLY_ERROR_CAP` says so.
By the agent's measurement,
the closing texts run to 92,
105,
140 and 93 units,
so a corpus read line's error text is at most 200 plus 140 units.

Open to the owner's veto:
The error text of the PROBE,
TALLY and CLEANUP lines can run up to 140 units past the cut,
and a cut opening ends in the mark within the cap;
the report names no alternative.

Recurrence:
`mistake-prevention.md`,
"Checks that vouch for a whole block".

## B374: a commit probe that threw in the process dropped the read's failure

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its eighth finding.
A working directory removed under the process made the probe throw `ENOENT` before git ran,
and the read's own failure was lost:
by the agent's run on the port build,
the case removing the working directory saw an `Error` with the code `ENOENT` alone.

The fix:
`probeKeepingRead` wraps any probe failure that is not a child's
in an `AggregateError` holding the read's failure and the probe's,
with the probe's as its cause,
still no `CorpusReadError`;
the case,
renamed for stopping with both failures,
asserts the name,
the message and both members,
code 128 and then `ENOENT`.

Open to the owner's veto:
A probe failure in the process surfaces as an `AggregateError` holding both failures,
still a fault at exit 5;
the report names no alternative.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B387: `UnansweredContestSliceError` printed as a bare class name in the tally and in verify-published

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers.
`UnansweredContestSliceError` (`corpus-run/would-ship-text.ts`) took a finished message and was unmarked,
so `refusalText` named its class alone:
a corpus pass's TALLY line read `refused by UnansweredContestSliceError`,
and verify-published's refusal line,
which `reportingRefusals` (`corpus-run/cli-refusal.ts`) writes through `refusalText`,
named the class the same way before printing the frames,
the slice lost in both,
where its marked siblings,
`CollapsedHeadingError` among them,
print their sentence.
By the agent's run against the base build,
the new `tally-error-text` case fails;
and the `verify-published-run` ordering case,
now building the error from its slice,
reads an empty message,
which is the red the new constructor gives against the base class,
since that class took a finished message.

The fix:
the class declares `messageNamesOnly: true`,
carries `sliceIndex`,
and writes "slice" and the index,
then "differs across lanes and the contest names it nowhere";
it is listed in `MARKED_CLASSES`.
Like `CollapsedHeadingError`,
it names no remedy for the operator.

Open to the owner's veto:
verify-published still prints this error as a fault at exit 5;
the alternative is a stated refusal at exit 6.

Closed on 2026-10-10 (UTC) by `752be22cf8f4d875333bf1d54efdb90955d619a1`,
which keeps the fault and records why in its message.
Every artifact verify-published reads goes through `parseSettledTwoLaneArtifact`,
which hands `parseLaneSelection` the comparison it recomputes,
and `assertContestCoversEligible` (`corpus-run/artifact-two-lane-read-contest.ts`)
refuses a contest that omits a slice whose lanes differ as an `ArtifactParseError`,
which `readEntry` (`corpus-run/verify-published-entry.ts`) prints as a `REFUSED by` line.
So the error is reachable only through a defect in the reader,
and its sentence offers the operator nothing to change;
no honest input reaches it,
and no case is red.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B422: the census of thrown text found every failed call already written

No code change.

The dark-seat work counted every path where a stage call can throw,
to find any whose error text reached no line,
since TianqiChen66622's log had none for the seats that threw.
Every caller already writes the thrown text on a warn line;
that log lost them to the launch's stream wiring
(M147).
The run's "seats dark" line now says where those lines are:
"<stage> <model>: <error>,
voice lost",
or "seat unreachable".

## B446: ConsistencyInvariantError recorded only its class name

Red `54d53e8b46`,
fixed in `d4736ccb6c`.

A contained failure wrote `refused by ConsistencyInvariantError` into a group's verdict and the section's findings,
since the class was not marked as carrying only names.
It is now marked,
every throw site interpolates positions,
labels and vocabulary only,
its sentences count places from one,
and the message inventory names the part `invariant`.

## B689: a TypeScript bridge that could not start made the spread rules report degraded findings with exit code zero

Found in the request memo round (`w190`),
where the first change for it,
`50ccee0440` and `5dce27bb59`,
was held (M275).
The redo (`w194`) found the root cause.

Outcome written on 2026-10-19 (UTC).
The spread rules of `package/oxlint-plugin/no-restricted-syntax` ask the TypeScript bridge for a type,
and read any bridge failure as an expression of unknown type.
The plugin started the native child of its own copy of the bridge on the first file that needed a type.
By then Oxlint had mapped 32 regions of 6 GiB for its 28 workers
(the notes say about 192 GiB of address space),
and a process that reserved more than the host holds in memory and swap (62 GiB and 79 GiB here) cannot fork.
The spawn failed with `ENOMEM`,
and the rules saw no type.
An `Array.from` of an array was dropped with exit code zero,
and a slice,
a `concat` or a spread was reported as ambiguous;
only a line on standard error mentioned the spawn failure,
and no diagnostic did (the bodies of `50ccee0440` and `249c3d2d0b`).
The held change had named loading the `prefer-readonly-parameter-type` plugin as a way out,
and that cannot help a plugin that bundles its own bridge.

Fixed in `249c3d2d0b`,
landed with the redo after gate g108.
The plugin starts the child when it loads,
before Oxlint starts its workers,
and keeps a failure to start instead of throwing,
because only the spread rules need the bridge.
The first spread rule that needs a type throws the stored failure,
naming the file,
the spawn failure and the ways out (the worker count and the host's memory),
and does not spawn again.
A file in no configured project still gets the type-free ambiguous report.
Seven cases of the package's test task failed on the base.
With the throw and without the load-time start 13 failed,
and none fail after the change.
Three of the 13 had passed on the base only because the degraded report happened to match what they expected:
"reports ambiguous spreads without a fix when no type is available",
"reports no-useless-spread when violated" and
"reports prefer-spread when violated".

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"What a failing check prints".

## B690: oxlint output that was not JSON reached the plugin tests as a bare `SyntaxError`

Found by the agent of the fix-mode fixture round (`w195`),
who left three follow-ups:
a crash in the `stylistic` tests surfaced as a JSON `SyntaxError` and not as the check for a diagnostic with no rule
code (B688),
the README of the test support documented none of the options of `runOxlintFixture`,
and `nano-spawn` stayed in the `package.json` of `tsdoc` after nothing imported it.

Outcome written on 2026-10-19 (UTC).
A Sonnet agent (`w196`) made the three changes:
red `9ced79f658` (landed as `f68bf69f64`),
fixed in `0c1114a3fa` (landed as `f09002e5b2`),
README in `bd6965cc65` (landed as `51a577a131`),
and the dependency removed in `29b3da16dc` (landed as `5e2fae86ca`).
`runOxlintFixture` let the `SyntaxError` of the parse escape when oxlint printed a failure text,
for example when a plugin throws in `createOnce`.
It now throws `OxlintOutputParseError`,
whose message names the target,
the config,
the command,
the stdout and the stderr,
with the parse failure as its cause
(the change adds `string-dedent` for the multi-line message).
A probe that throws in `createOnce` needs a plugin file of its own,
because oxlint calls `createOnce` for every rule when it loads a plugin.
The README (`package/oxlint-plugin/test-support/README.md`) documents the runner and its options.
In `package/oxlint-plugin/tsdoc`,
`rg` found no mention of `nano-spawn` except its `package.json` entry (the body of `5e2fae86ca`),
and the lockfile loses the matching importer entry.
Gate g110 ran the new parse-error case and passed with the known lint findings of `tsdoc` and `stylistic` (B688);
the cherry-picks `f68bf69f64` to `5e2fae86ca` were pushed on 2026-10-19 (UTC).
The agent did not render the README (M273).

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"What a failing check prints".

## B692: every provider's HTTP refusal carried Synthetic's class name and no provider

Status:
fixed on 2026-10-19 (UTC),
red `180a72421d`,
fix `a81da6863e`,
gate g117.

`SyntheticHttpError` (`provider-stream/src/completion-shape.ts`),
documented as a non-success status "from the Synthetic API",
was the error every client threw for a refused call:
Synthetic,
Hyper,
OpenRouter,
Bedrock,
and the OpenRouter decision transport.
It carried no provider.
`zheermao14` logged three Bedrock seats as
`SEAT CREDENTIAL REFUSED google.gemma-4-31b: refused by SyntheticHttpError with HTTP 401 (the provider said: "...The security token included in the request has expired...")`,
which reads as a Synthetic refusal,
and the line's remedy was only
"Renew the credential for this seat's provider before the next run",
naming neither the provider nor the other valid course,
which the owner had already chosen for Bedrock:
not renewing,
and letting the run go on without the seat.
The request-size refusal said "Synthetic API refused a request body" for every provider in the same way.

The class is now `ProviderHttpError` with a required `provider` (`ProviderName`),
passed by every construct site from the client that served the call;
the shared helpers that build it
(the retry ladder,
`failureForReply`,
`requireNoStreamError`,
`requireWholeAnthropicMessage`,
`extractAnthropicCompletion`)
take the provider as a parameter with no default.
`exchangeFailureText` renders `refused by ProviderHttpError from <provider> with HTTP <status>`,
and the seat line reads
`SEAT CREDENTIAL REFUSED <model> on <provider>: ...`,
ending with both courses:
renew that provider's credential before the next run,
or leave it,
since the run withholds the seat after its failed asks and goes on with the other seats.
Nothing parses the rendered text back,
and no cache key folds it in;
stored records written before the change keep the old wording.
Historical mentions of the old name keep their text with an in-place rename note.

Left open:
`SyntheticRequestTooLargeError` keeps its name and its byte threshold,
which were measured against Synthetic only,
though `failureForReply` raises it for every provider;
whether Hyper,
OpenRouter or Bedrock refuse at that size is unmeasured.
`SyntheticClient` (renamed `ModelClient` on 2026-10-19) and `createSyntheticClient`
keep the Synthetic name for the routed client.

## B693: the request-size refusal claimed Synthetic's cap for every provider's reply

Status:
fixed on 2026-10-19 (UTC),
red `e5bf21649b`,
fix `af2a0a27e5`,
landed in `8b1eb54574`.
It closes the first item B692 left open.

`failureForReply` re-raises a reply as `SyntheticRequestTooLargeError`
when it carries HTTP 400,
a body beginning `Could not parse request as valid JSON`,
and a request body over 10485760 bytes.
The Charm Hyper,
OpenRouter and Bedrock clients route their failures through the same helper,
so a reply from any of them with those three signals named a cap measured on Synthetic's gateway alone.
Whether those gateways cap a body at any size,
and in what words they refuse it,
is unmeasured.
No run log under `~/temp/agent` holds the size refusal from any provider
(`rg` for `API refused a request body` over every `*.log` there on 2026-10-19,
three unrelated container directories unreadable),
so no run has met it.

The reply now has to come from Synthetic as a fourth signal;
any other provider's reply stays a plain `ProviderHttpError` saying what that provider said.
The class lost its `provider` parameter,
since it can describe only Synthetic,
and its message names Synthetic.
`doc/troubleshooting/synthetic-request-body-size-cap.md` records the fourth signal and why.

Prevention:
a class or constant named for one provider,
and measured on that provider,
is gated on that provider where the decision is made,
not only in its name.
When a helper becomes shared across providers,
every provider-named class it builds is read for that gate.

## B694: the client type every driver takes was named for Synthetic

Status:
fixed on 2026-10-19 (UTC),
`9081481c0d`,
a rename with no change in behaviour.
It closes the second item B692 left open.

The type `SyntheticClient` in `provider/src/chat-contract.ts`
was the client surface every driver consumes,
and the routed client that spans every provider was typed with it
(`run/src/corpus-run/run-config.ts`),
so a reader of a driver's signature took the routed client for Synthetic's.
It is now `ModelClient`,
in 275 TypeScript files of 21 family packages,
with its TSDoc saying it fits a provider's own client and the routed one alike.
`createSyntheticClient` and `synthetic-client.ts` keep their names:
they build Synthetic's own client.
Historical mentions of the old name keep their text with an in-place rename note.
Build,
`lint:types`,
`lint` and `test:unit` pass in all 21 packages,
and the source scans pass.

Prevention:
the rule "Provider names in failures" in `mistake-prevention-messages-labels-and-counts.md` covers names
as well as failures:
a name that says one provider is kept for what only that provider serves.
