import {
  collapseSoftLineBreaks,
  normalizePunctuation,
  rendersAsNothing,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { RenderingAuditInvariantError, } from './rendering-audit-invariant.ts';

//region Rendering audit anchoring
// Where in a document one claimed defect actually is, as an interval rather
// than as a string.
//
// TWO SPANS, NOT ONE, and the difference is what the first version of this got
// wrong. A quote does three jobs at once: it FINDS the text, it names the
// smallest wording that CHANGED, and it decides whether two voices are talking
// about the same defect. One string cannot do all three. Asking for a locator
// and a focus separates them, and the third job then falls to the focus
// interval, which is a fact about the document rather than about the auditor's
// typing habits.
//
// THE FLOOR IS GONE, and its absence is the point. A minimum length was meant
// to stop a quote that identifies nothing, but a short quote occurring exactly
// once identifies a span perfectly, and the floor's real effect was to force
// every voice to PAD: `not`, `three` and a three-character Chinese negation all
// failed it. Padding is what made two different defects in one sentence arrive
// as the same quote, so the rule written to prevent unfounded evidence was
// manufacturing false agreement. Uniqueness does the whole job the floor was
// reaching for, and does it without touching what a voice may point at.
//
// A FOCUS NEED ONLY BE UNIQUE INSIDE ITS LOCATOR, which is what lets a voice
// name a repeated negator: the locator says which sentence, the focus says
// which word of it, and neither has to be unique in the document alone.

/**
 What `indexOf` answers when a needle is absent.
 */
const NOT_FOUND = -1;

/**
 One span of a document, located.

 @example
 ```ts
 const span: AnchoredSpan = { text: '不吃罐头', start: 12, end: 16, };
 ```
 */
export type AnchoredSpan = {
  /**
   Wording the DOCUMENT holds here, rather than what the auditor typed, so a
   report never quotes a text back with characters it does not carry.
   */
  readonly text: string;

  /**
   Where the span begins.
   */
  readonly start: number;

  /**
   Where it ends, exclusive.
   */
  readonly end: number;
};

/**
 What locating one claim's two spans in one text found.

 @example
 ```ts
 const anchor: RenderingAuditSpanAnchor = { anchored: false, reason: 'ambiguous-locator (source)', };
 ```
 */
export type RenderingAuditSpanAnchor = {
  /**
   Both spans were found, and the focus sits inside the locator.
   */
  readonly anchored: true;

  /**
   Span that identifies which occurrence is meant.
   */
  readonly locator: AnchoredSpan;

  /**
   Smallest span carrying the claimed change.
   */
  readonly focus: AnchoredSpan;
} | {
  /**
   Nothing was located, for the stated reason.
   */
  readonly anchored: false;

  /**
   Which check refused, in wording a tally can group by.
   */
  readonly reason: string;
};

/**
 One text in the broadest form the anchoring accepts.

 @param text - text to canonicalize

 @returns Same text with punctuation variants folded and sole line breaks read
 as spaces

 @throws {@link Error} when canonicalization changed the length, since every
 offset here indexes the stored text through the canonical one

 @example
 ```ts
 const canonical = canonicalize({ text: quote, },);
 ```
 */
function canonicalize({ text, }: { readonly text: string; },): string {
  /**
   Text with both maps applied.
   */
  const folded = collapseSoftLineBreaks({ text: normalizePunctuation({ text, },), },);

  // THE LENGTH INVARIANT, CHECKED RATHER THAN TRUSTED. Both maps replace one
  // UTF-16 unit with one, which is what lets an offset found in the canonical
  // text index the stored text unchanged; `quote-normalize.ts` states it three
  // times. A map that ever stopped being length-preserving would not fail here,
  // it would return neighbouring wording as evidence and locate defects a few
  // characters off, so the invariant is worth more than the comment it used to
  // be.
  if (folded.length !== text.length)
    throw new RenderingAuditInvariantError({
      invariant: 'quote canonicalization changed the length, so no offset in it indexes the text',
    },);

  return folded;
}

/**
 Locates one span inside another, both already canonical.

 @param haystack - canonical text being searched

 @param needle - canonical span to find

 @param from - where to start, so a focus is searched inside its locator only

 @param to - where to stop, exclusive

 @returns Where it occurs uniquely, or which check refused

 @example
 ```ts
 const found = locateUnique({ haystack, needle, from: 0, to: haystack.length, },);
 ```
 */
function locateUnique(
  {
    haystack,
    needle,
    from,
    to,
  }: {
    readonly haystack: string;
    readonly needle: string;
    readonly from: number;
    readonly to: number;
  },
): { readonly at: number; } | { readonly refused: 'absent' | 'repeated'; } {
  /**
   Window the needle has to occur in.
   */
  const window = haystack.slice(
    from,
    to,
  );

  /**
   Where it first occurs in that window.
   */
  const at = window.indexOf(needle,);

  if (at === NOT_FOUND)
    return { refused: 'absent', };

  if (window.includes(
    needle,
    at + 1,
  ))
    return { refused: 'repeated', };

  return { at: from + at, };
}

/**
 Locates one claim's locator and focus in the text it names.

 A SPAN THAT SHOWS A READER NOTHING IS EMPTY, asked of `rendersAsNothing`
 (ledger B40) rather than of an empty-string comparison: a locator of one
 ideographic space the text held once, or a focus of the one space or
 zero-width space inside its locator, anchored a claim on a character no
 reader sees.

 @param text - side the claim names

 @param locator - span identifying which occurrence is meant

 @param focus - smallest span carrying the claimed change

 @param side - which side this is, for the refusal wording

 @returns Both spans as the text holds them, or why nothing was located

 @example
 ```ts
 const anchor = anchorLocatedSpan({ text: sourceText, locator, focus, side: 'source', },);
 ```
 */
export function anchorLocatedSpan(
  {
    text,
    locator,
    focus,
    side,
  }: {
    readonly text: string;
    readonly locator: string;
    readonly focus: string;
    readonly side: string;
  },
): RenderingAuditSpanAnchor {
  /**
   Text in the form quotes are matched against.
   */
  const haystack = canonicalize({ text, },);

  /**
   Locator in the same form.
   */
  const locatorNeedle = canonicalize({ text: locator, },);

  if (rendersAsNothing({ text: locator, },)) {
    return {
      anchored: false,
      reason: `empty-locator (${side})`,
    };
  }

  /**
   Where the locator is, if it identifies one span.
   */
  const located = locateUnique({
    haystack,
    needle: locatorNeedle,
    from: 0,
    to: haystack.length,
  },);

  if ('refused' in located) {
    return {
      anchored: false,
      reason: `${(located.refused === 'absent') ? 'unanchored' : 'ambiguous'}-locator (${side})`,
    };
  }

  /**
   End of the located span, which is the window a focus must fall inside.
   */
  const locatorEnd = located.at + locatorNeedle.length;

  /**
   Focus in canonical form.
   */
  const focusNeedle = canonicalize({ text: focus, },);

  if (rendersAsNothing({ text: focus, },)) {
    return {
      anchored: false,
      reason: `empty-focus (${side})`,
    };
  }

  /**
   Where the focus is within the locator.
   */
  const focused = locateUnique({
    haystack,
    needle: focusNeedle,
    from: located.at,
    to: locatorEnd,
  },);

  if ('refused' in focused) {
    return {
      anchored: false,
      reason: `${(focused.refused === 'absent') ? 'unanchored' : 'ambiguous'}-focus (${side})`,
    };
  }

  return {
    anchored: true,
    locator: {
      text: text.slice(
        located.at,
        locatorEnd,
      ),
      start: located.at,
      end: locatorEnd,
    },
    focus: {
      text: text.slice(
        focused.at,
        focused.at + focusNeedle.length,
      ),
      start: focused.at,
      end: focused.at + focusNeedle.length,
    },
  };
}

//endregion Rendering audit anchoring
