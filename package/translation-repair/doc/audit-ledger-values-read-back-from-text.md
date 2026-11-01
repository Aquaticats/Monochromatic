# Audit ledger: values read back from text

Numbers,
fields and keys the code wrote into text and then read back more leniently,
or by a rule the writer does not keep.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B35: footnote mentions encoded as key strings and parsed back

Status:
fixed 2026-09-30 (UTC) with the sixth T8 batch,
found reading its stretches:
`translate-definition-leak.ts` 46 and `translate-marker-drop.ts` 50 each fell back to `''`
for the third space-separated part of a key that always has three.
`footnoteIdentifiers` keyed each mention as the string `role convention identifier`,
and four readers wanting the parts split it back apart three ways:
the two floors by `split(' ')[2] ?? ''`,
the insertion admission by the first space,
and the assembly guard by `startsWith` and `endsWith`.
A key is a comparison value,
and none of the readers compared keys;
each wanted a field.

Fix:
`footnoteMentions` lists each mention with its role,
convention and folded identifier as fields,
and `footnoteIdentifiers` counts keys built from it
for the one reader that compares two texts' mentions.
The four readers read the fields.
`footnote-mentions.unit.test.ts` pins the list,
the counts and the overflow refusal's whole message;
the two floors' refusal cases read their whole verdicts.

## B63: a draw's manifest never names the pool's build

Found by the twelfth coverage batch,
reading `corpus-run/pool-generation.ts` beside the census it reads.
`poolGeneration` looked each kept name up in `eligible.digestByEntry`.
The draw hands it `keepEligible`'s result,
artifact file names (`whiskers.json`),
while `artifact-eligible.ts` `mapDigests` keys that lookup by the census's entry ids (`whiskers`),
so no lookup matched,
and every real draw's sample manifest recorded `unrecorded: no kept entry recorded a pipeline digest`,
the gap the module's region comment says it closes.
The unit fixture keyed its lookup by file name too,
agreeing with the defect rather than with the lookup's contract
("keyed by entry id",
`EligibleEntries.digestByEntry`).
A search of the package directory,
the main checkout's package directory and the cache found no sample manifest outside test fixtures
(`t8-batch12-plan.md`),
so by inference no graded sheet carries the wrong record.

Fixed (`568e78912` red,
`0ad52495e`):
the names are typed `ArtifactFileName` and looked up by their entry id (`entryIdOfArtifact`);
the fixture keys entry ids,
and a case drives the chain the draw runs
(`listArtifactFiles`,
`censusByGeneration`,
`selectEligible`,
`keepEligible`,
`poolGeneration`) over two placed artifacts.
Looking the name up as it came failed four of the fixture's cases.

Recurrence:
a lookup is fed the key its builder used,
and an entry's file name and its id are distinct types;
a fixture standing in for a lookup another module builds is paired with one case that drives that module.

## B67: the container deficit found a container's tag by searching for its text

Found by the fourteenth coverage batch,
reading `insertion-container-deficit.ts` beside its cold arm for a tag the search could not find.
`insideContainer` cuts one half of a container at the container's own tag,
so the deficit counts only the blocks inside it.
It took the tag from `maskLoneContainerTags`,
which records each tag's offsets but returned only its kind,
name and text,
and then searched the slice for that text:
`indexOf` in the opening half,
`lastIndexOf` in the closing half.
A whole element of the same name in the same half slice
(a closed `<details>` pair before the container's opener,
or after its closer)
holds the same text,
so the search found it instead and counted its blocks as the container's.
Where the original's and the archive's copies of that element differ in blocks,
the difference moves this container's deficit:
in the red cases,
a container the archive renders whole admitted an absent passage.
Whether a pinned page has such a half slice is not measured.

Fixed (`1af921d31` red,
`ad58185f7`):
`LoneContainerTag` carries `startOffset` and `endOffset`,
and the deficit cuts after the first lone opener of the container's name
and before the last lone closer,
at those offsets.
Cutting at the text search again failed the two red cases and their describe,
and nothing else.

Recurrence:
a reader that needs where a tag or token stands reads the offset its finder recorded;
searching the text again for what was found is a second finder,
which finds the first or last copy of the text rather than the one found.

