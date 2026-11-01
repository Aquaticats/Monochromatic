# Translation repair history 2026-08-24 to 2026-08-25: error text that quoted corpus wording, found, scanned for and closed

The unreadable run file that printed its own contents and the class of reads it exposed,
the sinks reached through a catch,
the parser errors that quote and the cause chains that carry them,
the scans of every error class and every cause,
and the guard proof that found the open call unguarded.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## An unreadable run file printed itself, and the fix was a whole class rather than one CLI

Filed as `#220` during the calibration,
deferred until the landing cleared,
then reproduced
on 2026-08-25.
It was filed as a crash with a minified stack.
Reproducing it showed something worse.

V8 gives a `JSON.parse` refusal a synthetic script whose source IS the text it was handed,
so Node's uncaught-exception report prints that line.
Against a throwaway ledger,
`ledger-report` printed a whole contest ahead of the stack trace:

```text
<anonymous_script>:1
{ "task": "whiskerfield-1", "at": "2026-08-25T00:01:00.000Z", "candidates": [ { "index": 0, "producers": ["tab

SyntaxError: Bad control character in string literal in JSON at position 110 (line 1 column 111)
```

A real ledger file holds candidate renderings and a person's entry id.
A garbled one published both to a terminal.

The package already stated the rule this broke.
`error-name.ts` records that a message is uncontrolled and that a run directory path can name
a person;
`LedgerShapeError` says outright that it NAMES,
NEVER QUOTES.
Nothing had applied either to the parse step.

### The class was twelve times larger than the report

Counting `JSON.parse` sites that no `try` encloses found 25 matches,
which is not the answer:
11 are TSDoc `@example` text and one parses a string on the write path.
The real class was 12 file-reading sites across 11 files,
every one under a run directory.

Fixing only the one that happened to be tripped would have left eleven instances of the same
contract violation,
which is the layer-1-only mistake `ELR` warns about.

`readRunJson` in `run-json-read.ts` is now the only way a run file is read.
It refuses with `RunJsonUnreadableError`,
naming the file's basename and the failure and
carrying no text from it.
A parse offset survives as a number,
because a truncation point is what tells an operator what
happened and it is content-free.
`parseRunJson` splits out for the two callers that hold text rather than a path.

`slice-cache-namespace.ts` is deliberately left alone.
Its `serialized` argument arrives from `persistSlice` on the WRITE path as a lane's own
in-memory serialization,
so it never reads a file.
That was traced to the call site rather than assumed.

### Two lint rules disagreed, and the answer was to delete the code

Reading the digit run by hand needed either a mutable cursor or a character array.
`no-function-root-let` forbids the first,
and `prefer-spread` and `no-misused-spread` forbid
each other on the second.
Per `LN1` the remedy is structural,
not picking a surface to silence:
`Number.parseInt` already reads a leading digit run and stops,
so the hand-written scan is gone.

### The test that could not have failed

Writing the absence assertion exposed a trap worth remembering.
V8 quotes only the FIRST TEN CHARACTERS of a file back inside its refusal message,
so a fixture
word of `Marmaladeslept` appears as `Marmalades`,
and a test asserting the full word absent
passes even against a reader that forwards the message whole.

The fixtures now lead with `Bixbyfluff`,
exactly ten characters,
confirmed against `JSON.parse`
directly to appear in the message it produces.
The same measurement corrected a truncation offset guessed at 30 to the real 27.

GFP-proven:
breaking both guards,
so `readRunJson` forwards V8's message and `refusalOf`
forwards every class's,
fails `readRunJson` on two children and `refusalOf` on exactly the
foreign-message case,
exit 1.
Restoring returns 680 PASS,
0 FAIL.

### What the boundary check found that the unit tests could not

Driving `rendering-audit-settled-report --run` against a malformed file confirmed the leak is
closed there too:
zero hits for the fixture's distinctive word,
with the file's name appearing
twice as a positive control that the parse was genuinely reached.

It also showed the OTHER half of `#220`,
which `#222` did not close:
the refusal was safe but still uncaught,
so Node printed the minified bundle line,
around three thousand characters of it,
around a correct one-line message.

`reportingRefusals` closes that.
It catches ONLY `RunJsonUnreadableError`,
because catching every `Error` would hide the stack
of a genuine programming fault,
and the forwarding case is tested to hold that line.

