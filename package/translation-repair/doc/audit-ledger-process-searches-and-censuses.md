# Audit ledger: process mistakes, searches and censuses

Searches and censuses that missed what they looked for,
or were read as complete when they were not.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M13: a pin search that quoted a whole clause

Status:
caught by the full suite before commit (`112b399a5`).
Before rewording `KEPT_SUBJECT_RULE` the search for pins quoted the clause "says that I (or that person)";
`source-subject-policy.unit.test.ts` pins it split across two string lines,
so the search found nothing
and the full suite failed on it.
Prevention:
search for a pin of a sentence by a fragment of four words or fewer,
and by each end of the sentence;
the full suite before commit (M2) stays the backstop.

## M24: a census read a helper's fields by their names, not their contract

Status:
corrected before use,
2026-09-27 (A17).
The A17 census and the first version of its floor sliced each signer out of the slice text with
`Signature.nameStart` and `nameEnd`,
which are offsets into the signature's own line;
the census reported 75 of 82 page signers unglossed over names that were not names,
and a first pass of that census,
reading the would-ship kind as `text` where it is `wording`,
had reported every signer unaligned.
Both were caught only because a result looked wrong or a guard failed.
Prevention:
a census that calls a helper reads the helper's type doc for every field it uses,
and prints one decoded example of what it counts (ids and code points only) before it prints a total;
a result where every item lands in one bucket is a defect in the census until shown otherwise.

## M33: a search capped with `head` read as complete

Status:
caught by the type check,
2026-09-28 (P4).
Removing `DEFAULT_JUDGE_MODEL_IDS`,
its users were listed with an `rg ... | head` that cut the list,
and `roster-reach.unit.test.ts`,
past the cap,
still imported it;
the build's type check named it,
and an uncapped search then showed no other user.
Prevention:
a search whose result decides what to change runs uncapped,
or with `--count` first;
`head` is for reading samples,
never for a census.
Again during D20,
capped by scope rather than by `head`:
the readers of the removed fields were searched in four named test files,
and the type check named `pair-blocks-evidence-identity.unit.test.ts` and `prepare-block-scope.unit.test.ts`.
A census searches the whole of `src`,
then narrows.
Again on 2026-09-30 (B52),
caught by the package's unused-imports scan:
the driver's import of `NaturalnessCompletenessError` was removed by a `sed`,
and the check of that removal,
`rg ... | head --lines=8`,
cut its output before the import's line,
so the import that the `sed` had missed read as gone.
`mise run source-scans` named it,
and it was removed with Edit before the commit.
A check of an edit's result is a census of that file too:
uncapped,
or `--count` first.

## M35: a teardown that kept what only its consumers read, and an audit that listed the sites it saw

Status:
fixed under D16 and D20,
2026-09-28.
`cbedea357` removed the receipt,
replay and calibration layer by import closure,
which finds modules nothing reaches but not comments,
fields and test names inside modules that stay:
the surviving pairing modules kept describing the removed consumers,
and two fields survived that only those consumers read.
The audit then recorded that remnant as the two comments it had met (D5,
D16),
where the removed layer's vocabulary found the same framing across eight source files.
Prevention:
a removal is followed by a search of `src` for the removed layer's own words
(its module names and the concepts its commits introduced),
and by a census of every export and field whose only readers were removed,
tests included;
an audit finding about a remnant searches that vocabulary before it writes a site list.

## M36: a search for an entry id printed corpus lines

Status:
happened 2026-09-28 during H9;
nothing was written to a file or a commit.
Looking for which entry an H9 observation came from,
an `rg --only-matching` over the session transcript
took up to 120 characters after a corpus term,
and printed lines of the poem around it,
against the rule that probes print ids,
indices,
counts,
code points and markup only.
Prevention:
find an entry by `--files-with-matches` over the pinned clone,
or by `--count` first;
an `--only-matching` pattern never carries wildcard context around corpus text.