## B71: the slice-cost reader read counts nobody wrote

Found by T8's seventeenth batch,
reading the two cold throws in `slice-cost-read.ts`.
`readLine` checked every field in one loop,
then read each again through `provenWhole` and `provenMember`,
which threw when the loop had not checked the field;
no line could reach those throws,
and the file's own note said it validated and built together.
Its count check was `Number(raw)` and `Number.isInteger`,
and `Number` reads an empty text as 0,
`0x1F` as 31,
`1e3` as 1000
and a leading sign,
so a `chunk=`,
`sourceChars=` or `ms=` field that `armSliceCost` never wrote
(it writes `String` of an integer)
became a count in a row.
A digit run past the largest integer a double holds exactly read as a neighbouring number.

Measured over every stored `SLICE-COST` line under `node_modules/.monochromatic` and the audit's scratch folder
(`b71-scan-count.mjs`,
1,395,824 lines,
37,183 distinct):
174 carry a count field that is not plain digits,
and every one is source or test text,
not a logged value:
the writer's template in built bundles and their maps,
the reader tests' own fixtures,
and ledger excerpts
(`b71-scan-values.mjs` lists each value).
The positive control,
five planted lines,
flagged the four malformed ones and not the fifth.
So no stored pass log carries a count this reader misread.

Fixed (`774710cb0` red,
`3da144aed`):
each field is read once into its value or a named refusal
(`memberField`,
`countField`),
and the row is built from those readings;
a count is read only when its text is ASCII digits
(`isAsciiDigits`)
that `Number` reads as a safe integer,
and an empty field is named `empty` in the reason.
With the digit check put back to `Number` alone,
the red case failed and nothing else;
restored.
A signed value is refused although the writer could write `ms=-3`:
it measures the slice as a difference of `Date.now()` readings,
the wall clock,
which a clock step can move backward mid-slice
(the elapsed-time family,
found in T8's seventeenth batch record and closed by B78,
after which the meter reads a clock no step moves).
A negative duration is not a measurement,
and the dropped reason names it.
The package's other number reads from text are the same family,
open there and closed by B73.

Recurrence:
a reader of text it wrote itself accepts only the spelling its writer writes,
and reads each field once.

## B73: numbers read out of text more leniently than their writers write them

Found beside B71,
which left the package's other number reads from text open as a family.
`Number` reads an empty or blank text as 0,
takes a hexadecimal,
binary,
octal or exponent spelling,
a sign and surrounding spaces,
and reads a digit run past `Number.MAX_SAFE_INTEGER` as a neighbouring number,
or one past the largest double as `Infinity`.
`Date.parse` reads a date-time without its zone as local time,
a date alone as midnight UTC,
and spellings outside the ISO format by a fallback each engine defines.
Each reader here that took text through one of them
took spellings its writer never writes.

What the red commits (`51dc87e82`,
`81f96f2a2`,
`977724373`,
`26d5849db`,
`ee2d57372`,
`a381f9da1`)
show each reader doing:

- the stage roster counted `(/2 heard`,
  `(-1/2`,
  `(0x1/2` and `(1e0/2` as units;
- the cap census paired a stream whose content count was empty,
  `-5` or `1e3` as delivering 0,
  -5 and 1000;
- call timing read `elapsed ms` as 0,
  and round timing read an empty,
  signed or exponent duration without throwing;
- the spend line took `prompt=0x5`,
  `5e0` and `+5`,
  and costs its writer's `String` never spells;
- the grade sheet called a heading with no number "headed 0";
- numbered reference lines,
  the cache account's source commit and version,
  and Han numerals read a digit run two past the exact range as its neighbour;
- the asked count and the rendering audit's arguments truncated 40.9 and 4.9;
- the fidelity and coverage probes fell back to their default for `--cap fourty`,
  took -3,
  and read `--cap`,
  `--only` and `--damage` written last as unwritten;
- the roster card listed a blank field as 0,
  `$0x1` as one dollar a token,
  OpenRouter's `-1` (a price it does not quote) as a negative price,
  and a 401-digit price as `Infinity`;
- the two grace windows,
  the hard cap,
  spend ceiling,
  Bedrock credit and request pace took a hexadecimal,
  an exponent,
  a sign,
  spaces around the digits and a point with no digit on one side,
  so `0x3E8` or `1e3` set a value the operator did not write as such;
