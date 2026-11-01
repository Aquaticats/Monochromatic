# Translation repair history 2026-08-24 to 2026-08-25: reporting refusals at entry points, which error messages may be repeated, and the guard proofs run once the bundle was free

The wrapping of every entry point decided by measuring four cells,
the rule deciding which error classes may repeat their message and the guard that enforces it,
and the four guard proofs run once the bundle was free,
with the check that a rebuild cannot reach a live calibration.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Every entry point now reports its refusals, settled by measurement (`#226`, 2026-08-25)

`#223` wrapped nine CLIs when `reportingRefusals` caught exactly one class.
It now catches everything,
so the list should be every entry point,
and now is:
29 more were wrapped,
in `f92df6042`.

This section keeps the argument that held the change back for a day,
because the argument was sound and the thing that resolved it was evidence,
not a better argument.

`refusalText` forwards a message only from a class that declares it quote-free,
and four classes declare it:
`RunJsonUnreadableError`,
`LedgerShapeError`,
`FrontMatterParseError`,
`MdxParseError`.
Wrapping a CLI therefore turns
`ArtifactParseError: <what it says>` into `refused by ArtifactParseError` plus frames.
For a report CLI that is a small loss.
For `corpus-pass`,
the production driver,
it is the diagnostic that matters most.

The scan of all seventy-five error classes cuts both ways here.
It says none of them quotes,
so forwarding their messages would be safe today,
which makes the conservative default look like pure cost.
It also says nothing about the seventy-sixth,
and a library added later reintroduces the risk silently,
which is the whole reason the default fails closed.

Three ways out,
and they are not equally good:

-   Mark all seventy-five classes.
    Restores every message,
    and makes the marker mean
    "audited" rather than "constructed safe",
    which is a weaker claim than the marker
    currently makes.
-   Wrap only the report and probe CLIs,
    and leave `corpus-pass` alone with its reason
    recorded,
    the way `ledger-report` is already excluded for a reason of its own.
-   Wrap everything and accept that an unexpected fault is located by frames rather than
    described by a message.

### The premise that it needed the suite was wrong

The sentence this section used to end on said deciding without the suite
would be guessing at what the output reads like.
That was a refusal with an unconsidered bridge behind it.
`node --experimental-strip-types` runs the package source directly,
so the output could be read at any point without building anything,
and the whole decision took one throwaway fixture under `~/temp/agent`.

The same bridge answered a question the section never thought to ask,
and answered it against the section's own numbers.

### What the census actually was

Twenty-five was wrong.
There are 38 entry points,
9 of them wrapped.
The count was low because the search looked for `if (import.meta.main)`,
and 13 entry points did not have it:
they ended in a bare top-level `await main();`,
which runs on import.
Nothing value-imports any of them today,
and the one import that exists takes only a type from `roster-bench`,
which erases,
so this was a latent hazard rather than a live defect.
They now carry the guard the other 16 had.

### The four cells

Each cell is one process,
source run on a throwaway fixture,
stderr measured in bytes:

```text
bare, marked error        exit 1, 708 bytes   stack dump, plus Node spilling the
                                              error's own fields as an object literal
wrapped, marked error     exit 4, 193 bytes   the message, and what to do next
bare, unmarked error      exit 1, 554 bytes   class, message and stack
wrapped, unmarked error   exit 5, 662 bytes   class and frames, message dropped
```

Read against the three options,
this picks the third and weakens the case for the first.

Wrapping is an outright win on the refusal path:
708 bytes of stack and spilled fields become 193 bytes that say what happened.
On the fault path it costs the message and buys a code.
That code is the part the options list undervalued:
unwrapped,
a refusal,
a fault and the command's own verdict are all `1`,
and no gate or operator can tell them apart.

Marking all seventy-five classes stays available and stays the lever
that gets the message back on the paths where it matters.
It is now an improvement to make on top of a working separation,
rather than a precondition for making any change at all.

### What was measured before the change rather than after

That a wrapped CLI still leaves its own verdict alone.
`reportingRefusals` sets `process.exitCode` only inside its catch,
which the source says and a probe confirmed:
`verify-published` on an unreadable run directory exits `2` either way,
with byte-identical output.
Reading that off the source would have been an inference about unchanged code.

Type-check and lint carry the same 20 findings after the change as before,
all of them in the three test files waiting on a rebuild,
none in the 29 files touched.

## Which error messages may be repeated, decided by a rule rather than an audit (`#227`, 2026-08-25)

`refusalText` repeats a message only from a class declaring `messageNamesOnly`,
and four of
eighty-five classes declared it.
So an operator meeting a domain refusal at a CLI boundary
read `refused by RosterConfigurationError` and learned nothing about what was wrong.

The obvious move,
marking all eighty-five,
is the one to refuse.
A 2026-08-25 scan did find that none of them quotes what it was handed,
but marking on that basis makes the marker mean "audited on a Tuesday"
rather than "constructed to name and never quote",
which is a weaker claim than the marker
already makes,
and it decays silently.

### The rule

A class may declare the marker when the CLASS writes the sentence,
and every value it interpolates is a number,
one of our own names,
or a value the operator supplied.

A class whose constructor forwards a `message` parameter to `super` may not,
however careful its throw sites are.
The claim would then be about thirty call sites rather than about one class,
and nothing could check it.
`StatedRefusalError` is the deliberate exception and carries its own note saying so.

