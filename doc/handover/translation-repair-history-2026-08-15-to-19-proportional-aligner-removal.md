# Translation repair history 2026-08-15 to 2026-08-19: removing the proportional aligner

The deletion of the proportional aligner and the contracts that kept describing it,
and the correction that the forced aligner is not blind to cross-language headings.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The proportional aligner removed

### The proportional aligner is gone, and so are the contracts it left

`3d34b72e2`,
which is `#98`'s two side findings.
`alignProportionally` merged
mismatched sections by cumulative character fraction;
it is what slid `XingZ60`
by two sections and made every issue filed on that entry noise.
The forced
aligner replaced it and it has been unreachable since.
272 lines,
plus
`totalChunkChars` and `mergeChunkRun`,
plus the `sections-merged` finding kind
that only it emitted.
Artifacts settled before the change still carry that
string as prose,
and no reader in the package matches on it.

WHAT THE DELETION EXPOSED is more interesting than the deletion:
seven separate
statements in that file described the behaviour it used to have.
Alignment
called itself total while a refused section lands in no pair on either side.
A
pair claimed either side might span several merged sections.
The fast path was
called `mirrored` while testing only counts and leading node kinds,
which for a
document of ordinary heading sections is just the counts.
`heading-affinity.ts`
still called itself an unwired prototype though the grid scorer has called it
since the forced aligner landed.
Dead code does not sit quietly;
it keeps its
documentation alive around it,
and the surrounding contracts go on describing a
system that no longer exists.

THE HEADLINE DEFECT IN `#98` IS STILL OPEN,
deliberately,
and one correction is
worth carrying:
I had written that the aligner is blind to cross-language
headings.
It is not.
`headingAffinity` scores shared Latin runs precisely so a
handle carried across pairs its sections.
What it cannot score is two headings
with no shared Latin at all,
which is the common case here.
So running the
aligner over the 85 fast-path entries would change no pairing,
and doing it
would look like a fix while altering nothing.
