import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readPageSkeleton, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Quote preservation
// A REPLACEMENT MAY NOT LEAVE THE DOCUMENT WITH FEWER QUOTED PASSAGES THAN THE
// ARCHIVE HAD.
//
// WHY THIS QUESTION RATHER THAN THE OTHER ONE. The harm is that a lane writing
// from the source alone deletes English the source cannot account for, and the
// obvious response is to work out which passages those are. That turns out not
// to be decidable cheaply: translation changes bytes by construction, so no
// exact match distinguishes an unpaired passage from an ordinary translated
// one, and deciding it properly is what the aligner already does. Measured
// against the nine transcripts known in the corpus, a structural rule anchored
// on shared markup reaches two of them, and fails three separate ways on the
// rest.
//
// DELETION, ON THE OTHER HAND, IS DECIDABLE FROM THE TWO TEXTS ALONE. Measured
// over both settled pools on 2026-08-18:
//
//   flagged pool   4 of 64 shipped replacements drop a whole quote block
//                  all four in the translate lane
//   natural pool   0 of 69
//
// Four hits, every one a passage a reader would want back, and no false
// positive in sixty-nine natural rows.
//
// THE COUNTER-CASE THAT KEEPS THIS HONEST. Deleting is not always wrong: on one
// entry the repair lane removed a paragraph of translator invention with nine
// separate accepted findings against it, and that was correct. It was not a
// blockquote, so this guard would not have stopped it. The guard is deliberately
// narrow for that reason: it protects the shape transcripts and quoted letters
// take, and says nothing about prose.
//
// THE COUNT IS THE PARSER'S (ledger B42, 2026-09-30). It was the number of
// blank-line-separated chunks opening with `>`, which the parser and the floor
// disagree with: a quote opening on the line after a paragraph's is a
// blockquote to both and was none to the count, so a replacement keeping every
// quote was refused, as the stored record of hulicaijia24 slice 2 was; a quote
// inside a container tag, which the floor's top-level block list does not
// reach, was none either. Both texts are now read as the floor reads a page,
// the strict grammar first and plain markdown where it refuses, and a text
// neither grammar reads keeps the archive, since a check that could not run
// has not shown the quotes survive.

/**
 How many quoted passages a text carries, or that no grammar read it.

 @example
 ```ts
 const count: QuotedPassageCount = 2;
 ```
 */
export type QuotedPassageCount = number | 'unreadable';

/**
 Both sides of the quote guard's comparison.

 STORED on a quote-loss refusal, as the dropped names are on a declared-name
 one: the guard compares the judged part of the archive, the target-only run
 held out, and the record keeps only the whole archive, so a reporter
 recounting from the record counted the held-out run's quotes as the
 archive's and named a count the guard never compared.

 @example
 ```ts
 const quotedPassages: QuotedPassages = { archive: 2, replacement: 1, };
 ```
 */
export type QuotedPassages = {
  /**
   Quoted passages of the archive wording being replaced.
   */
  readonly archive: QuotedPassageCount;

  /**
   Quoted passages of the wording that would replace it.
   */
  readonly replacement: QuotedPassageCount;
};

/**
 Whether a stored value is one side's count.

 @param value - parsed JSON field

 @returns Whether it is a count of none or more, or the unreadable mark

 @example
 ```ts
 isQuotedPassageCount(2,); // true
 ```
 */
function isQuotedPassageCount(value: unknown,): value is QuotedPassageCount {
  if (value === 'unreadable')
    return true;
  if ((typeof value) !== 'number')
    return false;
  return Number.isSafeInteger(value,) && (value >= 0);
}

/**
 Whether a stored value is the pair a quote-loss refusal carries, so the
 cache reader recomputes a record that lacks it rather than reporting counts
 nobody compared.

 @param value - parsed JSON field

 @returns Whether both sides are counts

 @example
 ```ts
 if (isQuotedPassages(record.quotedPassages,)) report(record,);
 ```
 */
