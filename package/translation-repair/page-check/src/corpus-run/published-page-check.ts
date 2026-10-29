import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { codePointLength, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ParsedArchiveText, } from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  type WouldShipSlice,
  type WouldShipSource,
  wouldShipTextPerSlice,
} from './would-ship-text.ts';
import { matchSpanEdges, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { fragmentBody, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { PublishedPageDisagreesError, } from './published-page-disagreement.ts';
import {
  carriesText,
  textOnPage,
} from './source-ships.ts';

//region Published page check
// Reads the DELIVERABLE back, which until now nothing did.
//
// The mirrored tree of fixed `*.en.md` pages is the thing this pipeline
// produces, and every check built since reads artifacts instead. An artifact is
// what the deciders said; a page is what a reader gets. What the gap
// between them costs is known: the publisher turned a named absence back into the empty
// string, and the only thing that noticed was a guard inside the splice, after
// four hours and forty-eight minutes of calls.
//
// NOTHING HERE PRINTS A PASSAGE. Every finding names a slice by index and a size
// by character count, because these reports are read beside a corpus whose text
// may not leave the run directory.
//
// A CHARACTER IS A CODE POINT (`codePointLength`), which is what the findings call
// it. Until 2026-10-06 every size here was a UTF-16 length, so a character beyond
// the first plane (a rare CJK character, an emoji) counted as two. Only the
// scan's cursor stays in UTF-16 units, since it is an index into the page.
//
// WHAT THIS CANNOT CHECK, stated so nobody reads more into a clean report than
// it carries. The exact check is to splice the artifact's readings over its
// archive text and compare byte for byte, which is what `publishFixedPage` does.
// That needs the slice SPANS, and `SettledPreparation` records `sliceCount`
// and an identity hash rather than offsets. Two necessary conditions stand in:
//
//   ORDER. Every wording the artifact says would ship occurs in the page, in
//   slice order, without overlapping.
//
//   LENGTH. The page is exactly as long as the archive plus what every slice
//   added and minus what it removed.
//
// NEITHER IS ENOUGH ALONE, which was measured rather than reasoned. Cutting two
// hundred characters out of the middle of a real published page left every
// wording in place and in order, because the scan only covers the slices and a
// page is mostly the text between them. The length invariant catches that cut
// and an order-only check does not; an order check catches wording swapped
// between two slices of equal size and a length check does not.

/**
 What `indexOf` returns for a wording the page does not carry.
 */
const NOT_IN_PAGE = -1;

/**
 One wording the page was supposed to carry and does not.

 SIZED RATHER THAN QUOTED, so a report can be read anywhere. The slice index
 and the length are enough to find it in the artifact, which is where the text
 legitimately lives.

 @example
 ```ts
 const missing: MissingWording = { sliceIndex: 12, characters: 344, };
 ```
 */
export type MissingWording = {
  /**
   Slice whose wording could not be found at or after the cursor.
   */
  readonly sliceIndex: number;

  /**
   How long that wording is, in characters (code points).
   */
  readonly characters: number;
};

/**
 What one page turned out to carry of what its artifact promised.

 @example
 ```ts
 const check: PageWordingCheck = { wordings: 9, silentSlices: 1, sourceShipped: 2, missing: [], };
 ```
 */
export type PageWordingCheck = {
  /**
   Slices whose reading carries English wording, so the page must contain it.
   */
  readonly wordings: number;

  /**
   Slices whose reading carries the original text of an untranslated passage, so the page must contain it
   too (the owner, 2026-10-23, "Source text + note"). Counted apart from the wordings because they are not
   English, and apart from the silent slices because the page does carry them.
   */
  readonly sourceShipped: number;

  /**
   Slices where nothing ships, which the page must simply not invent. A passage the page carries as its
   original text is not among them.

   COUNTED RATHER THAN CHECKED. An anchor nobody filled leaves the page exactly
   as the archive had it, and a content span the deciders emptied leaves a hole
   where its wording was. Telling those two apart in the page needs the spans
   this file does not have, so the count is reported for a reader to weigh
   against the run's own `pageSilent`.
   */
  readonly silentSlices: number;

  /**
   Wordings the page does not carry in order, empty on a page that agrees.
   */
  readonly missing: readonly MissingWording[];
};

/**
 What a page is expected to weigh and what it weighs, or that it cannot be
 weighed at all.

 A TAGGED ABSENCE RATHER THAN A CLEAN RESULT, for the reason
 `ParsedArchiveText` gives for being one itself. An artifact written before
 the archive text was stored gives this check nothing to subtract from, and
 reporting that as agreement would let a whole run of old artifacts read as
 verified. `unweighable` says the arithmetic never ran.

 @example
 ```ts
 const weighed: PageLengthCheck = { kind: 'weighed', expected: 3840, actual: 3840, exact: true, };
 ```
 */
export type PageLengthCheck = {
  /**
   Says this artifact predates the stored archive text.
   */
  readonly kind: 'unweighable';
} | {
  /**
   Says the arithmetic ran and these are its two sides.
   */
  readonly kind: 'weighed';

  /**
   Characters (code points) the archive plus every slice change comes to.
   */
  readonly expected: number;

  /**
   Characters (code points) the page on disk actually has.
   */
  readonly actual: number;

  /**
   Whether `expected` is an equality or a floor.

   A FILLED ANCHOR MAKES IT A FLOOR. `spliceSlices` composes the separators
   around an inserted rendering rather than carrying them in any row, which
   `delivery-invariants.ts` states in the same words: a concatenation of row
   texts differs from the document while nothing is wrong. Those separators
   are real characters nobody counted, so a page with one is longer than this
   arithmetic predicts and shorter than it is a fault either way.
   */
  readonly exact: boolean;
};

/**
 Net characters one slice adds to the archive, negative where it removes.

 @param slice - slice and what it would carry

 @param incumbentBySlice - archive wording per slice index

 @returns Characters this slice adds, negative where it removes

 @example
 ```ts
 const delta = sliceDelta({ slice, incumbentBySlice, },);
 ```
 */
function sliceDelta(
  {
    slice,
    incumbentBySlice,
  }: {
    readonly slice: WouldShipSlice;
    readonly incumbentBySlice: ReadonlyMap<number, string>;
  },
): number {
  /**
   Characters the archive held there, none at an anchor.
   */
  const held = codePointLength({
    text: nonNullishOrThrow(incumbentBySlice.get(slice.sliceIndex,),),
  },);

  /**
   What this slice would carry, or that it carries nothing.
   */
  const { reading, } = slice;

  if (!carriesText(reading,))
    return -held;

  /**
   Characters this slice publishes in its place: the text as the splice writes
   it, with the archive span's own line-ending edges rather than the lane's
   (class seventy-five), so a lane's trailing newline is not counted as a
   character the page then lost.
   */
  const ships = codePointLength({
    text: matchSpanEdges({
      replaced: nonNullishOrThrow(incumbentBySlice.get(slice.sliceIndex,),),
      text: textOnPage({ reading, },),
    },),
  },);

  return ships - held;
}

/**
 Whether a slice filled an anchor, which puts separators in the page that no
 row counted.

 @param slice - slice and what it would carry

 @param incumbentBySlice - archive wording per slice index

 @returns Whether wording was inserted where the archive had nothing

 @example
 ```ts
 const inserted = filledAnAnchor({ slice, incumbentBySlice, },);
 ```
 */
function filledAnAnchor(
  {
    slice,
    incumbentBySlice,
  }: {
    readonly slice: WouldShipSlice;
    readonly incumbentBySlice: ReadonlyMap<number, string>;
  },
): boolean {
  /**
   What this slice would carry, or that it carries nothing.
   */
  const { reading, } = slice;

  if (!carriesText(reading,))
    return false;

  return nonNullishOrThrow(incumbentBySlice.get(slice.sliceIndex,),) === '';
}

/**
 Weighs a page against its archive and the changes every slice made to it.

 ARITHMETIC RATHER THAN SEARCH, which is what makes this cover the text no
 slice names. Splicing replaces each span with its replacement, so the
 document grows by exactly what each slice added and shrinks by what it
 removed, and everything between the spans is carried through untouched. A
 page that lost a paragraph nobody decided on has the wrong length and every
 per-slice check still passes.

 @param artifact - settled entry, read for what each slice would carry

 @param archive - archive English this entry was published over, or the
 statement that this artifact never recorded it

 @param pageText - published page as it sits on disk

 @returns Expected length, actual length and whether the two must be equal,
 or that the artifact gave the arithmetic nothing to start from

 @example
 ```ts
 const weighed = pageWeighsWhatItShould({ artifact, archive, pageText, },);
 ```
 */
export function pageWeighsWhatItShould(
  {
    artifact,
    archive,
    pageText,
  }: {
    readonly artifact: WouldShipSource;
    readonly archive: ParsedArchiveText;
    readonly pageText: string;
  },
): PageLengthCheck {
  if (archive.kind !== 'stored')
    return { kind: 'unweighable', };

  /**
   What each slice would contribute, beside the row it came from.
   */
  const slices = wouldShipTextPerSlice({ artifact, },);

  /**
   Archive wording each slice covers, by index, so the sum this function takes reads both
   sides of one slice without walking the comparison again.
   */
  const incumbentBySlice = new Map(artifact.comparison
    .map(function pair(row,): readonly [
      number,
      string,
    ] {
      return [
        row.sliceIndex,
        row.incumbentText,
      ];
    },),);

  /**
   Net characters every slice adds to the archive, negative where it removes.
   */
  const net = slices.reduce(
    function addSlice(
      sum: number,
      slice,
    ): number {
      return sum + sliceDelta({
        slice,
        incumbentBySlice,
      },);
    },
    0,
  );

  /**
   Whether any anchor was filled, which puts uncounted separators in the page.
   */
  const inserted = slices.some(function filled(slice,): boolean {
    return filledAnAnchor({
      slice,
      incumbentBySlice,
    },);
  },);

  /**
   Characters the archive itself came to.
   */
  const archiveChars = codePointLength({ text: archive.text, },);

  return {
    kind: 'weighed',
    expected: archiveChars + net,
    actual: codePointLength({ text: pageText, },),
    exact: !inserted,
  };
}

/**
 Reads a weighing for whether it refutes the page.

 ONE-SIDED WHERE THE ARITHMETIC IS, which is the whole reason this is a
 function rather than an equality at each call site. An exact weighing must
 match; an inexact one only sets a floor, because the separators a filled
 anchor composes are real characters this sum never counted. A page shorter
 than its floor lost text either way, and that is the direction that matters.

 @param weight - what `pageWeighsWhatItShould` returned

 @returns Whether the length refutes the page

 @example
 ```ts
 const wrong = pageWeightRefutes({ weight, },);
 ```
 */
export function pageWeightRefutes(
  { weight, }: { readonly weight: PageLengthCheck; },
): boolean {
  if (weight.kind === 'unweighable')
    return false;
  if (weight.exact)
    return weight.actual !== weight.expected;
  return weight.actual < weight.expected;
}

/**
 One form of a slice's wording and where the page carries it.

 @example
 ```ts
 const found: FormOnPage = { at: 120, form: '她说', };
 ```
 */
type FormOnPage = {
  /**
   Index in the page where the form starts at or after the cursor.
   */
  readonly at: number;

  /**
   The form searched for.
   */
  readonly form: string;
};

/**
 Checks that a page carries every wording its artifact says would ship, in
 slice order.

 A CURSOR RATHER THAN A SET OF SUBSTRING TESTS, because order is half the
 claim. Slices are contiguous and ordered in the document, so their wordings
 appear in the page in `sliceIndex` order and do not overlap. Searching from
 where the previous one ended enforces both, and a page that carried every
 wording in the wrong order would pass a set test and fail this one.

 NECESSARY RATHER THAN SUFFICIENT, and deliberately one-sided. A wording that
 also occurs somewhere it does not belong still satisfies this, so a clean
 result is not proof the page is right. It cannot report a correct page as
 wrong, which is what a standing check has to guarantee before anyone trusts a
 red result enough to stop a release.

 @param artifact - settled entry, read for what each slice would carry

 @param pageText - published page as it sits on disk

 @returns Counts, and every wording the page does not carry in order

 @example
 ```ts
 const check = pageCarriesEveryWording({ artifact, pageText, },);
 ```
 */
export function pageCarriesEveryWording(
  {
    artifact,
    pageText,
  }: {
    readonly artifact: WouldShipSource;
    readonly pageText: string;
  },
): PageWordingCheck {
  /**
   What each slice would contribute, in the artifact's own comparison order.
   */
  const slices = wouldShipTextPerSlice({ artifact, },);

  /**
   Slices contributing no text at all, which the scan walks past.
   */
  const silentSlices = slices
    .filter(function saysNothing(slice,): boolean {
      return slice.reading
        .kind
        === 'nothing-ships';
    },)
    .length;

  /**
   Slices carrying the original of a passage nobody translated, which the scan looks for beside the wordings.
   */
  const sourceShipped = slices
    .filter(function carriesOriginal(slice,): boolean {
      return slice.reading
        .kind
        === 'source-ships';
    },)
    .length;

  // ONE PASS THAT APPENDS, since copying the wordings found missing so far at
  // every slice costs the square of the slice count (ledger B70).
  /**
   Wordings the page did not carry at or after the cursor.
   */
  const missing: MissingWording[] = [];
  /**
   Where the previous wording ended, so the next is searched for after it.
   */
  const cursor = { at: 0, };
  for (const slice of slices) {
    /**
     What this slice would carry, or that it carries nothing.
     */
    const { reading, } = slice;
    if (!carriesText(reading,))
      continue;

    /**
     Wording this slice contributes AS THE SPLICE WRITES IT, not as the
     stage recorded it. An inserted slice goes through the insertion
     composer, which cuts the blank lines around a fragment and the spaces
     at its end (class one hundred one: a footnote definition whose lane
     text ended in two spaces was reported missing from a page that carried
     every word of it); a content span keeps its interior and takes the
     archive span's line-ending edges. The body is inside both, so the scan
     asks for what either path leaves on the page and stays one-sided.
     */
    const wording = fragmentBody({ fragment: textOnPage({ reading, },), },);

    /**
     What the page may carry for this slice instead of that wording: an original written as literal text, which the
     publisher does where the page with the original as recorded does not parse (`source-ships-compose.ts`). It is the
     same words with a backslash before a brace or a less-than sign, and the scan stays one-sided by accepting either.
     */
    const alternatives = (reading.kind === 'source-ships')
      ? [fragmentBody({
        fragment: textOnPage({
          reading,
          encoding: 'literal',
        },),
      },),]
      : [];

    /**
     Where each form of this wording sits at or after the cursor, the ones the page does not carry left out.
     */
    const found = [
      wording,
      ...alternatives,
    ]
      .map(function foundAt(form,): FormOnPage {
        return {
          at: pageText.indexOf(
            form,
            cursor.at,
          ),
          form,
        };
      },)
      .filter(function carried({ at, },): boolean {
        return at !== NOT_IN_PAGE;
      },)
      .toSorted(function byPosition(
        left,
        right,
      ): number {
        return left.at - right.at;
      },);

    /**
     The form the page carries first, or that it carries none here.
     */
    const [first,] = found;

    if (first === undefined) {
      missing.push({
        sliceIndex: slice.sliceIndex,
        characters: codePointLength({ text: wording, },),
      },);
      continue;
    }

    // PAST THIS WORDING RATHER THAN PAST ITS START, so the next slice cannot
    // match inside it. Two adjacent slices whose wordings share a suffix and
    // a prefix would otherwise both find the same stretch of page. An index
    // into the page, so in UTF-16 units, as `indexOf` counts.
    cursor.at = first.at
      + first.form
        .length;
  }

  /**
   Slices carrying English wording: those that carry any text, less the ones carrying the original.
   */
  const withText = slices.length - silentSlices;
  return {
    wordings: withText - sourceShipped,
    sourceShipped,
    silentSlices,
    missing,
  };
}

/**
 Refuses a page that does not carry what its artifact says would ship.

 CALLED BEFORE THE PAGE IS WRITTEN, which is what makes the refusal cheap and
 truthful. `pass-entry.ts` already reads its tally line before publishing for
 the same reason: a question that can refuse an entry has to be asked while
 refusing still costs nothing, because raised after the write it would leave a
 page on disk that no artifact accounts for. Asked here, a disagreement
 publishes nothing and settles nothing, and the stage caches still hold every
 answer, so a re-run reproduces the contradiction rather than losing it.

 ONE-SIDED, WHICH IS WHY IT MAY REFUSE AN ENTRY AT ALL. Neither check can call
 a correct page wrong: the scan asks only that each wording occur in order,
 which a correct page satisfies by construction, and the arithmetic is an
 equality only where no anchor was filled and a floor otherwise. A red result
 is a defect in assembly or publishing rather than a judgement about quality.

 @param artifact - settled entry, read for what each slice would carry

 @param archive - archive English this entry is published over

 @param pageText - assembled page, not yet written

 @param entryId - person entry, named in the refusal

 @throws {@link PublishedPageDisagreesError} when the page lost, reordered, or
 gained text against what the artifact accounts for

 @example
 ```ts
 refusePageThatDisagrees({ artifact, archive, pageText, entryId, },);
 ```
 */
export function refusePageThatDisagrees(
  {
    artifact,
    archive,
    pageText,
    entryId,
  }: {
    readonly artifact: WouldShipSource;
    readonly archive: ParsedArchiveText;
    readonly pageText: string;
    readonly entryId: string;
  },
): void {
  /**
   Wordings the page does not carry in slice order.
   */
  const { missing, } = pageCarriesEveryWording({
    artifact,
    pageText,
  },);

  if (missing.length > 0)
    throw new PublishedPageDisagreesError({
      entryId,
      disagreement: {
        kind: 'wordings-missing',
        missing,
      },
    },);

  /**
   What the page should weigh against what it does.
   */
  const weight = pageWeighsWhatItShould({
    artifact,
    archive,
    pageText,
  },);

  if (!pageWeightRefutes({ weight, },))
    return;
  // `pageWeightRefutes` returns false for an unweighable weight, so the
  // refutes check has already returned; the throw exists to narrow the types.
  if (weight.kind === 'unweighable')
    throw new Error('unreachable: an unweighable weight cannot refute',);

  throw new PublishedPageDisagreesError({
    entryId,
    disagreement: {
      kind: 'weight-off',
      actual: weight.actual,
      expected: weight.expected,
      exact: weight.exact,
    },
  },);
}

/**
 How one settled entry's artifact and published page line up.

 @example
 ```ts
 const pairing: PublishedPairing = { matched: ['lintong'], unpublished: [], unsettled: [], };
 ```
 */
export type PublishedPairing = {
  /**
   Entries carrying both an artifact and a page, in the order given.
   */
  readonly matched: readonly string[];

  /**
   Entries the run settled and never published.

   THE ORDERING IN `pass-entry-persist.ts` IS SUPPOSED TO MAKE THIS EMPTY, and
   that is exactly why it is worth counting. `publishFixedPage` runs BEFORE the
   artifact write so that an artifact existing means a page was written, and a
   pass builds its skip set from the artifacts on disk. An entry in this list
   is one no pass settles again; the archive ships for it until the next pass
   started in the runs directory writes its page from the artifact
   (`pass-republish.ts`, ledger A16c).
   */
  readonly unpublished: readonly string[];

  /**
   Entries carrying a page whose artifact is absent.

   The other half of the same ordering, and the expected one: a crash between
   the two writes leaves this rather than `unpublished`. The page ships as it
   stands, since a run always ships (the owner, 2026-09-27), with nothing to
   check it against until a pass settles the entry again; it is reported so
   the two cases are never counted together.
   */
  readonly unsettled: readonly string[];
};

/**
 Pairs the entries a run settled against the pages it published.

 @param settled - entry ids one artifact each was written for

 @param published - entry ids one page each was found for

 @returns Which entries have both, and which have only one

 @example
 ```ts
 const pairing = pairPublishedPages({ settled, published, },);
 ```
 */
export function pairPublishedPages(
  {
    settled,
    published,
  }: {
    readonly settled: readonly string[];
    readonly published: readonly string[];
  },
): PublishedPairing {
  /**
   Pages by entry, for membership tests that do not rescan the list.
   */
  const hasPage = new Set(published,);

  /**
   Artifacts by entry, on the same terms.
   */
  const hasArtifact = new Set(settled,);

  return {
    matched: settled.filter(function hasBoth(id,): boolean {
      return hasPage.has(id,);
    },),
    unpublished: settled.filter(function missingPage(id,): boolean {
      return !hasPage.has(id,);
    },),
    unsettled: published.filter(function missingArtifact(id,): boolean {
      return !hasArtifact.has(id,);
    },),
  };
}

//endregion Published page check