Which CLIs needed it was measured off the built bundles' own import graph rather than guessed:
11 of 39 entry bundles can reach the reader,
one is the library barrel,
one is `ledger-report`
which reports its unreadable files as a shortfall inside its own output,
and the other nine are
wrapped.
The same throwaway now yields two lines and 211 characters at exit 4.

### The test suite taught something about its own runner

Written with three sibling cases,
the suite failed.
`describe` runs children concurrently by default,
and all three cases swap process-global state:
one saw zero captured lines because a sibling had already restored `console.error`,
and another
read `undefined` where it had just written zero.
`concurrency: 1` is documented for exactly this.
Both swaps are process-wide,
and the runner spawns `node` once per test FILE,
so nothing outside
the file was ever at risk.

A grep counting `PASS` and `FAIL` lines reported 681 passes and zero failures on that failing
run,
because the runner reports a file-level failure in its own line.
The exit code was right and the count was wrong,
which is what `TLY` says to expect.

Landed as `768d26b18`,
`ba83d021c`,
`7a4f27db0` and `a72d9b6fb`.

### One observation recorded rather than acted on

The suite output shows the pipeline's own warn logs forwarding a model's raw non-JSON answer,
`raw="not json at all"`,
alongside the `SyntaxError` message that quotes it.
That is the same shape as the unguarded-parse defect,
but the exposure is different:
those lines go to a run log inside a run directory that already holds corpus wording,
and the
owner's instruction is to log more rather than less.
Naming it here so the difference is a decision rather than an oversight.

## The same defect had a second half, reaching a sink through a catch (`#224`, 2026-08-25)

`#222` claimed every run-file read goes through the guarded reader.
That claim is true as it was scoped,
and the scope was narrower than it sounds:
it covered parses that NO `try` encloses,
which are the ones V8's uncaught-exception reporter prints whole.

Re-reading the sweep found the other half.
V8's `SyntaxError` MESSAGE quotes the text as well,
and four handlers forwarded that message to somewhere it could be read.
Catching the error prevents the whole file being printed;
it does not prevent the ten characters the message carries.

### What was leaking, and to where

Three of the four read artifacts,
which hold corpus renderings:

-   `corpus-run/editor-standing-read.ts` parsed an artifact and printed
    `caughtValueText(error)` to stderr.
    `caughtValueText` returns `error.message` for an Error,
    which is the whole mechanism in one call.
-   `corpus-run/attribution-read.ts` parsed an artifact and STORED the same text
    as the `reason` on a `MalformedArtifact` record that travels to its caller.
    This one does not merely print,
    it persists.
-   `corpus-run/artifact-placement.ts` printed it on the `POOL malformed` line.

The fourth,
`corpus-run/runs-lock.ts`,
parses a lock file holding a pid and a timestamp,
so nothing corpus-bearing was ever exposed there.
It is fixed for one contract rather than four.

### Two things measured rather than assumed

V8 quotes only where the text stops being JSON near its start.
A file truncated at its tail yields a positional message that quotes nothing:

```text
"Pouncewick not json at all"  ->  Unexpected token 'P', "Pouncewick"... is not valid JSON
'{ "tail": "Pouncewick" '     ->  Expected ',' or '}' after property value in JSON at position 23
```

That matters for the test as much as for the defect:
a fixture that fails late would have tested nothing while appearing to test the guard.
The fixture word is ten characters,
exactly V8's quote window,
so it is neither padded nor truncated for a reason unrelated to the guard.

### What was correctly excluded, each checked rather than waved through

`attempt-store.ts` and `slice-cache-namespace.ts` catch and branch on the class,
printing no message.
Both are deliberately left unconverted:
each treats `error instanceof SyntaxError` as "half-written file,
recompute",
and routing them through the guarded reader would change the thrown class
so that branch silently stopped matching.
`verify-published.ts` reports `errorName`,
which is a class name.
`pass-schema-census.ts` has no parse inside its `try` and forwards only `ArtifactParseError`.
`runner-closure.ts` reads our own built bundle.

### The shape of the fix

`src/refusal-text.ts` holds the decision.
`refusalText({ error })` repeats a message only from a class that DECLARES its message
names rather than quotes,
and otherwise renders `refused by <class>`.
`RunJsonUnreadableError` and `LedgerShapeError` declare it;
`ledger-directory.ts`'s `refusalOf` now delegates rather than keeping its own copy of the list.