- the overlap dial refused a value set to blanks,
  where every other numeric variable reads blanks as unset;
- the meter reader,
  call timing and the cap census read a log stamp without its zone 14,400 seconds late
  on a machine at UTC minus four hours (`stamp-reads-probe.mjs`),
  and a date alone at midnight UTC;
  a stamp cut off part way became NaN,
  which call timing kept as a call's end that `measureInFlight`'s sort cannot order,
  and the cap census as a sample's time that pairs with nothing and counts as before the caps.

Measured before fixing:
every stored log record these readers take is spelled as its writer spells it
(1,630 stream lines,
551,805 `SPEND` lines,
287,785 round lines and 1,305 stage findings;
each scan flagged the malformed lines planted in its control),
so the fix refuses no stored log record.
Over the 28 stored provider listings
(`listing-number-probe.mjs`,
with a control file of planted values),
the 7,870 prompt and completion values,
four of them planted,
hold 20 written `-1` by OpenRouter,
10 of each,
and 2 written `0.042e-6` in LLM Gateway's catalogue,
the only exponent;
the fix reads those two as before and lists the 20 as not listed,
where the card printed a negative price.

Fixed (`73984a59a`,
`a572a1cac`,
`65dd1e6a8`,
`0ebf573a3`,
`f3c4a1c59`,
`93a0d06b1`,
with the guard red at `6006f9816`).
The shared rules:

- `whole-number-text.ts` holds `isWholeNumberText`
  (ASCII digits a double holds exactly,
  leading zeros read as written),
  cited by every refusal as `WHOLE_NUMBER_RULE`;
  `isNegativeWholeNumberText`
  (a minus sign before a whole number more than zero),
  so a reader can answer a count under zero in its own words;
  `isDecimalText`
  (digits,
  or digits,
  a point and digits);
  and `isUnsignedNumberText`
  (a plain decimal with an optional exponent,
  as JSON writes a number).
- `iso-stamp-text.ts` holds `isIsoStampText`,
  which takes a stamp only when `toISOString`,
  the logger's writer,
  writes the instant `Date.parse` read back as the very text read.
- `corpus-run/command-flags.ts` holds `flagValue`,
  `wholeNumberFlag` and `idListFlag`,
  the probes' one reader of a flag and its value.

Where each applies:
a count the package wrote is read through `isWholeNumberText`,
the spend line's cost by spelling back what `String` wrote,
and a log stamp through `isIsoStampText`.
A count or flag an operator types is digits;
the hard cap,
spend ceiling,
Bedrock credit and request pace are a plain decimal,
then finite;
the grace windows are digits;
and every numeric variable reads blanks as unset.
A listing string is read through `isUnsignedNumberText`,
then finite and not under zero,
a listing number finite and not under zero,
and anything else is `NOT_LISTED`,
so the card asks for the field rather than printing a number nobody listed.
Call timing gains an `unstamped` kind,
counted as `callsWithoutStamp` and printed by `run-timing-report`;
`readCapLog`
(once `capSamplesOf`)
returns its samples with `unstampedLines`,
which `cap-census` prints;
and the meter reader skips the line as it skipped an unreadable stamp before.
`admitWithinShortfall` named each passage by its position written as text,
and two of its three callers read the admitted names back with `.map(Number)`;
a passage now carries its position as a number.

Calls made here are open to veto:

- a count is digits only,
  and leading zeros read as written;
- the asked count and the audit's arguments refuse 40.9,
  reversing cases agent commit `4133bf926` pinned to truncate,
  which no doc records as an owner ruling;
- `-0` draws the rule's own refusal,
  and a minus sign before digits is answered as a count under zero;
- the overlap dial and the request pace read blanks as unset,
  as the other dials did;
- a listing price takes the JSON number spelling,
  exponent included,
  since a stored catalogue writes one,
  and OpenRouter's `-1` reads as not listed;
- the operator's fractional dials take a plain decimal,
  exponent refused,
  and the grace windows digits;
