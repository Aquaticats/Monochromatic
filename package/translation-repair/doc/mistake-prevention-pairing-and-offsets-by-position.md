# Preventing mistakes: pairing and offsets by position

Rules for pairing items of two texts by their place and for offsets that are compared or carried between texts.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
The other rules for reading text and structure are in
[comparing text and reading structure](mistake-prevention-text-comparison-and-structure.md).

## Pairing by position

What happened:
three page-assembly passes paired page headings with the original's or the archive's by position.
The heading collision restore checked only that the page's total matched,
so a slice that dropped a heading and another that gained one shifted every heading between them,
and a section the page never replaced shipped as empty text (ledger B83).
The heading series pass and the contributor name pass paired inside one slice with no count check at all,
so a page slice that dropped its first heading gave the next section the wrong number,
or wrote a signer's name over its title (ledger B84).
The contributor name pass already checked the count for signatures,
one line away.

A replacement for a slice the page lacks was taken with the archive read as empty text (ledger B273).
A row that names a slice is read through the reader that refuses a slice the page does not hold.

The rule:
pair two lists by position only where they carry the same count
at the level the positions are read:
per slice where positions are read per slice,
and per slice as well where a page-wide position runs through slices,
since equal totals can hide a shift.
Where the counts differ,
leave the items as they stand and let the guard downstream speak;
never fall back to empty text or a neighbour's item.
A pass reads per-slice page text through `pageTextOf` (`assembly-page-text.ts`),
which refuses a slice its map was not built from.

What enforces it:
each pass's tests hold a case whose page drops or gains one item inside a slice
(`heading-collision-restore`,
`heading-series-unify`,
`contributor-name-restore`),
and `assembly-page-text.unit.test.ts` holds the refusal.
No scan finds a new positional pairing;
a search for position-indexed reads over `corpus-run/` found the others,
each already checking its count.

## Offsets and positions

What happened:
the assembly guard's advancing withdrawal compared where the strict grammar stopped on the page as assembled
with where it stopped on the page with one replacement withdrawn,
as raw offsets into two different texts,
so a whole replacement shorter than its archive text read as moving a break it never touched,
and outranked the withdrawal that removed it (ledger B85).
The reader of those offsets placed a refusal at the start of the span the parser named,
though the parser raises it on leaving that span,
and placed a refusal naming no span at the start of the body,
though the parser had read to the body's end;
a page whose first named break was withdrawn then read as stopping at offset 0,
and the guard gave up every replacement (ledger B86).

The parser skips a byte order mark that opens its input without counting it,
so every position of the tree sat one character short of the body as written (ledger B257).
A parser that drops input without counting it moves every offset it reports,
and the shift is undone where the offsets are made,
not in each reader.

The rule:
an offset means something only in the text it indexes.
Compare two only after mapping one into the other's frame
(`assembledOffsetOf` in `assembly-structural-withdrawal.ts` for two pages that differ in one slice),
and keep "the parser accepted the page" as its own outcome rather than a number that can tie with a refusal.
A parser's position is where its message says it stopped:
read the deciding source for which end of a span that is,
and for what a message with no place means,
before turning either into an offset.

What enforces it:
`assembly-structural-withdrawal.unit.test.ts` holds a page whose earlier replacement only changes length
and a page whose remaining refusal names no place;
`strict-refusal-offset.unit.test.ts` holds an element left open in a paragraph and one left open at the document's end.
No scan finds offsets compared across texts.