A declared field rather than a symbol,
for a measured reason:
`--isolatedDeclarations` rejects a computed property name on a class (TS9038),
and the `Error.isError` gate already refuses every plain object a run file could carry.
What a symbol would have added is refusing a forged marker on a real Error,
which is the same trust a symbol import gives anyway.

It fails closed.
An unmarked class,
a foreign Error and a thrown non-Error all take the naming branch,
so the only way to leak through here is to mark a class whose message quotes.

### State

Landed in `2e8dd62f3`,
tests in `3c53df242`.
Type-check clean,
lint clean at 0 warnings and 0 errors over 908 files.

NOT YET RUN,
and this is the honest gap:
the suite imports the built bundle,
and the calibration in flight owns `dist/final/node`,
so the new cases resolve only after a build.
Lint on the test file reports two `TS2305` errors on that import
plus 14 warnings cascading from them,
and nothing else.
Running the suite and proving both guards per GFP is the remaining work on `#224`.

`mise tasks deps` was used to establish that `lint:types`,
`lint:oxlint` and `test:unit`
carry no build dependency,
which is what made checking anything mid-pass possible.
The type-check writes `dist/final/types/` only,
never the node bundle the pass is running.

## A third shape, found by asking which other parsers quote (`#225`, 2026-08-25)

`#220` and `#224` were both about `JSON.parse`.
The question that found this one was smaller and better:
which OTHER parsers does this package hand corpus text to,
and what do their refusals say?

Two,
and they answered differently.

### YAML quotes, every way it was asked

The `yaml` package raises `YAMLParseError` carrying a source code frame:

```text
Nested mappings are not allowed in compact mappings at line 1, column 7:

name: Pouncewick
      ^
```

That is a whole line where V8 gives ten characters,
and five failure shapes were tried rather than one:
nested mapping,
duplicate key,
unclosed flow sequence,
tab indentation,
and bad block indent.
All five reproduced the frame.
Front matter holds a person's name,
their dates and their links.

WRAPPING IT DID NOT CONTAIN IT.
`FrontMatterParseError` carried the parser error as `cause`,
and Node's uncaught-exception reporter renders a cause chain,
so the frame printed under `[cause]:`.
Measured end to end by throwing a wrapped error from `node` and reading the terminal.

Reachable two ways:
`corpus-run/recall-benchmark.ts` calls `splitFrontMatter` and is not one of the nine CLIs `#223` wrapped,
and `parse-document.ts` runs inside the pass,
which writes to a terminal when run interactively.

### MDX was the near miss, and the first probe called it safe

Four of five MDX failure shapes report a position and an expectation and quote nothing.
On that evidence the module was written off as already safe.
Widening the probe found the fifth:

```text
1:1: Expected a closing tag for `<Pouncewick>` (1:1-1:13)
```

It quotes the tag name.
Narrow,
but `parse-document.ts` stringified that cause into a `ParseFinding.detail`,
which is STORED in the artifact rather than printed and forgotten,
and `mdx-downgraded` is a routine tolerance rather than a corruption path.

The lesson is the one `RXH` states:
the narrowest query returning nothing is not evidence of nothing.
One probe produced a clean negative that a fifth case refuted.

### What both errors say now

Position and fault code,
which is the whole of what a reader acts on:

```text
Front matter fence pair found but YAML inside refused to parse at line 1 column 7
(BLOCK_AS_IMPLICIT_KEY); corpus metadata parses upstream, so this signals corruption.

MDX body refused to parse at 1:1 (mdast-util-mdx-jsx/end-tag-mismatch); corpus documents
compile as MDX upstream, so failure signals corruption or an unsupported construct.
```

Neither carries the parser error as `cause` any more.
Nothing is lost that a reader acts on:
the file is on disk to open at that line.

### A recorded decision reversed, with its reasoning answered

`cli-refusal.ts` caught only `RunJsonUnreadableError`,
and its note argued for that:
catching every `Error` would destroy the stack of a genuine programming fault,
trading a rare ugly report for a permanently undiagnosable one.

That reasoning was right about the cost and wrong about the choice,
because the two are separable.
Everything is caught now,
and the frames are kept anyway.
What is dropped is the message line and the cause chain,
which is where text travels;
what is kept is the frames,
which name files inside our own `dist`.
An unexpected fault exits 5,
distinct from 4,
because one is a bug report and the other is a re-run.