Recurred 2026-09-29 during D28,
again with nothing written to a file or a commit:
looking for provider balances in `TianqiChen66621.log`,
an `--only-matching` search with 60 characters of context either side of "spent"
printed three fragments of model reasoning that quote the page.
A run log carries the models' reasoning,
which quotes the corpus,
so the prevention covers run logs too:
search a run log by its line tags (`TALLY`,
`SEAT`) or its JSON keys,
never with wildcard context around an ordinary word.

Recurred again the same day during D32:
a probe listing placeholder links masked every non-ASCII character as its code point
but printed the ASCII link text around each,
which is page text (film and song titles).
A probe masks the text it prints,
not only its script:
print the markup and a length,
or a hash,
for any text a page wrote.

Superseded in part on 2026-10-05 (UTC),
when the owner ruled:
"I've allowed you to handle PII.
The corpus is public."
The lead's reading,
open to the owner's veto:
a probe need not mask corpus page text,
and a doc may quote it where the quotation helps.
What stands:
credentials and raw provider requests and replies stay private,
so a run log,
which carries provider replies,
is still searched by its line tags or its JSON keys;
the corpus stays out of the repository,
since it is unlicensed;
unit fixtures stay invented;
and a session transcript is still neither searched nor quoted,
since it holds more than the corpus,
so an entry is still found in the pinned clone.

## M39: a key change committed on a census of callers, and a commit message and its correction garbled

Status:
happened 2026-09-28 in X13;
the missed test fixed in `b84c2384c`,
the messages corrected by a commit comment.
Before `fea6e4688` the census of what reads the pairing keys searched for calls to the key builders.
Two of the tests it found also derive the key by hand,
which surfaced only when they went red;
`corpus-run/pass-footnote-lifecycle.unit.test.ts` calls no builder and joins the key material itself,
so it went red only in the full suite.
Afterwards a search for tests joining the version and a separator returned one of those three,
while a search for every test hashing with `createHash('sha256'` found all four hashing tests:
the M33 lesson again.
The message of `b84c2384c` then carried a garbled phrase,
and the first correction comment on it
carried a placeholder where a hash belonged;
the comment was edited to the resolved hash.
Prevention:
census a key by the material it hashes (`createHash`,
the version constant,
the separator),
never by the names of its builders,
and sanity-check a search that returns nothing with a broader one.
Read a commit message back before committing,
and resolve every hash a message or comment names with `git log`.

## M40: a source file with raw NUL bytes, skipped by every line search

Status:
fixed in `aacf919eb`,
2026-09-28.
`prepare-section-round.ts` carried two raw NUL bytes as its key separator since `3e93519df` (2026-08-23),
so `rg` and `grep` treated it as binary and printed one "binary file matches" notice in place of its lines.
Every line search over `src` since then got that notice,
or nothing once piped through a filter;
X13's fix edited the key beside the raw bytes without noticing them,
and this audit's outline of the file came back empty until an unfiltered search printed the notice.
Prevention:
a "binary file matches" notice from a search over source is a finding,
never noise.
A scan for control bytes other than tab,
newline and carriage return
(`rg --text --files-with-matches '[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]'`,
positive-controlled on a scratch file)
found no other file in the package outside `node_modules`,
`dist` and `.cache` after the fix,
and belongs in the pre-launch checklist.

## M48: a census of a rule that searched the names of functions holding it

