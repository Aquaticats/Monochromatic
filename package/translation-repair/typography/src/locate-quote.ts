import {
  shownTranslation,
  type StoredStretch,
  storedStretchesOf,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { wordForCount, } from './count-word.ts';
import type {
  AnchorTarget,
  DocumentSide,
  SpanAnchor,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  collapseLineBreaks,
  collapseSoftLineBreaks,
  normalizePunctuation,
} from './quote-normalize.ts';
import {
  continuesLatinWord,
  isLatinWordCharacter,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { rendersAsNothing, } from './renders-as-nothing.ts';

//region Quote location
// Deterministic evidence anchoring: find a critic's quote in the document,
// reject absence and ambiguity, and bind the located region to block nodes.
// Block-crossing regions split into one span per touched block, because models
// quote across paragraph boundaries no matter what the prompt demands. A quote
// copied out of a sheet that cut the notes between translators splits the same way
// at each note it crosses (`document-structure/src/translators-note-view.ts`).
//
// ONE SEARCH OVER THE BROADEST ACCEPTED FORM, rather than a chain of passes
// from strict to loose. Both normalizations replace one UTF-16 unit with one,
// so offsets found in the normalized text index the stored document unchanged.
//
// WHY IT IS NOT A CHAIN, which is what it was until a review found the hole.
// Each pass used to check ambiguity WITHIN ITS OWN CLASS and return on its
// first hit, so a document holding `bad\nword` early and `bad word` late
// answered the quote `bad word` with the late one: unique among byte-exact
// matches, and the earlier occurrence just as valid under the wrapping rule the
// next pass would have applied. Whitespace and punctuation in a model's quote
// are not evidence of WHICH occurrence it meant, since a model normalizes both
// when it copies, so uniqueness has to be judged over every form treated as
// equal. Measured before changing it, over three corpus passes and 16,479
// anchored quotes checked against the slice each was anchored in: NOT ONE would
// be refused by the stricter rule. The same count against whole pages, which is
// the wrong scope but proves the probe can see ambiguity, reports 566.

/**
 Located-quote outcome: anchors, or the failure reason.

 @example
 ```ts
 const location: QuoteLocation = { located: false, reason: 'quote-not-found (target)', };
 ```
 */
export type QuoteLocation =
  | {
    readonly located: true;

    /**
     One anchor per block the quoted region touches, never none;
     block-crossing quotes split into per-node spans.
     */
    readonly anchors: readonly [
      SpanAnchor,
      ...SpanAnchor[]
    ];
  }
    | {
      readonly located: false;

    /**
     Which check refused, in scorecard-stable wording.
     */
      readonly reason: string;
    };

/**
 Binds one located region to its blocks.
 A region inside one block yields one anchor;
 a region crossing blocks splits into one anchor per touched block,
 each carrying the document's own bytes for its intersection
 (inter-block gaps are simply not covered by any span).

 @param document - side being anchored

 @param side - which side the anchors belong to

 @param at - region start in document offsets

 @param end - exclusive region end

 @returns Anchors, or the outside-blocks failure

 @example
 ```ts
 const bound = bindQuoteRegion({ document, side: 'target', at, end, },);
 ```
 */
function bindQuoteRegion(
  {
    document,
    side,
    at,
    end,
  }: {
    readonly document: AnchorTarget;
    readonly side: DocumentSide;
    readonly at: number;
    readonly end: number;
  },
): QuoteLocation {
  /**
   First block the region touches, then the rest, in document order.
   */
  const [firstTouched, ...laterTouched] = document
    .nodes
    .filter(function overlapping(candidate,) {
      return (candidate.startOffset < end) && (candidate.endOffset > at);
    },);
  if (firstTouched === undefined) {
    return {
      located: false,
      reason: `quote-outside-blocks (${side})`,
    };
  }

  /**
   One block's share of the region.

   @param node - block the region touches

   @returns Its anchor
   */
  function toAnchor(node: AnchorTarget['nodes'][number],): SpanAnchor {
    /**
     Start of this block's share of the region.
     */
    const spanStart = Math.max(
      at,
      node.startOffset,
    );

    /**
     Exclusive end of this block's share.
     */
    const spanEnd = Math.min(
      end,
      node.endOffset,
    );
    return {
      side,
      nodeId: node.id,
      nodeHash: node.contentHash,
      startOffset: spanStart,
      endOffset: spanEnd,
      quotedText: document.text
        .slice(
        spanStart,
        spanEnd,
        ),
    };
  }

  return {
    located: true,
    anchors: [
      toAnchor(firstTouched,),
      ...laterTouched.map(toAnchor,),
    ],
  };
}

/**
 Binds the stored stretches a quote across cut notes came back as, each to its
 blocks, so no anchor holds a note.

 A stretch that lies wholly between blocks binds to nothing and is passed over,
 as the gap between two blocks is for a quote across them; the quote is
 refused as outside the blocks only when no stretch binds.

 @param document - side being anchored

 @param side - which side the anchors belong to

 @param stretches - stored stretches of the quote, in order

 @returns Anchors of every stretch that binds, or the outside-blocks failure

 @example
 ```ts
 const bound = bindQuoteStretches({ document, side: 'target', stretches: [{ from: 0, to: 22, },], },);
 ```
 */
function bindQuoteStretches(
  {
    document,
    side,
    stretches,
  }: {
    readonly document: AnchorTarget;
    readonly side: DocumentSide;
    readonly stretches: readonly StoredStretch[];
  },
): QuoteLocation {
  /**
   Anchors of every stretch that binds, in order.
   */
  const anchors = stretches.flatMap(function anchorsOf(stretch,): readonly SpanAnchor[] {
    /**
     This stretch bound to the blocks it touches.
     */
    const bound = bindQuoteRegion({
      document,
      side,
      at: stretch.from,
      end: stretch.to,
    },);
    return bound.located ? bound.anchors : [];
  },);
  /**
   The first anchor, which a location needs, and the rest.
   */
  const [first, ...later] = anchors;
  if (first === undefined) {
    return {
      located: false,
      reason: `quote-outside-blocks (${side})`,
    };
  }
  return {
    located: true,
    anchors: [
      first,
      ...later,
    ],
  };
}

/**
 Describes the missed quote into the failure finding by its shape, so a miss
 can be diagnosed by size and script rather than only counted, and without
 writing corpus text into a finding.

 NO TEXT, BY THE RULE FINDINGS LIVE UNDER. An earlier version quoted up to
 sixty characters of the needle; findings travel into logs, artifacts and any
 command that prints them, none of which may carry corpus text. The length,
 counted once over the one-line form, and the count of Latin tokens say what
 kind of quote missed (a paragraph, a name, a number) and nothing of its
 wording.

 @param needle - punctuation-normalized quote that was not found

 @returns Shape note prefixed with a space, ready to append to a reason

 @example
 ```ts
 needlePreview({ needle: 'a quote that missed', },);
 ```
 */
function needlePreview(
  {
    needle,
  }: {
    readonly needle: string;
  },
): string {
  /**
   Needle flattened to one line, the form a finding would have shown.
   */
  const flat = collapseLineBreaks({ text: needle, },);

  /**
   Latin tokens in it, counted as runs of Latin letters and digits in one
   linear pass; a combining mark goes on with the token before it, so
   `Château` is one token however its accent is written (ledger B18).
   */
  const counted = {
    tokens: 0,
    inToken: false,
  };
  for (const character of flat) {
    /**
     Whether this character continues a token.
     */
    const inToken = counted.inToken
      ? continuesLatinWord({ character, },)
      : isLatinWordCharacter({ character, },);
    if (inToken && (!counted.inToken))
      counted.tokens += 1;
    counted.inToken = inToken;
  }
  return ` needle=${String(flat.length,)} ${
    wordForCount({
      count: flat.length,
      one: 'char',
      many: 'chars',
    },)
  }, ${String(counted.tokens,)} Latin ${
    wordForCount({
      count: counted.tokens,
      one: 'token',
      many: 'tokens',
    },)
  }`;
}

/**
 Locates one quote inside one document and binds it to its blocks.

 Searches the document and the quote in one canonical form, where curly and
 ASCII punctuation are the same character and a soft line break is a space, so
 a quote copied out of a wrapped paragraph still anchors. Refuses a quote that
 occurs more than once in THAT form, whatever the stored punctuation and
 wrapping happen to be, because a model's own punctuation and line breaks do
 not say which occurrence it read.

 A QUOTE THAT SHOWS A READER NOTHING IS EMPTY, asked of `rendersAsNothing`
 (ledger B40), not of an empty-string comparison: a quote of one ideographic
 space, zero-width space or Hangul filler that the document holds once
 anchored a critic's claim on a character no reader sees.

 A QUOTE COPIED OUT OF A SHEET THAT CUT THE NOTES BETWEEN TRANSLATORS is searched
 in the document as the sheet showed it (`original` given), and the place it is
 found comes back as the stored stretches of words either side of each note it
 crosses (`storedStretchesOf`), one anchor for each stretch and block, never an
 anchor that holds a note. Uniqueness is judged in the text as shown, since that
 is the text the quote was read in.

 @param document - side being searched

 @param side - which side the anchors belong to

 @param quote - substring the critic claims

 @param original - the original passage the sheet's translation was cut
 against, whose own comments stay in view; absent when the quote is stored text
 (a region a stage recorded from the document, a text no sheet cut), which is
 searched as it stands

 @returns Anchors, or the failure reason

 @example
 ```ts
 const located = locateQuote({ document, side: 'target', quote, original, },);
 ```
 */
export function locateQuote(
  {
    document,
    side,
    quote,
    original,
  }: {
    readonly document: AnchorTarget;
    readonly side: DocumentSide;
    readonly quote: string;
    readonly original?: string;
  },
): QuoteLocation {
  if (rendersAsNothing({ text: quote, },)) {
    return {
      located: false,
      reason: `empty-quote (${side})`,
    };
  }

  /**
   The document as the quote was read: cut of its notes between translators
   when the sheet it came from cut them, as it stands otherwise.
   */
  const shown = (original === undefined)
    ? undefined
    : shownTranslation({
      text: document.text,
      original,
    },);

  /**
   Document read in the broadest form this function accepts: punctuation
   variants canonical, soft line breaks read as spaces. Both maps replace one
   UTF-16 unit with one, so every offset here indexes the text the quote was
   read in (the stored document unless notes were cut), and anchors still
   carry the stored document's own characters.
   */
  const haystack = collapseSoftLineBreaks({ text: normalizePunctuation({ text: shown?.text ?? document.text, },), },);

  /**
   Quote read the same way.
   */
  const needle = collapseSoftLineBreaks({ text: normalizePunctuation({ text: quote, },), },);

  /**
   Where the quote sits once both are read that way.
   */
  const at = haystack.indexOf(needle,);
  if (at === (-1)) {
    return {
      located: false,
      reason: `quote-not-found (${side})${needlePreview({ needle, },)}`,
    };
  }
  if (haystack.includes(
    needle,
    at + 1,
  ))
    {
    return {
      located: false,
      reason: `ambiguous-quote (${side})`,
    };
  }
  if (shown === undefined) {
    return bindQuoteRegion({
      document,
      side,
      at,
      end: at + needle.length,
    },);
  }
  return bindQuoteStretches({
    document,
    side,
    stretches: storedStretchesOf({
      shown,
      start: at,
      end: at + needle.length,
    },),
  },);
}

//endregion Quote location