The old forwarding test pinned the old contract,
so it was replaced rather than left to fail.
Three cases now hold the design from both sides:
the message must not appear anywhere the reporter writes,
and the frames must still be there.
Either assertion alone is satisfiable by a wrong implementation.

### State

Source landed in `7b81a95c3`,
tests in `9145cc8aa` and `1e862f9e5`.
Type-check clean,
and every source file lints clean.

NOT YET RUN.
Every new suite imports the built bundle,
and the calibration in flight owns `dist/final/node`.
Lint reports 20 findings across the three test files,
all four errors being `TS2305` on symbols the rebuild will supply.
Running them,
and proving each guard by removing it,
is what `#224` and `#225` still owe.

## Does any error class quote what it was handed? Scanned, not sampled (2026-08-25)

Three disclosure defects were found by hand,
one after another,
which is a bad way to learn a class is closed.
So the question was asked mechanically instead:
of every error class this package defines,
which builds a message that interpolates a value
that could be corpus text?

Seventy-five files define an `Error` subclass.
Scanning each class's `super(...)` call for interpolated identifiers whose names are text-shaped
(`text`,
`raw`,
`body`,
`content`,
`passage`,
`wording`,
`rendering`,
`answer`,
`source`,
`slice`,
`value`,
`excerpt`,
`snippet`,
`output`,
`reply`,
`message`)
returns three,
and reading all three settles them.

-   `WordingCoherenceError` takes a caller-supplied message and forwards it.
    All four call sites build `${at} reports ...`,
    where `at` is a slice locator,
    and none interpolates the wording.
    The scan flagged it on a neighbouring `@param wording`,
    not on the message.

-   `HardCapOverrideError` quotes what the hard-cap environment variable held.
    That is an operator's own input,
    not corpus text,
    and quoting it is the point:
    an operator who set a ceiling believes the run is bounded the way they asked.

-   `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
    carries 600 characters of a provider's HTTP error body.

### The third is a judgment, and it is being recorded rather than changed

A provider's error body is the provider's text,
not ours.
The one instance recorded in this handover is
`{"error":"You've exceeded your subscription rate limits. Upgrade, or try again later..."}`,
which echoes nothing.
A provider COULD echo part of a request in a 400,
and a request carries corpus wording,
so the path is not impossible.

It stays as it is,
for two reasons the owner has already stated.
Provider issues are normal and expected and the pipeline must stay diagnosable through them,
and the standing instruction on logging is to add more where it is thin,
not less.
An excerpt bounded at 600 characters is the diagnostic that makes a provider fault legible.

Recorded here so it reads as a decision rather than an oversight,
which is the same treatment the `raw=` warn logs already have.

### What this closes

The three defects found by hand (`#220`,
`#224`,
`#225`) all came from a parser's own message
or from a cause chain,
never from a class this package wrote.
That is now a measured statement about all seventy-five rather than an impression from three.

## The cause sweep, which the message scan had missed (2026-08-25)

Scanning seventy-five error classes checked their `super()` MESSAGES.
It said nothing about their causes,
and a cause chain is exactly how `#225` travelled:
a clean message carrying a quoting cause that Node's reporter renders anyway.

So the same question was asked of causes.
Every site constructing an error with `{ cause: error }`,
and what that cause can hold:

-   TWO PARSER CAUSES,
    `FrontMatterParseError` and `MdxParseError`.
    Fixed in `7b81a95c3`;
    neither carries a cause any more.

-   THREE PROVIDER-PARSE CAUSES:
    `completion-shape.ts`,
    `stream-completion.ts` and
    `anthropic-completion.ts` each wrap a `JSON.parse` of a model's response as the cause of a
    `MalformedCompletionError`.
    V8 quotes ten characters,
    and a model's answer is a rendering of corpus text.

-   TWO FILESYSTEM AND GIT CAUSES,
    in `ledger-directory.ts` and `artifact-generation.ts`.
    One wraps `readdir`,
    the other a git revision resolution.
    Their messages name a path and a revision,
    not content.

-   ONE COMMAND CAUSE in `corpus-source.ts`,
    whose message carries the `git show` command line.
    That names an entry id,
    which these tools print by design:
    `ledger-report` and `verify-published` both report per entry.