- a log stamp is read only in `toISOString`'s spelling,
  so one naming the right instant without milliseconds,
  with `+00:00`,
  or with a six-digit year before 10000 is refused too:
  a line carrying one did not come from the logger as written;
- a log line whose stamp the logger did not write is left out and counted,
  not thrown on,
  so one such line costs its call,
  not the report.

The guard is `number-reads.unit.test.ts`,
which `source-scans` runs.
It finds every call of `Number`,
`parseInt`,
`parseFloat`,
`BigInt`,
`Number.parseInt`,
`Number.parseFloat` and `Date.parse`,
with or without `new`;
`new Date` given an argument;
a unary plus on anything but a literal;
and any of those readers handed on as a value.
Each read is keyed by file,
enclosing named function and reader,
and the package is held to 43 keys and 48 reads,
each naming the rule that holds it to its writer's spelling,
or why it reads no text.
Two kinds read outside the shared rules are named there with why:
hexadecimal prefixes of the package's own digests
(`translate-slate.ts`,
`corpus-run/window-trial-order.ts`)
and the digits opening an engine message (`run-json-read.ts`).
Out of its reach:
a reader reached through `globalThis` or a renamed binding,
and arithmetic on text.
Its first count
(`number-reads-probe.mjs`,
35 keys)
missed the readers handed on as values and `Date.parse`;
widened before it became the guard,
the scan found the six more that `f3c4a1c59` and `93a0d06b1` fixed (ledger M90).

Guard-offs ran on every fix
(each commit message records its own).
On the last two:
`isIsoStampText` without its write-back check failed its refusal case,
the stamp case of each of the three readers and call timing's count case;
each reader's own check removed with the rule intact failed that reader's case in each file and nothing in the rule's;
the left-out counters disabled,
the stream line's and call timing's in one build and the `SPEND` line's in another,
failed the cases counting them.
The scan without its detection of readers handed on as values,
of unary plus,
or of a type name as no read failed its fixture case;
without its detection of `new Date`,
both its cases;
and with one read unlisted,
its package case.
Each was restored,
and `git diff` printed nothing after.

Found on the way and open:
a flag written twice is read at its first value (closed by B75);
plain-object tables are looked up and filled by text from outside the package,
which reaches a prototype's keys
(the fidelity probe's defect table closed by B75,
the rest by B77);
the request pace's real-clock case flaked,
18.89 and 18.99 milliseconds against a 19 millisecond bound,
recorded with the elapsed-time family (B78);
`readCapLog` copied its queue of a label's streams at every stream line,
a copy `fold-copies.unit.test.ts` did not read (closed by B74);
and `run-timing-report` prints its count of completion lines without a duration before a fixed plural,
which the open family of counts before a fixed plural covers.

Recurrence:
a reader of a number in text takes only the spelling its writer writes,
through a shared rule
(`whole-number-text.ts`,
`iso-stamp-text.ts`)
or a round trip through the writer,
and an operator's text through the same rules,
saying what leaving the value off does;
`number-reads.unit.test.ts` fails on a new read until it names its rule.

## B77: tables keyed by text answered the names every object inherits

Found in B73 and left open there;
B75 made the fidelity probe's defect table a map,
and this entry reads the rest of the package.
A plain object read through a key the package did not write answers `constructor`,
`toString` and the other names on `Object.prototype` with the prototype's values,
and answers `__proto__` with the prototype itself where it holds no entry of that name;
written through `__proto__`,
it sets its prototype instead of holding the entry,
so the entry is gone without a word.
A JSON file that records a `__proto__` key does read back,
since `JSON.parse` defines it as the object's own,
so the defect needs a key the object does not hold,
or one the code writes.
Found by reading the code;
no stored output was searched for a key spelled this way.

Five sites read a key from outside the package
(the red cases in `42eb0b0dd` each failed on its assertion before the fix):

- `anthropic-delta-scan.ts` routed a stream delta by the type the provider names,
  through `DELTA_CHANNELS` and `DELTA_TEXT_FIELDS`;
  a delta typed `__proto__` or `constructor`,
  with text in the field the inherited value's string form names,
  yielded that text with `Object.prototype` as its channel;
- `image-asset.ts` read an asset's media type off its extension through `MEDIA_TYPES`,
  and answered `tabby.constructor` as usable;
