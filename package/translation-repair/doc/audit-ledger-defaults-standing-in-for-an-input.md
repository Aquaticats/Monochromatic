# Audit ledger: defaults that stand in for an input

Places where a missing,
malformed or unreadable input was replaced by a default,
and the run went on as though the input had said so.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B29: silent fallbacks (B21, family seven)

Status:
fixed,
2026-09-29,
with one gap left open and named under its own heading.
The full suite passed on `e396ceb67`,
the entry's last commit touching source,
with no failing case among 1355 passing groups.

A silent fallback is a value the code supplies where the caller held a real one:
a parameter default read as "none",
a caught error turned into nothing,
a nullish fallback substituting another text.
The census read all three,
and the permissive gate defaults and string accumulators found beside them.

### Census

Parameter defaults:
`~/temp/agent/f7-fallbacks/default-census.ts`
(esbuild and acorn over the package's non-test source)
found 144 functions whose destructured first parameter defaults a key,
and listed every call leaving one out.
Most are test seams
(a transport,
a clock,
a poll interval)
that production never passes.
The floor-inputs prototype
(`floor-scan-probe.ts`,
over the package's own oxc parse,
which keeps the types)
then read defaulted and optional keys alike:
89 functions let a caller leave out an input that decides what a floor refuses or what a sheet shows,
over 162 calls.

Caught errors:
`catch-census.ts` read 142 catch clauses;
15 neither log nor rethrow,
and 14 of those return or record the caught error for their caller.
One dropped it
(`openrouter-chunk-scan.ts`).

Nullish fallbacks:
623 `??` uses,
101 substituting something other than a literal.
Read one by one,
they are sentinels,
map lookups,
the documented seating fallback
(no fresh reading leaves the roster the driver was given standing,
`repair-checker-reseat.ts`,
`standingSeating`),
and text fallbacks each commented at its line
(the archive's front-matter shape,
else the source's,
for a new insertion;
the first rendering an answer was seen under).
None is silent.

Gate defaults:
12 gate parameters default to the permissive answer
(`eligible`,
`incumbentEligible` twice,
`standingValid`,
`standingMayShip` four times,
`standingEligible` twice,
`choiceMayShip` twice).
Every production call states them;
only the judging-window trial,
a measurement file,
leaves `incumbentEligible` out.

### Findings fixed

- Reference attestation decided on a lone voice:
  the extraction's votes-needed was `rosterQuorumSize` of the voices heard,
  which is one when one is heard,
  and a confirmation hearing one voice confirmed alone,
  against `MIN_STAGE_VOICES`
  (no stage is decided by a single model).
  Red `7d28373e7`,
  fix `e903f4a15`:
  `attestationVotesNeeded` floors the count at `MIN_STAGE_VOICES`,
  and a confirmation hearing fewer keeps the extraction's details with a warning and a finding.
  Over the stored rounds
  (944 extractions,
  205 confirmations)
  none heard exactly one voice,
  and the attestation keeps no cache
  (`corpus-run/pass-attest-references.ts`).
- The final polish dropped the slice's disputed wordings:
  `applyFinalPolish` held them on its subject and never passed them,
  so a polish landing on one,
  in all but whitespace,
  shipped past the standing verdict,
  the lane offer and the producers' floor that had each refused it.
  Red `907572336`,
  fix `6df19b319`,
  riding inside consolidation version 20.
- The chunk's checker check left the refiners out,
  though its TSDoc said it refused a checker who refines
  and the other three callers pass them;
  and the bench read again at the checker stage was read for quorum alone,
  so a re-seat naming this chunk's editor would have had it grade its own rewrite.
  Red `645b60fbc`,
  fix `6c9e948e2`
  (`assertBenchAgainstWriters`).
  Neither reached a run:
  `corpus-run/run-seats.ts` asserts every derived bench against the static editors and refiners.
- Two writers' sheets displayed candidates without their break counts,
  which `renderedBreakPrompt` defaults to none
  (on 2026-10-22 that function became two:
  `renderedBreakContract`,
  the fixed instruction in the system prompt,
  and `renderedBreakFacts`,
  the counts of the call in the user turn,
  which defaults to no renderings):
  the consolidation writer,
  shown both lanes' texts,
  and the translate follow-up writer,
  shown the candidates the judges declined;
  the gates,
  the lane contest and the slate judges pass theirs.
  Red `b34848d40`,
  fix `0335bd489`,
  riding inside consolidation version 20 and translate version 15.
- The translate producer left `pageText` to its default;
  the value was the same,
  and `e97c46e88` states it,
  matching the floor after the repair turn.
- Three names were exported by two files each
  (`settleTranslateSlice`,
  `withoutComments` under one signature for two comment grammars,
  `wordsOf`),
  which conflated the census's first read and names two functions alike in every `fn.name` logger tag.
  Red `f51407716`
  (`exported-function-names.unit.test.ts`),
  fix `c9e78d56b`.
- `whitespaceTokensOf` and `handleReading` grew strings a character at a time,
  the accumulator rebuild RG2 in `AGENTS.md` rules out for text.
  Pinned `c360b4c01`,
  rewritten `13adce419` to slice by index.
- `openRouterChunksOf` swallowed a parse error with `void error`,
  the only such line;
  `07d4a4504` reads the failure as data through `parseModelJson`.

### The guard

`628404247` adds `floor-inputs-stated.unit.test.ts`,
and `39e61d6d0` extends its key set to the gate flags
(the corrective comment on that commit fixes its count of them to twelve).
It fails on a production call leaving a floor input or gate flag out,
unless the call is named with its reason
or sits in a named measurement file.
Nine calls are named:
the dispute note,
which reads no stand-in eligibility;
three lane-contest eligibility calls,
since front matter carries no dispute
(`0bf3305ba` pins that the repair lane raises no issue there)
and the contest verdict decides only persistence and the log,
while the consolidation re-reads every contested standing against the disputed wordings;
the seeded-error benchmark twice;
the library entry,
which takes no cited references;
the artifact rebuild,
which re-carves recorded slices;
and the translate slice's stage input,
whose literal states all seven inputs it forwards.
Fourteen measurement files are named as files.

The mutation batch
(`b29-mutants.json`,
15 mutants over every fix and both guards)
caught 13.
The two survivors were gaps in cases written this day:
a negative assertion that searched for a label the empty lane never carries,
and an astral case that exercised the other branch's offset.
`c0738cda9` closes both,
and a rerun caught each.

`fba7ec063` adds two more scans,
so the catch and accumulator rules are not left to review:
`caught-errors-kept.unit.test.ts` fails on a catch clause that binds nothing,
discards its error with `void`,
or neither logs,
rethrows nor names it;
`text-accumulators.unit.test.ts` fails on a `let` begun as text and grown inside a loop,
with `roman` in `corpus-run/ordinal-style.ts` named,
since it reads a fixed table.
Three mutants
(`b29-guard-mutants.json`)
were each caught.

The one unreadable call the scan names rests on a literal it cannot read,
`translateSliceInput`'s returned stage input.
A batch dropping each of that literal's seven inputs
(`b29-stage-input-mutants.json`)
caught four
(the identity,
the dispute note,
the disputed wordings,
the pictures)
and left the cited references,
the attested lines and the declared names unpinned.
`709c0d3d4` adds a case reading all three off the stage input,
a rerun caught each,
and `1aa472e32` names the pinning tests in the exemption's reason.

The cache-account audit
(`mise run cache-account-audit`)
reports every source commit since the seven versions were set riding inside all seven,
this entry's among them,
since no slice-cache file has been written since:
386 source commits since the oldest of the seven was set
(refine version 5,
`30e66051e`),
60 of them named by an account.

### The rendering audit's cited references

The settled rendering audit showed its auditors the declared names the producers had
and not the cited references
(`corpus-run/rendering-audit-settled-input.ts`),
which by that module's own reasoning lets an auditor mark a reference-backed rendering wrong.
Artifacts record no references,
so the audit reads them the way the pass does.
Red `45ef57b2e`,
fix `4b02cff72`:

- The audit sheet carries the references in a fenced block
  under `CITED_REFERENCE_AUDIT_RULE`,
  its own wording rather than the critics' copy,
  which names a TRANSLATION the audit never shows
  and an accuracy/addition label its wire rejects.
  The rule names the CANDIDATE and the unsupported-addition category,
  says a CANDIDATE leaving out what only a reference states has omitted nothing,
  and keeps reader protection above the references.
  A page linking nowhere gets no block,
  so its sheet reads as audits did before.
- Every subject carries its whole page original,
  and `withCitedReferences` reads each bought page once
  through `RUN_OUTSIDE_READS.references`,
  the reader the pass uses,
  and reads nothing when nothing is bought.
  The reader is required,
  so no unit test can fall back on the environment's key (ledger X19).
- `auditOne` takes the references as a required input,
  records `referencesKind`,
  and keys `textIdentity` on a references digest
  that is absent when none were shown,
  so a row shown none keys exactly as the rows written before this,
  all of which were shown none.
- The references are a tagged value,
  `SettledReferences`:
  `cited` with the lines,
  `none` where the page links nowhere,
  or `unread` with the count of links
  where the page links pages and the read returned nothing,
  which `citedReferenceBlock` does for every linked page when no key is set.
  The first version recorded both empties as `none`,
  the silent fallback this entry is about,
  which a review of the fix caught.
  Red `37c4adbe6`,
  fix `e396ceb67`:
  `withCitedReferences` tells the two apart by the page's own links
  (`citedReferenceUrlsOf`)
  and warns on an unread page,
  and an unread row keys as one shown none,
  since it was shown the same texts.
  A batch of 5 mutants
  (`b29-refs-unread-mutants.json`)
  caught all 5.
- The attested lines are not shown,
  a quality call recorded here and open to veto:
  they name archive details,
  and the CANDIDATE under audit is often a fresh rendering.

Exposure,
measured by `b29-audit-refs-exposure.ts`
through the pass's own `citedReferenceUrlsOf`:
34 of the 92 originals at the run pin link at least one page the reader would read,
67 links in all;
in the settled archive,
`zheermao101` links one page in both run sets,
and `Aniloviraw` links none.

The mutation batch
(`b29-audit-refs-mutants.json`,
13 mutants over the sheet block,
its fence and rule,
the row's references,
kind and digest,
the per-page read and pairing,
the subject and the input module)
caught all 13.

Beside it,
the audit sheet showed DECLARED NAMES over an empty fence
when handed an empty identity string,
unlike the references block;
no caller passes one.
Red `a174c5d66`,
fix `2beaca02f`.

### Open

- A Han character beyond the first plane that pinyin-pro cannot read passes through the handle reading as written.
  No pinned original or stored artifact carries one
  (ledger B21's census).

### Fallbacks T8's coverage found unreachable

T8's batches read each cold nullish fallback for reachability before writing a case,
and remove the ones no input reaches,
since one would substitute a value silently the day an invariant broke.
Each is replaced by code that has no such branch,
or by `nonNullishOrThrow` where the type needs narrowing,
so a broken invariant throws.

- `placeTally` read an unloaded bundle's sources with `?? []`;
  `requireMapFor` refuses every unloaded bundle with no map first
  (`5e27208c7`).
- `parseNaturalnessConfirmations` fell back to `-1` for a confirmation's round position
  and read a missing decisive round as a different roster;
  each confirmation is now matched to its round in one step that refuses a confirmation matching none
  (`a564eccc7`).

## B36: a page no grammar reads passed a candidate against the original alone

Status:
fixed 2026-09-30 (UTC) with the sixth T8 batch,
found reading `translate-skeleton-page.ts` 114 to 124,
the catch around the plain-markdown reading,
which its own comment called close to impossible to reach.
Plain markdown accepts any text,
but reading a page can exhaust the parser:
8,000 nested quotation markers overflow its stack
(`relaxed-shape-probe.mjs` in the audit's scratch folder).
Such a page read as unparseable,
the validation took an empty page,
its block floor compared the candidate with the original alone,
and the verdict passed the candidate as read by the relaxed grammar:
at 10,000,
20,000 and 50,000 markers the verdict on an ordinary rendering was valid,
where at 1,000 and 5,000 the page was read and the same rendering refused.
The validation's own comment says a check that cannot run must not answer yes,
and names the case that taught it.
No page a run has read reaches it:
the pinned corpus's 279 Markdown files nest quotation markers at most 3 deep
(`quote-depth-probe.mjs`),
so no stored verdict would come out differently.

Fix:
a page neither grammar reads is treated as an original no grammar reads:
the floors that read text still run,
and with none refusing,
the verdict is unknown with the page's reason,
so the candidate stands unvalidated and says so.
`grammarFreeVerdict` holds that path for both sides,
and the empty skeleton the old path used is gone.
`translate-validate.unit.test.ts` pins both verdicts over a page of 16,000 markers,
and the case that said the floor fell back to the original alone,
whose page plain markdown reads,
now says it reads the page under plain markdown and checks the grammar it names.

## B49: standing flags that defaulted to letting a standing ship

Found by the eighth coverage batch (census-dUOMFS),
at `input.standingMayShip ?? true` in `consolidate-slice-buy.ts`.
Four consolidation functions defaulted `standingMayShip`,
and two of them `standingEligible`,
to `true`:
the slice buy and its attempt,
the settlement,
and the persistence predicate and writer.
The driver passes both flags on every production call,
so no production caller used a default;
a caller that forgot one would have let a standing no contest endorsed,
or no gate found eligible,
ship as though it were both.
That is the silent permissive default of B21's seventh family.

Fixed in `30127d1c3`:
the flags are required.
The test calls that relied on a default were listed by a type check against a rebuilt `dist`
(a check before the rebuild read the old declarations and passed),
and each now passes the value the default gave.

Recurrence:
a flag whose absent value grants something takes no default;
and a type check of tests that import the build runs after the build.

## B106: the attribution reader read a missing or malformed field as an empty one

Found 2026-10-01 (UTC) reading the twenty-fourth T8 batch,
the attribution reader's stretches in `census-xSHq2Q`,
red in `48c4433d9`,
fixed in the commit adding this entry.
`gatherAttributionEntries` (`corpus-run/attribution-read.ts`) told absent from malformed for the critic record alone,
which is right,
since an artifact settled before attribution carries none.
Every other field it read fell back to an empty reading:

- `issues` absent or not an array read as no issues;
- an issue record that is not a record,
  or whose `issue` is not one,
  read as a view with an empty status and no claims,
  the reading B92's case pinned;
- a `status` that is not a string read as empty,
  `claims` absent or not an array as no claims,
  and a claim member or `claimId` of the wrong shape was passed over;
- a `lanes` key on an unversioned or version 1 artifact,
  which keeps its records at the root,
  was ignored;
- a version 2 or later artifact whose `lanes`,
  `lanes.repair` or `lanes.repair.result` is not a record read its records off the root,
  which holds none of them;
- a claim credited to a proposer its own chunk's `heardCriticIds` does not name was counted,
  and the report's roster took critics from the claim maps as well as the heard counts,
  so such a critic got a row with no chunk heard.

Every one of these fields is written by every generation that has settled an artifact.
Over the 379 stored artifacts
(the run directory's `artifacts`,
`round-one-archive` and `round-two-archive`,
and every throwaway runs directory's `artifacts` under the audit's scratch folder),
every `issues` is an array,
all 26,544 statuses are strings,
all 36,496 claim ids are strings,
all 263 artifacts of version 2 or later hold `lanes.repair.result` as a record,
no root-record artifact carries `lanes`,
and all 23,714 proposers were heard in their own chunk's record.

The reader now refuses each shape with `ArtifactParseError`,
naming the path:
`readIssueViews` and `readClaimIds` at every level,
and `recordsHolderOf` by the artifact's own generation.
`decodeChunkRecord` (`corpus-run/attribution-decode.ts`) reads `heardCriticIds` before `claimAttributions`
and refuses a proposer the chunk did not hear.
`buildAttributionReport` (`corpus-run/attribution-report.ts`) narrows the eligible entries by a type guard,
so three `?? []` defaults go,
and builds its roster from the heard counts alone,
ordered by `compareCodePoints` (ledger B95).
The entry reader is exported as `attributionEntryOf`,
and holds the artifact's `id` to its file name,
as placement does (`corpus-run/artifact-placement.ts`),
since it reads the file a second time;
its refusal names the field,
`<file>.id`,
where it named the file alone.
Its own cases reach the two refusals the gather never feeds it,
a body that is not a record and an `id` that is not the file's name.

The red commit's cases failed at the build of `020c9bebd`:
the rewritten B92 case,
and the table's 16 rows,
each of which a probe ran alone in a directory of its own
and saw read as an entry,
with nothing refused.
With the `id` check weakened to a type check,
the case naming another entry's `id` failed alone in its file,
and passed once the check was restored.

Reach:
the built gather ran over every stored directory holding JSON
(240 directories)
at the build of `48c4433d9` and at this commit's.
It read 205 entries with 42 refused before,
and 202 entries with 45 refused after;
the eligible entries stayed 202.
The three newly refused are `Cat.json` files of 180 bytes,
written 2026-09-10 in `issue-authority-cache-*` folders,
holding only `id`,
`tip`,
`pipelineDigest` and `status`:
test leftovers,
from a prefix no current source names,
that the gather had counted as ineligible entries.
The 42 refused both times are B107's.
Twelve directories the gather refuses whole,
six for an empty pool and six for mixed generations,
both times.

Calls made here are open to veto:

- a field every generation writes is refused when absent,
  rather than read as empty;
- the entry reader holds the file-name identity again
  rather than trusting the pool's earlier read;
- the roster is the heard critics alone,
  a proposer outside them being refused upstream;
- the three test leftovers count as refused rather than as ineligible entries.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B127: a sample size that was not a whole number drew a slot from every band on every pass

Red in `1d3f3dafe`.
Found on 2026-10-05 (UTC),
reading the MiMo trial's production diff once the trial was merged.
`allocateBandQuota` (`sample-draw.ts`) hands a sample's slots to the size bands one at a time,
round-robin,
and stops when `remaining` reaches exactly zero.
The trial's T8 change replaced the loop's no-progress break with a count of passes taken from the size,
and a size carrying a fraction steps over zero:
on the build of `7f069adc9`,
ten candidates a band and a size of 2.5 returned three slots a band,
nine handed out for two and a half asked
(`node --eval` against the built package).
The loop as it stood at `28303c42a` ran while `remaining` was over zero,
which ends that call at three slots;
that is read off the source,
since no build of that commit is kept.

No sample was harmed:
the one production caller,
`corpus-run/draw-sample.ts`,
passes `DEFAULT_SAMPLE_SIZE`,
the whole number 50.
The function is exported,
so the fraction is an input its signature admits.

The fix:
`allocateBandQuota` refuses a size that is not a safe whole number,
zero or more,
with a `RangeError` naming the size,
the way `assertSourceBytes` in `sample-grading.ts` refuses a byte count.
With the size whole the trial's pass count is exact,
and it stays.
The cases refuse 2.5,
-1,
`NaN` and `Infinity`,
and hand out nothing for zero.

The trial's other change to the file stands.
It dropped the issue-key tiebreak from `selectFromBand`:
a rank is a position inside one entry's bucket and `byEntry` keeps one bucket an entry,
so two candidates of one rank never share an entry key and the comparison never ties.
The `issueKey` field the comparator stopped reading is gone from `RankedCandidate`,
and the comment gives that reason
in place of "the stable order settles anything else".

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B131: a directory id with no Latin letter stood on any alias without one

Red in `cd37ea51a`,
fixed in `dc4d651a1`.
Found on 2026-10-05 (UTC) by an independent reviewer,
behind a T8 trial case whose source fixture carried no alias,
so the comparison it named never ran.
`directoryIdNameStands` (`corpus-run/directory-id-name.ts`) asks whether the source's own aliases carry the directory id
by comparing Latin letters alone (`latinLettersOf`).
An id with no Latin letter reads as the empty string,
and so does an alias in another script:
the two compared equal,
and the id stood as the page's visible name.
The red case gives an id of punctuation,
one of digits and one in Han each a source whose only alias is a different Han name:
`expected [ true, true, true ] to deeply equal [ false, false, false ]`,
1 [PASS] and 2 [FAIL] lines.

No page was harmed as far as the corpus shows:
the reviewer counted 93 entry directories at the pinned commit,
none named without an ASCII letter;
that count was not repeated.

The fix:
an alias carries the id where it is the id character for character,
or where the id has Latin letters and the alias's Latin letters equal them.
The first arm keeps an id in another script standing on its own alias,
which the letters rule alone would drop once an empty reading stopped matching.
Three sibling cases fed their guards values the code after the guard refuses anyway;
each now also feeds the value only the guard stops.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B139: a link destination the trim emptied compared equal to every other one it emptied

Red in `855553c3b`,
fixed in `987f767dd`.
Found on 2026-10-05 (UTC) by an independent review of the MiMo trial's cases,
whose case pinned the wrong result.
`trimDestination` (`corpus-run/dropped-destinations.ts`) cuts a destination at its first stopper
and sheds trailing sentence punctuation,
for an address that prose ran into.
A tree destination made of that punctuation,
or opening on a stopper,
was cut to the empty string and still recorded,
so the current directory,
its parent,
a bare query and a link with no destination all read as one destination:
a source linking the parent directory and the current one,
against a page that kept only the current one,
reported nothing dropped.
A recorded empty destination was also found by the trace in every slice,
since every text includes the empty string
(read off `destination-completeness.ts` by the agent that fixed it,
not run).
The red run printed 4 [PASS] and 8 [FAIL] lines,
five of them cases.

No page was harmed,
by the agent's count over the pinned corpus:
`git grep` at the pin finds no page line whose link destination is empty,
is sentence punctuation alone,
or opens on a stopper,
against 331 lines in 173 files for a destination opening on a web scheme;
reference definitions were not searched.

The fix:
where the cut and the shed would leave nothing,
the destination stands as written,
so `trimDestination` answers the empty string only for an empty destination;
a scanned run never reaches that line,
since it opens with its scheme.
`markdownDestinations` leaves an empty destination out:
it names nowhere a reader could follow,
and no page owes it.
The trial's case asserted `urls: ['']` for a link to the current directory,
under a name that also claimed an assertion it did not make;
its four assertions are four cases now,
each named for what it asserts.

Open to the owner's veto,
both the agent's choices:
the destination stands as written rather than cut at its first stopper,
which would still read a destination opening on a stopper as the empty string;
and an empty destination is recorded on neither side,
so a source link with no destination that a page drops is no longer reported by this check,
though the slice rule still protects the link as an atom.

Open when this entry was written,
closed in B173,
B174 and B175:
`trimDestination` still cuts an explicit destination at a stopper where something precedes it,
so two explicit links differing only past a full-width comma read as one,
where the cut is right for an autolink literal only;
`carries` (`destination-completeness.ts`) searches by substring,
so a dropped destination as short as `.` is traced to every slice holding a full stop;
and `sameAddress('/')` is the empty string (`destination-renderings.ts`),
so the site root's key was the empty destination's,
and a source linking `/` against a page with an empty link reported nothing dropped before this fix
(the agent's probes).

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B158: an unavailable deterministic reader recorded as zero characters read

Red in `49f3ebdbf`,
fixed in `699ad2f8c`.

Found on 2026-10-05 (UTC)
by the fourth review of the trial's added cases (its finding F4).
`readImagePair` (`image-reading-pair.ts`) confirms a picture textless where two readers report little or no text,
and wrote the deterministic reader's count into that verdict:
`0` for every deterministic reading that was not `read`.
Where the deterministic reader was unavailable,
the verdict read `{ kind: 'no-text', characters: 0, confirmedBy }`
and the log line said the picture was confirmed textless past the deterministic reader's 0 characters:
a count the deterministic reader never took,
which a reader of the record takes for one it measured.
The trial's case pinned the zero
under a name saying the reader had found none.

The fix:
a second `no-text` shape carries `deterministicUnavailable`,
the reader's reason,
beside the confirming readers,
and no count;
the log line names the reader unavailable with its reason.
The counted shape and its line are unchanged.

The picture store learned the shape in the same commit:
`isPairedReading` (`corpus-run/reading-cache-store.ts`) had accepted a `no-text` record only with a numeric `characters`,
so the new verdict would have been refused on every resume,
with a warning per picture per run,
and its readers asked again on every run;
it now accepts one whose reason is one the deterministic reader gives
and whose confirming readers are names,
and three cases in `reading-cache-store.unit.test.ts` hold it.
Records the unfixed build wrote are not resumed by the fixed one,
since the store's generation is the digest of the built files.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input":
a reading that failed reaches the verdict as a failure,
never as an empty value.

## B175: the site root keyed as the empty string

Red in `5df296c5f`,
fixed in `25c33df4f`.

Found on 2026-10-05 (UTC),
as an item ledger B139 left open.
`sameAddress` (`corpus-run/destination-renderings.ts`) shed a trailing slash from every address,
so `/` keyed as the empty string,
the key of a destination written with nothing in it.
`judgeDestinationRenderings` over a source linking `/` and a page carrying an empty destination dropped nothing.

The fix:
the slash is shed only where something remains,
so the site root keys as `/`.
The red run printed `expected { root: '', ... }` for the case
"KEYS the site root as itself,
so it never compares equal to the empty string".

No pinned page links the site root:
0 of 276 page files read `/` as a destination (the fixing agent's measurement).

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B272: `percentileOf` answered zero for a percentile no rank belongs to

Red in `96dad6b2c`,
fixed in `5723cd071`.

Found on 2026-10-06 (UTC) by the coverage cluster over the `corpus-run` files (T8),
in a fallback the census listed as a cold stretch of `census-spread.ts`.
`percentileOf` answered zero for a percentile under zero or not a number on a sample that holds values,
since the rank read nothing and `?? 0` filled in.
No caller passes such a percentile
(the reported ones are 50,
90 and 99,
by the agent's reading),
so no printed figure was wrong.

The fix:
such a percentile throws a `RangeError` naming it,
and the value at a clamped rank is an `unreachable:` throw.
A percentile over 100 still reads the last rank,
and an empty sample still reads zero.
The case is "REFUSES a percentile below zero or not a number";
on the lead's red tree it failed with `Expected act to throw, but it returned`.

Measured by the lead for B272 and B273:
37 named test files pass on the fix tree,
with the 39 source scans and a clean type check.
The whole suite ran on `5723cd071` in a side checkout:
1,648 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 39 source scans passing.
That run started a child with the provider keys in its environment,
through a fixture this commit added (M121,
closed by B291).

Open to the owner's veto:
the refusal,
where clamping to rank zero is the alternative.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B275: a routed call that left its loop with no attempt left said every provider refused it

Red in `f45278048`,
fixed in `71974714f`.

Found by the coverage cluster of B274.
`routedText` (`provider-router.ts`) ended a call that left its loop with no attempt left
as `every provider refused this call`,
under a comment calling that end unreachable.
It is reached when the last attempt is cut at its stream bound,
which takes a model all four providers serve.

The fix:
the stream bound holds are read there as the next decision would have read them
(`reachPastBoundHoldsOrRefusal`,
shared with `chooseProvider`),
so four providers cut in turn end with the reason a model three providers serve already ends with,
naming the hold.
Where a provider before the last refused on its budget,
the reason says the providers refused the call or ran past their bound.

The ending and one of the two cases are the lead's.
The agent's fix worded the end as every provider serving the model having run past its stream bound,
which the loop can also reach after budget refusals,
so that wording would have been false there.
The cases are in `provider-router-stream-bound.unit.test.ts`:
one cuts the call at its stream bound on all four providers,
the other has three providers refuse on their budgets and the fourth cut,
and on the lead's red tree both failed with the old reason in place of the expected one.

Open to the owner's veto:
the two reasons.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input":
a comment calling a loop's end unreachable is checked against every `continue` in the loop.

## B276: seed detection skipped an issue naming a slice the preparation lacks

Red in `f45278048`,
fixed in `71974714f`.

Found by the coverage cluster of B274.
`gradeSeedDetection` skipped an issue naming a slice the preparation lacks without a word,
which reads every seed of that slice as never reported.

The fix:
it throws.
On the lead's red tree `seed-detection-slicing.unit.test.ts` failed by assertion.

Open to the owner's veto:
the throw.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B285: `score-agreement` scored without a file the operator named

Red in `a13f71ba8`,
fixed in `1088e72d9`.

Found by the move B284 records.
`--manifest` or `--pre-grades` with a path that was not there printed the note for a file nobody named,
and scored without the file the operator typed the flag to supply.

The fix:
`readNamedOrBeside` refuses a named file that is absent,
and `readOptional` states every other read failure in words.

Open to the owner's veto:
refusing a named file that is absent.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input":
absence is an answer only for a file nobody named.

## B447: a died of an unreadable kind left no finding on the artifact

Red `94371feed6`,
fixed in `f42d69c2fe`.

Found on 2026-10-12 (UTC) by the lead's decision on B441.
A `died` that is a truth value,
a list,
a record or a number that is not finite was logged (B433) and left no finding,
so the artifact could not tell such an entry from one that records no death.
`entryInfoFindingsOf` now answers one finding for it,
naming the key and the kind and quoting no value,
built from the same clause as the warning so the two cannot drift;
every way out of `preparePassEntry` carries it,
and a case pins each.
Findings reach no key and no sheet,
so no cache key or schema version moved.

## B717: a block with no source section was reviewed against an empty source in silence

Status:
fixed on 2026-10-21 (UTC),
red `a331908d9c`,
fix `ab0ccbe7a3`,
source scans `651ae7846d`.
Found in the same review as B716.
No cache version moved.

What the code did:
`repairArchiveBlocks` read the expected source section with `sourceContexts.get(identity) ?? ''`,
and `archiveBlockSourceContexts` read the aligned pair of a block with `alignment.pairs.at(pairIndex)?.source.text ?? ''`.
An empty section is a real answer
(a section only the archive has is filed as empty),
so a miss could not be told from it,
and a miss checked every quote against nothing:
each reply that cited the original was dropped as unanchored,
and since B715 the log says only that the reply failed an anchor test.
A miss means the caller and the map disagree about the block's identity
(its location,
id or wording differ,
as when the archive text the identity is cut from differs),
or the preparation and the alignment disagree about its place.
Neither input of the review can repair that.

The fix:
both throw a custom error that names the block and what the other side holds.
`ArchiveBlockContextMissingError` carries the identity and every identity the map holds,
or says the map holds none.
`ArchiveBlockAlignmentMissingError` carries the block,
the pair it names
and how many pairs the alignment holds.
Neither message carries archive wording.
An entry filed as empty still reviews.
The red commit replaced two cases that pinned the silent default.
The second lookup was not in the brief's search path
(`page-check`,
not `archive`);
it is the same fault one step earlier,
and a case that pinned its empty answer said "rather than the archive's whole original section",
a concern about the opposite fallback.
`preparation-unclaimed.ts` builds `aligned-pair` locations from the same alignment,
so a block naming a pair the alignment lacks is a fault.

Other sites read:
the search `rg '\?\? .{0,3}(\x27\x27|""|\[\])'` over `archive/src`,
`archive-block-repair.ts` and the sibling `archive-*.ts` of `page-check`,
and the `.get(...) ??` and `.at(...) ??` forms beside it.
Left as they are,
because an absent value is a legitimate state:
`identityContext` and `referenceContext` defaults (a page that declares no names or links nowhere),
the label of a footnote definition that opens with none (`archive-footnote-order.ts`),
the counting accumulators of the casing restores,
and `read.get(from) ?? []` in the destination restore (a key never read has no entries).
`pieces.at(-1) ?? ''` and `owner.lines[index] ?? line` in the composite narrow an indexed read that cannot miss.

Open to the owner's veto:
throwing for a block naming an absent aligned pair,
where the pass once went on with reviewers who could anchor nothing.

Cache account:
none moves.
No cached stage result is read or written by these lookups,
and the audit named in B716 read the same eight versions.

Recurrence:
`mistake-prevention-inputs-defaults-and-values-read-from-text.md`,
"Defaults that stand in for an input".