export function isQuotedPassages(value: unknown,): value is QuotedPassages {
  return isJsonRecord(value,)
    && isQuotedPassageCount(value.archive,)
    && isQuotedPassageCount(value.replacement,);
}

/**
 Counts one text's quoted passages as the floor reads a page.

 @param text - archive wording or its replacement

 @returns Blockquotes at every depth, or that neither grammar read the text

 @example
 ```ts
 const count = quotedPassagesIn({ text: '> She said so.', },);
 ```
 */
function quotedPassagesIn({ text, }: { readonly text: string; },): QuotedPassageCount {
  /**
   Page reading under the strict grammar, or plain markdown where it refuses.
   */
  const { read, } = readPageSkeleton({ text, },);
  if (read.kind === 'unparseable')
    return 'unreadable';
  return read.skeleton
    .quotedPassages;
}

/**
 Counts the quoted passages on both sides of a replacement.

 @param incumbentText - archive wording being replaced

 @param shippedText - wording the lane wants to put there

 @returns Both counts, for the guard and for the record it refuses with

 @example
 ```ts
 const quotedPassages = countQuotedPassages({ incumbentText, shippedText, },);
 ```
 */
export function countQuotedPassages(
  {
    incumbentText,
    shippedText,
  }: {
    readonly incumbentText: string;
    readonly shippedText: string;
  },
): QuotedPassages {
  return {
    archive: quotedPassagesIn({ text: incumbentText, },),
    replacement: quotedPassagesIn({ text: shippedText, },),
  };
}

/**
 Whether a replacement would leave fewer quoted passages than the archive has.

 @param quotedPassages - both sides' counts

 @returns Whether a quoted passage would be lost, or could not be shown kept

 @example
 ```ts
 const lost = dropsQuotedPassage({ quotedPassages, },);
 ```
 */
export function dropsQuotedPassage(
  { quotedPassages, }: { readonly quotedPassages: QuotedPassages; },
): boolean {
  /**
   Both counts, named for the comparison.
   */
  const {
    archive,
    replacement,
  } = quotedPassages;
  if ((archive === 'unreadable') || (replacement === 'unreadable'))
    return true;
  return replacement < archive;
}

/**
 Names one side's count in a finding, in the number its count takes.

 @param side - which wording the count is of

 @param count - quoted passages, or that no grammar read the text

 @returns Clause naming the side and what it carries

 @example
 ```ts
 const clause = sideClause({ side: 'archive', count: 1, },); // 'archive carries 1 quoted passage'
 ```
 */
function sideClause(
  {
    side,
    count,
  }: {
    readonly side: 'archive' | 'replacement';
    readonly count: QuotedPassageCount;
  },
): string {
  if (count === 'unreadable')
    return `${side} could not be read to count its quoted passages`;

  /**
   The noun in the number the count takes.
   */
  const noun = wordForCount({
    count,
    one: 'passage',
    many: 'passages',
  },);
  return `${side} carries ${String(count,)} quoted ${noun}`;
}

/**
 Names a quote-loss refusal in scorecard-stable wording.

 PARALLEL TO THE ALIGNMENT REFUSAL, so a run's findings read the same way
 whichever guard kept the archive, and so a corpus-wide count can separate
 them.

 @param sliceIndex - slice refused

 @param quotedPassages - counts the guard compared

 @returns One finding line

 @example
 ```ts
 const finding = quoteLossRefusalFinding({ sliceIndex, quotedPassages, },);
 ```
 */
export function quoteLossRefusalFinding(
  {
    sliceIndex,
    quotedPassages,
  }: {
    readonly sliceIndex: number;
    readonly quotedPassages: QuotedPassages;
  },
): string {
  /**
   Both sides' clauses, the archive's first.
   */
  const clauses = [
    sideClause({
      side: 'archive',
      count: quotedPassages.archive,
    },),
    sideClause({
      side: 'replacement',
      count: quotedPassages.replacement,
    },),
  ];
  return `translate-refused-quote-loss (slice ${String(sliceIndex,)}: ${clauses.join(', ',)})`;
}

//endregion Quote preservation