- `corpus-run/judge-crosscheck.ts` placed a claim in an arm by its issue's stored status through `ARM_OF_STATUS`,
  and filed a status `constructor` or `__proto__` under the `Object` function or `Object.prototype`;
  the status count `score-crosscheck` prints,
  counted into `{}`,
  printed the `Object` function's text in the count for `constructor` and dropped `__proto__`;
- `repair-sheet.ts` printed the inherited function as the note for a stored disposition `constructor`;
- `corpus-run/attempt-store.ts` held the attempt counts the corpus pass keeps by entry id as a plain object,
  so an id spelled as an inherited name counted as the inherited value,
  and a count for `__proto__` was not written.

`f55cb82a0` first moved the two reads that sat in runner bodies,
the attempt count and the status count,
into `attemptsOf`,
`countAttempt` and `statusBreakdown`,
with no change in what they do,
so cases could reach them.

The other tables and records read or wrote only keys the package itself chose,
and changed for one rule across the package:
the three canon tables in `quote-normalize.ts` and `CLOSER_OF` and `OPENER_OF` in `translate-quote-balance.ts`,
keyed by one UTF-16 unit,
which no inherited name is;
`CANADIAN_PAIRS`,
which reached its readers only through the `CANADIAN_SPELLINGS` map;
`nodeEntries`,
the build's own entry names;
`ADMISSIBILITY_FIELDS`,
looked up by the names of the counts the reader itself declared;
and records filled by ids or names the package itself produces
(claim,
cluster,
issue,
seed and envelope ids,
repair statuses and dispositions,
refusal reasons,
recall bands),
in `adjudicate-wire.ts`,
`tally-resolution.ts`,
`restoration-judge-wire.ts`,
`derivability-wire.ts`,
`repair-scorecard.ts`,
`repair-region.ts`,
`corpus-run/editor-width-probe.ts`,
`corpus-run/sentinel-probe.ts` and `corpus-run/recall-benchmark.ts`.

Fixed (`2293b0ded`):
a table looked up by text is a `ReadonlyMap`,
and a record filled by a key is a `Map` until it is handed on whole,
as an object built by `Object.fromEntries` at its function's return or its file's write,
so the shapes callers and stored files read are unchanged.
The attempt counts are a `Map` in memory and an object only in their JSON file.
A table over a closed set of names keeps the names in its type instead,
so the compiler refuses a lookup by unchecked text:
`ADMISSIBILITY_FIELDS`,
which the reader now walks by its own list of counts rather than by the declared record's entries,
`perBand`,
and `HYPER_CREDIT_RATES`,
checked with `satisfies` rather than typed as a record of strings.
`nodeEntries` and `CANADIAN_PAIRS` are maps rather than literal-keyed objects
because `isolatedDeclarations` (TS9010) needs an annotation on an exported constant,
and an annotation widens the keys to `string`;
the build configs hand rolldown `Object.fromEntries(nodeEntries)`.
Written one tuple per line,
the pairs table went past `eslint(max-lines)`'s 300 counted lines (490),
so its doubled-l and word-by-word groups moved to
`corpus-run/canadian-spelling-pairs-doubled-l.ts` and `corpus-run/canadian-spelling-pairs-word.ts`,
spread back in their places so the map's order is unchanged.

Found on the way and fixed in the same commits:
the media-type note said the table had two entries because the corpus had two extensions,
where the table had four from the commit that wrote the note (`158117e79`);
counted over the data repository at the pin,
the corpus's pictures are 210 `.webp`,
91 `.jpg`,
6 `.png` and 1 `.jpeg`,
and the note now says so.
The pairs table's TSDoc called every Canadian form lower case,
where `id` maps to `ID` (class 169).
And the comment over `score-crosscheck`'s status count said it counted control-arm claims only,
where it counts the undecided claims too (`f55cb82a0`).

Calls made here are open to veto:

- every table looked up by text is a map,
  the ones whose keys could never be an inherited name among them,
  so one rule covers the package and the scan lists no exceptions;
- records filled by a key are maps inside the function and objects outside it,
  rather than maps all the way to the stored file,
  which would change what every reader and stored artifact holds;
- tables over a closed key set keep literal keys rather than becoming maps,
  since the compiler then holds every lookup to the set;