-   TWO ABORT CAUSES in `stream-cut.ts` and `stream-drain.ts`,
    carrying an abort reason.

### The provider three are dominated, and that is measurable rather than arguable

`stage-call.ts` already logs `raw=${JSON.stringify(opening,)}` on every lost-voice warning,
bounded at `RAW_PREVIEW_CHARS = 120` grapheme clusters of the model's text.
Its own note records why that bound exists and what it bought:
the Kimi-K3 outage was a two-character channel marker,
507 mismatches in one pass were explained by it,
and its 2026-08-13 recurrence was explained the same way from `p|>` and `ep|>`.

So the deliberate disclosure is 120 characters,
on a routine warning,
to the run log.
The cause is ten characters of the same text,
only when the answer is malformed JSON,
and only if the error reaches a printer.
Twelve times smaller,
far rarer,
same destination.

They stay.
Changing them would trade a diagnostic the owner asked for
against an exposure strictly smaller than one already accepted beside it.

### What this sweep does and does not cover

It covers two channels:
an error's own message,
and its cause chain.
It does not by itself cover a third,
a value written into a persisted record rather than thrown,
which is how `attribution-read` was leaking in `#224`.
That one was found by reading call sites,
and no mechanical scan has been built for it.
Naming the gap here so a later session knows which of the three has no scanner.

## The guard proof found the guard was somewhere else (`#224`, 2026-08-25)

`#224` routed four sinks through `refusalText` and owed a proof that the routing mattered.
The proof was run the way the rule prescribes:
revert the fix,
rebuild,
run the suite,
see it fail.
It did not fail.
686 of 686 passed with all four sinks reverted to a bare `caughtValueText(error)`.

A passing revert reads as missing coverage,
and this session nearly recorded it as one.
The reason it is not is that the probe could not have failed:
a MALFORMED file reaches every sink through `parseRunJson`,
which already wraps it in a class declaring its message quote-free,
so `refusalText` returns that message and the bare expression returns the same message.
The two are the same string.
A null result from a probe that cannot show a difference means nothing,
and the control that gives it meaning is a file that will not OPEN.

### Two of the four opened without a guard

Measured on a mode-`000` fixture in a throwaway directory:

```text
POOL malformed Basket.json: EACCES: permission denied, open '/tmp/sink-probe-OXYrhl/Basket.json'
```

`attribution-read.ts` and `artifact-placement.ts` called `readFile` directly
and only PARSED through the guard,
so a file that would not open arrived as an ordinary `Error`
whose message quotes the whole path.
`refusalText` refused to repeat it,
which is the second layer doing its job,
but the only thing left to say was `refused by Error`,
and an operator reading that learns neither that the file exists nor that it is a permission fault.

`#222` recorded that all twelve run-file reads went through the guarded reader.
That was true of the parse and not of the open.
A run directory path names the run,
and under `artifacts/` a file's own stem is a person's entry id,
so the path is the sensitive half.

Landed in `f8a00627f`:
both call `readRunJson`,
which opens and parses behind one refusal:

```text
could not read Basket.json as JSON (EACCES)
```

`runs-lock.ts` and `editor-standing-read.ts` already said exactly that,
so all four sinks now speak one vocabulary.

### The suite that pins it, and the two-layer proof

`src/corpus-run/sink-names-only.unit.test.ts` holds five cases,
one per sink plus the case that reads the path back.
It pairs with `message-names-only.unit.test.ts`:
that one decides which CLASSES may repeat a message,
this one checks that the SINKS actually ask.

`editor-standing-read.ts` exports nothing,
so its case runs the built command the way an operator does and reads its stderr.
That is the first spawn in this package's suite.
The command exits 1 on the fixture for its own `#217` reason,
having recorded no rounds,
so the helper reads both streams off a `spawnSync` status
rather than treating a non-zero exit as a failure to run.

Every fixture proves itself unreadable before any assertion runs.
A suite run as root opens a mode-`000` file,
which would leave all five cases exercising the happy path while reporting a pass.

The two layers are proven separately,
because they fail in different worlds:

-   guarded open removed,
    policy call kept:
    2 of 5 fail,
    and the path-leak case PASSES,
    because `refusalText` is what keeps the path out there.
-   both removed:
    3 of 5 fail,
    the path-leak case included.
-   restored:
    5 of 5 pass,
    whole suite 687 of 687.