That line is what makes the rest mechanical.
Thirty-four classes forward a message.
Fifty-one write their own sentence.

### What the scan found, reading every site rather than sampling

Across 490 construction sites outside the suites,
the free-text fields turned out to be
authored phrases almost everywhere:

-   `ArtifactParseError` has 93 sites,
    and all 77 distinct `reason` values are shape
    descriptions written by us,
    `a boolean`,
    `a record`,
    `distinct members`.
-   `MalformedCompletionError`,
    `QuotaShapeError` and `CreditsShapeError` take a `detail`
    that is an authored phrase at every site,
    naming which part of the protocol broke.
    None carries a byte of the body.
-   `TranslateAbsenceError` is the best case:
    its `reason` is typed `TranslateAbsenceReason`,
    a closed union,
    so the type proves what the inventory would otherwise assert.

Forty-two classes gained the marker,
joining the five that had it.

### Four that write their own sentence and still stay unmarked

Recorded rather than left silent,
because an absent marker looks identical to an oversight:

-   `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
    interpolates an excerpt of the provider response body,
    on purpose.
-   `MislabelledArtifactError` takes `caughtValueText(error)`,
    which renders a caught error's
    own message.
-   `StreamCutShortError` reaches the message through `String` of an unknown abort reason.
-   `ArtifactProvenanceError` interpolates `expected` and `observed`,
    whichever field
    disagreed,
    which may be text.

### The guard, and why it is a source scan

`src/message-names-only.unit.test.ts` reads the source and checks four things:
the marked set equals the recorded list,
every interpolation in a marked message is named in
an inventory that says what it holds,
no forwarding class carries the marker,
and every
owning-but-unmarked class has a recorded reason.

A behavioural test would have pinned today's wording,
which is not the property worth
guarding:
rewording a sentence is fine,
and interpolating a new value into it is the thing
that needs a second look.

GFP-PROVEN,
four probes,
each restored afterwards:

```text
  marked but not recorded    KEEPS exactly the classes ... FAIL   exit 1
  message gains a new part   ACCEPTS only the message parts ... FAIL   exit 1
  owning class unmarked      KEEPS exactly ... + KEEPS a reason ... FAIL   exit 1
  withheld class marked      three of four FAIL                        exit 1
```

The second probe fails that assertion and no other,
which is the isolation worth having.

WHAT IT CANNOT CATCH.
A field the inventory already names,
such as `detail`,
can be handed
different text by a new throw site.
The inventory records what each field holds today,
and
that kind of change is caught by reading,
not by this file.
The stronger form would put the rule in the type system,
with the message built through a
tagged template whose interpolated values are a closed union rather than `string`.
That is a large refactor across the domain primitives and is not proposed here as part of
this landing;
it is written down as the direction if the inventory ever starts to strain.

The guard imports no built artifact,
so it runs from source while a pass holds the bundle.
Landed in `f0f15f5c3`.

## Four guard proofs, run once the bundle was free (`#225`, `#228`, 2026-08-25)

The suites for `#224`,
`#225`,
`#226`,
`#227` and `#228` all import the built bundle,
and the bundle had been held by a live calibration since the day before.
Rebuilding released all five at once.

`#228`,
both providers.
Replacing `verify: wholeMessage` with a comment fails exactly two cases per provider:
`REFUSES a stream that ended without its terminator` reports `expected 1 to equal 3`,
one attempt spent instead of three,
and `ACCEPTS the retry when only the first attempt was cut short`
lets a `MalformedCompletionError` escape.
Four failures across the two suites,
no collateral.

`#225`,
parser half.
Reverting `front-matter.ts` and `parse-mdx.ts` fails three cases in each suite:
the position case,
the no-cause case,
and the marker case.
The no-quoting case passes in both worlds,
which is right rather than a gap:
the pre-fix classes already wrapped,
and the leak travelled through the CAUSE chain
that Node's reporter renders whether asked to or not.

`#225`,
CLI half.
Reverting `cli-refusal.ts` fails five of its seven cases,
including the two added this session for `#226`'s stated-refusal branch
and `#227`'s marked-class fault path,
so both of those are boundary-proven here too.
The survivors are the `RunJsonUnreadableError` report,
which the old code already caught,
and the clean run.

### Rebuilding under a live pass, measured rather than argued

`doc/runbook/translation-repair-corpus-pass.md` says to run nothing through `mise`
while a pass is in flight,
and gives the pipeline digest as the reason.
That reason is sound and it is narrower than the rule.

The pass in flight was `editor-calibrate`,
which writes no artifacts and stamps no digest:
its output is its log.
Before rebuilding,
three facts were checked on the running process rather than assumed:

```text
node dist/final/node/editor-calibrate.mjs 40    one process, zero children
/proc/<pid>/fd                                  no descriptor under translation-repair/dist
grep -o "import(" dist/.../editor-calibrate.mjs 0
```

A running pass never reads that directory again,
so overwriting it cannot reach the process.
The bundle was rebuilt eight times over the session,
twice with deliberately broken source for a guard proof,
and the calibration kept producing rounds throughout.

This does not license rebuilding under a CORPUS pass.
That one stamps a digest into every artifact it settles,
and a rebuild leaves the run recording a digest that no longer describes what is on disk.
The distinction is what the run WRITES,
not whether the process would notice.