- `judge-crosscheck.unit.test.ts`'s two suites sit under one root `describe`,
  so a failure in the first no longer stops the second from running (ledger M97).

Guard-offs on the fixed tree,
restored with `git diff` printing nothing after:
with `DISPOSITION_NOTES` a plain object again,
the scan's package case failed (`expected [ Array(1) ] to deeply equal []`)
and the repair sheet's inherited-name case failed (`expected false to equal true`).
The red commits show the rest:
each red case failed on its assertion against the unfixed code,
and the scan's package case listed every site before the fix
(`f0322ce6f` and `98f5b8861`).
The full suite on `d2af668bd`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1,493 PASS lines and no FAIL line.

What the scan cannot see was read by a typed census of the fixed tree
(`~/temp/agent/audit-glossary-fix/b486-prototype-census.mjs`,
TypeScript 7.0.2's checker over the package's 2,005 source files),
which lists every element access and `in` test whose key is not a written literal.
In production code,
no write goes through a key typed as text,
and the 63 reads and 3 `in` tests keyed by text each take a field name the package chose
(read through a helper whose callers pass a written name),
an id the package minted,
or a key `Object.keys` gave from the same record;
the 4 keyed by a type parameter are keyed by a closed set.
A second probe (`b486-casts.mjs`) found no cast narrowing text to literal keys,
and two computed destructurings,
both of `process.env` by the package's own variable names.

Recurrence:
a table looked up by text is a `ReadonlyMap`,
and a record filled by a key is a `Map` until handed on whole;
`text-keyed-tables.unit.test.ts`,
which `source-scans` runs,
fails on a declaration typed with string keys whose value is an object literal at a module's top or `{}` anywhere,
or that is written through a computed key.
Out of its reach,
and named in its module note:
a table built by `Object.fromEntries` or handed back from a function and then read by text,
a class field,
a parameter destructured from an object an alias types,
a key typed by a template literal,
and a cast of text to a table's literal keys.

## B82: timing readers that read less than their writers write

Found by T8's nineteenth batch,
whose red commit `71f7720a9` cases all four,
and fixed in `43e444768`.
The run timing readers (`corpus-run/run-timing-parse.ts` and `corpus-run/run-timing-read.ts`)
read back the round and completion lines a run logs,
and `run-timing-report` prints what they read.

- A round whose quorum never stood read as a line about something else.
  `runGatherRound` (`stage-round.ts`) writes such a round as
  `<stage> round: h/n heard, Tms total, no quorum (h of k needed), every ask settled`,
  and the reader knew only the line that ends in grace,
  so the report left the round out of its count,
  its total time
  and its voices never heard.
  `readRoundTiming` now reads both shapes into a `quorum` that either `stood`
  (time to quorum and time in grace)
  or was `never` reached
  (the count needed),
  and refuses a no-quorum field whose heard count is not the ratio's.
  `summariseRounds` folds every round into the report's figures
  and sums grace only where quorum stood.
- A round line short of its four fields was refused as a field that "carries no ms unit",
  naming a unit where a whole field was missing.
  It is now refused by its field count against the four its writer writes.
- A completion line whose first field is not `<label>: <outcome>` read with an empty outcome.
  `reportStreamProgress` always writes `stream <label>: <outcome>`,
  so such a line is now refused as an unreadable completion line.
- `measureInFlight` divided by a zero span as if it held calls:
  timed calls that all took no time at one instant
  reported every call in flight on average
  while the sweep,
  which takes an end ahead of a start at one instant,
  counted a peak of none.
  A zero span is now refused with `NothingInFlightError`,
  as an empty call list already was,
  and the report says which empty span the calls left.

The refusals are named classes:
`TimingFieldError` for a field,
`TimingLineError` wrapping it for a line,
`NothingInFlightError` for an empty span.
`NothingInFlightError` carries the forwarding marker,
since both its sentences are fixed;
`TimingLineError` is on the message scan's withheld list,
because it quotes the whole line it could not read.

Running the built report on a throwaway log
(both round shapes and two calls of no duration at one instant)
printed the no-quorum round in every figure
and found one more defect:
the in-flight refusal followed its heading after a period,
so the refusal's lower-case sentence read as a broken one.
`8175dc866` joins them with a colon
and brings the corpus-pass runbook's quoted output to the report.