Status:
happened 2026-09-28 (audit area six,
letter predicates);
found when the prose-scanner switch widened its search.
The ASCII letter merge replaced 11 local copies,
found by searching for named predicates and their bodies,
and the plan for switching prose scanners to Latin letters was drawn from the same list.
The same test written inline was never searched:
`(character >= 'a') && (character <= 'z')` inside a larger function,
character-code ranges (`align-blocks.ts`),
literal alphabets and digit strings,
case-fold tests (`lower !== upper`),
and regex classes and properties (`\p{L}`).
A search by shape found inline letter tests in 13 more source files and a test file,
among them prose scanners the plan lacked
(`reading-refusal.ts`,
`assembly-content-survival.ts`,
`lexical-restoration.ts`,
`line-structure-guard.ts`,
`archive-italic-spans.ts`),
plus digit and hex tests,
and further definitions of a word character
(`\p{L}\p{N}` in `declared-name-survival.ts`,
cased letters by regex in `quote-neighbours.ts`,
cased letters by case fold in `canadian-date-parts.ts`,
`handle-gloss-place.ts` and `prose-ranges.ts`).
It also found three `opensTag` definitions that disagree on which character after `<` opens a tag.
Prevention (the prevention doc's copies family):
a census of a rule searches every way the rule can be written,
not the names of functions known to hold it:
range comparisons on characters and on character codes,
literal alphabets,
case-fold comparisons,
and regex classes and properties,
each query with a positive control.
The shape search itself then missed the negated form,
`(character < '0') || (character > '9')`,
which held four whole-text digit tests and a lower-case letter test (`baed88f59`);
a search for reversed operands (`'0' <= character`) and for character-code arithmetic found none,
while its control pattern matched 2.
So every shape is searched in both its asserted and its negated spelling.

## M67: a cold stretch recorded under the source of its first character alone

Status:
happened 2026-09-29 in `f09ec0eb3`,
which mapped a cold stretch through its first and last characters,
caught the same day by a census of one batch's test files,
fixed in `25dc7e9f6`,
with baselines read before the suite in `740dd721e`.
A bundle chunk concatenates modules,
and V8's cold runs merge across them,
so a stretch recorded under its first character's source put every later source out of the census.
The census of eight test files recorded bundle offsets 19,599 to 29,168 of the repair-chunk bundle as line 172 of
`src/select-candidate.ts` (moved on 2026-10-18 to `package/translation-repair/adjudication/src/select-candidate.ts`)
while listing `repairChunk` as never called,
and gave `src/repair-chunk.ts` (moved on 2026-10-18 to `package/translation-repair/repair/src/repair-chunk.ts`) no
stretch at all;
read against a baseline,
that source's stretches then counted as ran.
The same mapping wrote the baseline at `e22373347`,
the scratch census before it
(`t8-cold-map.mjs` in the audit's scratch folder maps a stretch's two ends the same way),
and the first batch's proof,
so the port control that reproduced every total of the scratch census proved the two agreed,
not that either was right,
and neither mapping control could see it:
both read single functions,
which never cross a module.
The fix maps every character of a stretch and records one stretch per run of one source;
rerun on the same eight test files,
the offsets once put at one line are four records,
the last `src/repair-chunk.ts` (moved on 2026-10-18 to `package/translation-repair/repair/src/repair-chunk.ts`) lines
110 to 497,
which holds `repairChunk`'s first line.
A second defect sat in the same reader:
Node returns a map segment naming no source as an entry whose source is undefined,
and the census called a string method on it;
no census is known to have met one
(the coverage build's maps held none when counted).
Prevention:
a measurement is checked against an invariant any correct output satisfies,
not only against an earlier tool's totals:
the census now refuses to write a report in which an uncalled function's first line sits in no stretch of its own source,
and refuses a baseline written before the fix
(its format lacks the split).
A census read that "holds no stretch" or "ran" is evidence only from a census of format 2 or later.

## M79: a coverage census measuring compressed code

Status:
found 2026-09-30 (UTC) while reading the sixth T8 batch's stretches,
fixed in the commit that switched the coverage build to `minify: false`
and made the census refuse a minified build (`requireUnminifiedBuild`).
The coverage build took the shared Node output options,
which compress,
and compression rewrites control flow:
`if (x) continue;` becomes `!x&&(…)`,
a chain of `if (…) return false;` guards becomes one `||` expression,
and an `if` whose arms swap places maps its cold arm across the branch between them.
V8 gives the untaken side of a logical expression no range,
so a guard no test reaches read as run,
and the swapped arms stretched cold spans over lines that ran.
The ledger's T8 entry had seen the stretching twice and concluded the error ran only toward calling code cold,
without a build that could show the other direction;
the first such build lists 145 more stretches on the same commit (the T8 entry has the comparison).
Prevention:
a measurement reads code as its source writes it,
so the coverage build keeps every statement (no minification),
and a claim that a transform's error runs one way is tested by measuring once without the transform.

## M90: a census probe narrower than the family it counted

Status:
happened 2026-10-01 (UTC) during B73,
found the same day writing the guard from it,
before any fix rested on its count.
`number-reads-probe.mjs` counted calls of `Number`,
`parseInt`,
`parseFloat`,
`BigInt` and the two `Number` members,
and unary plus:
35 keys over 1,075 production files.
It did not look for a reader handed on as a value (`.map(Number)`),
nor for `Date.parse` and `new Date`,
which read numbers out of text the same way.
A search for `Number` as an argument found two,
`admitWithinShortfall`'s callers reading back positions written as text,
and a search for date parsing found four,
three of them log-stamp readers that took a stamp without its zone as local time (ledger B73).
Prevention:
before counting a family,
list every way the language reaches the behaviour
(a call,
a construction,
a value handed on,
a coercion)
and every function with the same behaviour on another type;
the guard then says which it reads and which it leaves out of reach.

## M98: a source scan written without listing the forms of what it scans for

Status:
happened 2026-10-01 (UTC) during B77,
twice,
the first caught before the scan's commit and the second after it,
with no fix resting on either.
Drafted,
the text-keyed table scan read a `satisfies` as typing a table with string keys.
A `satisfies` checks without widening,
so the keys stay the literal's own and a lookup by unchecked text does not compile;
the draft flagged tables the compiler already held,
`CLAIM_BY_ROLE` among them.
Committed (`f0322ce6f`),
it read a parameter as typed with string keys only when the parameter was a name annotated in place,
and missed one destructured from an object,
the form every function of two or more parameters takes here.
Moving the attempt count into `countAttempt({ attempts, id })` dropped the package findings from 36 to 35 with the write still there,
which is how the gap showed,
and `98f5b8861` closed it.
M90's family,
met again in a guard's scan.
Prevention:
before a scan is written,
list every way the language declares the shape it looks for
(an annotation,
a cast,
a `satisfies`,
an alias,
a parameter named,
destructured or defaulted)
and say for each whether it gives the shape;
the fixture holds one of each,
and a count that moves when code moves without changing what it does is read as a gap in the scan.

## M99: a search whose pattern was refused, read through its fallback as finding nothing

Status:
happened 2026-10-01 (UTC) during B77,
caught before any conclusion rested on it.
A search for casts used a lookahead,
which `rg`'s default regex engine refuses,
and the call's `|| echo` fallback said no casts were found,
printing for the refused pattern exactly as it would for an empty search.
The search was rerun with `--pcre2`,
then replaced by a typed probe (`~/temp/agent/audit-glossary-fix/b486-casts.mjs`).
Prevention:
a fallback report names the exit status (`|| echo "exit $?"`),
since `rg` exits 1 for no match and 2 for an error,
and a null is read only with the command's error output in view.

## M108: a whitespace-only check that counted the diff's own markers as content

Status:
happened 2026-10-02 (UTC) while checking `7f529468e`'s diff,
and caught before the commit.
To show the normalization changed whitespace alone,
one check counted added diff lines holding a non-space character,
expecting none,
and read 16,849:
each added line still carried the diff's leading `+`.
The other check,
`git diff --ignore-all-space --stat`,
printed nothing and decided it;
the commit message cites that check alone.
Prevention:
a check written to come back empty is first run on a case that must not
(here a line whose added content is not whitespace),
and a check over diff text strips the diff's markers before reading content.

## M122: a fix committed after a named run that left out a file holding the old wording

Status:
happened 2026-10-06 (UTC) in `4c26a48a0`,
found by the whole-suite run on that commit,
repaired in `495f040b6` (B235).
The fix changed what `StreamCutShortError` says of its cause.
Before the commit the lead ran 35 named test files,
and `stream-idle-guard.unit.test.ts`,
which asserted the old wording,
was not among them.
The suite was red at that commit:
1,624 `[PASS]` lines and 3 `[FAIL]` lines,
all three one case and its two enclosing suites.
Prevention:
when a message's wording changes,
the named list holds every test file that holds the old wording,
found by a search for the old words over `src` and not by the list of files the patch touches.
`mistake-prevention.md`,
"Searches and censuses that miss".

## M128: a probe whose positive control used another pattern than the probe

Status:
happened 2026-10-06 (UTC),
caught before the count was relied on.
To count the corpus pages that hold a byte order mark (B257),
the lead first ran `git grep --perl-regexp '\xEF\xBB\xBF'`,
which found none,
and its positive control used another pattern (`^---`,
276 files).
The pattern matched the code points U+00EF,
U+00BB and U+00BF,
and not the three bytes.
Read by bytes in a script,
with a control on the same test,
one file holds the mark.
Prevention:
a positive control exercises the same pattern and the same tool as the probe,
on a text known to hold the thing looked for.
`mistake-prevention.md`,
"Searches and censuses that miss".

## M170: an outcome that was rare became common and nobody re-read its consumers

Status:
happened on 2026-10-13,
found by the sixth review (B479 and B480).
The test `edits.length < receivers.length` in the block carry was harmless while `unavailable` was rare.
The mark rule made `unavailable` common,
and two consumers then withheld work from places that could have been carried:
the splice offer of a block,
and the writability of a paragraph.

Prevention:
when a change makes an outcome more frequent,
a search lists every consumer of that outcome and each is re-read.
`mistake-prevention.md`,
"Searches and censuses that miss".

## M181: a count of zero in the corpus was read as a shape that cannot happen

Status:
happened on 2026-10-13 in the brief of the proven-apostrophe work (B488),
found by the seventh review on 2026-10-14 (B492).
The brief set the scope of the proof as "the paragraph",
taking the unit from the corpus count that came first,
and did not ask whether a quotation can cross a paragraph.
The count over 92 pages found no such case,
and the brief read that as "cannot happen".
The review built eight small pages (B1 to B3,
F1 and N1 to N4 of B492) in which a replace deleted a quotation mark:
the closing mark in six of them,
the opening mark in one,
and both in the last.
In four the quotation opened in another paragraph,
and in four the opener was glued to a letter or was `‚`,
`‛` or `‹`,
none of which the rule counted as a partner.
Hand fixtures of that shape are what the oracle of the work had lacked (B499).

Prevention:
a corpus count sizes a cost and never proves a shape absent.
The scope of a rule is decided from what the language allows,
and each shape the count shows as zero gets a hand-built input before the rule leans on its absence.
`mistake-prevention.md`,
"Searches and censuses that miss".

## M258: a null over stored runs was taken from artifacts that predate the field under test

Status:
happened after the split of 2026-10-18 (UTC),
in the first check of stored artifacts for the finalist extension (`w180-notes/reader-check/`).
The check searched the pooled runs directory only and found 164 legacy artifacts,
none with a consistency record.
Every hand-picked pass writes to a throwaway `${HOME}/temp/agent/<run>/` directory
(step 1 of `doc/runbook/translation-repair-corpus-pass.md`),
so the consistency-era runs of TianqiChen666 were not in the pool,
and a null over artifacts that predate the feature cannot test the feature.

The notes record it as caught because a null over artifacts that predate the feature cannot test the feature.
The handover records the re-run over the five consistency-era TianqiChen666 run directories:
the new reader refused `corpus-pass-TianqiChen66624` group 0
(generation 17,
2 to 1 to 1 of 4,
no run-off),
which the old reader accepted,
so the artifact generation was bumped and generation 17 and earlier were recounted under the tied-only rule
(B657 to B664).
None of the 164 artifacts of the first search could have shown that refusal.

Prevention:
before a null over stored runs is trusted,
at least one searched artifact is shown to carry the field under test,
and the places where hand-picked passes write are listed as searched or not searched.
`mistake-prevention-guards-searches-and-claims.md`,
"Searches and censuses that miss".