Guards shown to fail:
the four red cases failed in `71f7720a9` against the code before the fix;
with the heard-count agreement check disabled,
the case naming what a round line contradicts failed alone in its file;
with no-quorum rounds left out of the fold,
the `summariseRounds` case failed alone in its file.

Calls made here are open to veto:

- a round is one of two kinds rather than one shape with optional fields,
  so no reader can take a no-quorum round's missing grace for a zero;
- the refusals are named classes rather than plain errors;
- a zero span is refused rather than reported as a mean of zero;
- a completion line missing its outcome is refused rather than read as an empty outcome;
- the report prints no grace share where the rounds took no time,
  since a NaN percentage reads as a measurement.

Open:
the fixture lines are copies of the writers' templates,
so a case that builds each line through `stage-round.ts` and `reportStreamProgress` themselves
would catch a new shape the copies miss.

Landed 2026-10-02 (UTC):
`runGatherRound`'s line is built by `roundLine` (`stage-round-line.ts`),
which its own unit test pins for a round whose quorum stood and one whose quorum never did,
and the run-timing fixtures build every line the current writers still produce through `roundLine` and `reportStreamProgress`.
Their tag prefix is the chain a real corpus pass logs,
traced from `corpus-run/pass-lanes.ts` to the round;
the replaced fixtures carried an `[editor]` tag no caller writes.
The one line still typed by hand is the pre-timing completion shape,
which `reportStreamProgress` can no longer produce.

Recurrence:
a reader of a log line the package itself writes
reads every shape that writer writes,
and is tested against lines built the way the writer builds them;
`mistake-prevention.md`,
"Readers of the package's own log lines".

## B132: a spend record with no logger prefix ended the cap census

Red in `66fa8de2e`,
fixed in `83942a8b4`.
Found on 2026-10-05 (UTC) deciding the T8 hunks of `corpus-run/cap-census-read.ts`,
and independently by a review of the trial's cases.
The trial turned `stampOf`'s `line.split('] [',)[1] ?? ''` into `nonNullishOrThrow(...)`,
on the reason that both line markers carry the bracket the split reads.
The stream marker does (`'] [reportStreamProgress] stream '`);
the spend marker is `'SPEND '`,
and `readSpendLine` reads a record at the start of a line or behind any space.
So `readCapLog` handed `stampOf` a spend line with no `] [`,
and the census threw `Expected non-nullish value, got undefined`,
where B73's rule is that a line whose stamp the logger did not write is left out and counted.
The red run printed 3 [PASS] and 3 [FAIL] lines.
No run log was read,
so whether a census ever met such a line is not known;
the spend reader documents the bare form as one it reads.

The fix:
`stampOf` returns `unstamped` where the line has no second bracket,
and a `cap-census.unit.test.ts` case pins a bare spend line and one behind `[info] `
as left out and counted.
The same file's wrong-unit case could not tell a stream line read from one skipped;
it now follows each misread stream line with its spend line.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B198: a rendering that holds a colon was written twice into its heading

Red in `6f61ba5dd`,
fixed in `72f5be638`.

Found on 2026-10-05 (UTC),
as an item ledger M113 left open.
`restoreHeading` (`corpus-run/contributor-name-restore.ts`) cut the page title at its last colon to find the prefix,
so `### Three: Mao: Cat` with the rendering `Mao: Cat` read `Three: Mao:` as the prefix and `Cat` as the name,
and wrote `### Three: Mao: Mao: Cat` with a restore reported.

The fix:
the prefix ends at the page title's first colon,
as `titleNames` reads the original,
whose prefix holds no colon while its name may.
`lastColonAt` (`corpus-run/assembly-page-text.ts`) had no reader left and is gone.
`restoreSignature` reads its name by offsets and has no such shape;
the case holds a signature as its control.

No page changes
(the fixing agent's count over the pin):
none of 251 original headings and none of 251 archive headings holds more than one colon,
and none of 24 signature names holds any.

Open to the owner's veto:
a page prefix that itself holds a colon is read as part of the name,
where the other way takes any colon whose suffix already carries the rendering.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text":
a cut at a delimiter is a first or a last match on a key nothing makes unique.

## B274: a parse offset was read out of a file's own words

Red in `f45278048`,
fixed in `71974714f`.

Found on 2026-10-06 (UTC) by the coverage cluster over the library files from m to z (T8).
`offsetIn` (`run-json-read.ts`) took the digits after the first ` at position ` of V8's refusal
as the offset the parser stopped at.
V8 quotes a short file back whole,
so a file reading `cat at position 7` was reported unreadable `at byte 7`,
an offset no parser stated.

The fix:
a refusal that ends ` is not valid JSON` quotes the file and states no position,
so it yields none.
A V8 refusal with a position phrase and no whole offset,
and `JSON.parse` throwing a value that is no error,
are `unreachable:` throws.
On the lead's red tree `run-json-read.unit.test.ts` failed by assertion.

Measured by the lead for B274 to B276:
of 48 named test files 47 passed and the duplicate-bodies scan failed,
until a shared fixture closed it (M123);
with the fixture in place the two test files that use it pass,
with the 39 source scans and a clean type check.
The whole suite ran on `71974714f` in a side checkout:
1,656 `[PASS]` lines and no `[FAIL]` line,
the 39 source scans passing,
and the lint ending with one warning,
which `230623bec` removed (M124).

Open to the owner's veto:
the position phrase throwing if V8 changes its wording.

Recurrence:
`mistake-prevention.md`,
"Numbers read back from text":
text a message may quote is never searched for a phrase the same message uses as structure.

## B292: a request rate with a fraction made the pace wait a whole window

Red in `3c74be74a`,
fixed in `040656c51`.

Found on 2026-10-06 (UTC) by a coverage agent of T8,
whose probe of a fallback in `request-pace.ts` the follow-up's brief names,
and fixed by that follow-up.
`hyperRequestsPerHour` read any plain decimal of `TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR` as a rate.
A fraction made the pace read its list of starts at a fractional position,
miss,
and wait a whole window.
With starts at 0 and 600 ms in a window of 1,000 ms,
a pace of 1.5 places read a wait of 1,000 ms at 700 ms where two places read 300
(the agent's probe,
not rerun by the lead).

The fix:
`hyperRequestsPerHour` refuses a value that is not a whole number of one or more written in digits,
as a stated refusal naming the variable,
the value as written and what is accepted.
`createRequestPace` throws a `RangeError` for a count of places that is not a whole number of one or more,
so a pace built by hand cannot reach that read either.
The mode that paced nothing for a count of zero or less goes with it;
no production caller used it,
and its cases are removed.
The two fallbacks on the list of starts are `nonNullishOrThrow` reads:
the first sits behind a length check,
and the second's index is in range for a whole count.
The red cases build a pace with 1.5,
0,
-1,
`NaN`,
`Infinity` and 2 to the 60th,
and set the variable to `1.5` and `0.5`.

With it,
no defect:
`sleepBackoff` (`transient-retry.ts`) rethrew whatever its timer rejected with when the caller had not aborted.
By the agent's reading of the runtime's own `timers/promises` source and its probe on Node 26.10.0,
a timer given a number and an abort signal rejects only when that signal aborts,
so no input reaches the rethrow.
It is an `unreachable:` throw that keeps the rejection as its cause.

`doc/configuration.md` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) said a bad value of
the variable was not refused and silently left the default,
which the code had already stopped doing,
by the agent's reading of the ledger;
it says what is refused,
and `doc/seats-and-calibration.md` (moved on 2026-10-18 to `package/translation-repair/doc/seats-and-calibration.md`)
says the rate is a whole number
(both the lead's edits,
in `040656c51`).

Measured by the lead on the red tree for B292 and B293:
the whole unit suite prints 1,823 pass lines and 10 fail lines,
every one of the 10 in the two red files
(two cases of `request-pace` and four of `stream-drain`,
with the lines of the suites that hold them),
each by assertion.
The whole suite's run on `040656c51` is recorded in B291.

Open to the owner's veto:
removing the mode that paced nothing;
the wording of the refusals and of the `unreachable:` throws.

Recurrence:
`mistake-prevention.md`,
"Numbers read back from text".
