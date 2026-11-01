//region Markup-only slice
// A slice whose ORIGINAL is structure rather than prose, which the relocation
// finding names as a known false donor.
//
// WHY IT MATTERS TO THE DISPLACEMENT SCREEN. That screen calls a slice a
// relocation candidate when a high-ratio slice sits beside a below-baseline
// neighbour, reading the pair as a passage that moved from one to the other. A
// slice that is entirely a photo component or an image block sits at a ratio
// near 1.00 whatever the translator did, because the same markup appears
// verbatim on both sides. It is below baseline for a reason that has nothing to
// do with giving text up, so it pairs with any high neighbour and always will.
//
// IT IS NAMED, NOT SUPPRESSED, which is this instrument's rule throughout: size
// cannot deliver a verdict, so every reading here says where to look. A reader
// subtracting a named subset knows what was subtracted; a screen that silently
// dropped pairs would report a smaller number nobody could audit.

/**
 Share of a slice's solid lines that must be structural before its original
 counts as markup rather than prose.

 NOT 1.0. A photo block commonly carries one caption or attribution line, and
 a slice that is nine parts component to one part caption still cannot expand
 enough to be a donor. Measured against the corpus at the pinned commit, this
 threshold selects exactly the two pairs identified by hand and nothing
 else, so it is not doing any work beyond the class it was written for.
 */
const MARKUP_LINE_SHARE = 0.8;

/**
 Whether one line is structure rather than prose.

 DELIBERATELY CRUDE, and prefix-based rather than parsed. The question is not
 what this markup means, only whether the translator had any prose here to
 move; a component tag, an image, a fence, a table row and the inside of a
 props array all answer no. Parsing the document to decide that would be a
 larger instrument than the reading it serves.

 @param line - one line of a slice's original

 @returns Whether it carries structure rather than translatable prose

 @example
 ```ts
 const structural = isStructuralLine({ line: '<PhotoScroll photos={[', },);
 ```
 */
function isStructuralLine(
  { line, }: { readonly line: string; },
): boolean {
  /**
   Line without its indentation, since indentation is what nests markup.
   */
  const trimmed = line.trim();

  /**
   Whether the line is one quoted path inside a props array. EITHER QUOTE
   MARK: four source pages at pin `a41fc607` quote their photo paths with
   double marks, and this screen, like the photo reference reader until
   2026-09-04, knew only the single one, so a double-quoted photo block
   read as three parts prose to two parts markup.
   */
  const quotedPath = (trimmed.startsWith('\'',) || trimmed.startsWith('"',))
    && trimmed.includes('/',);

  return trimmed.startsWith('<',)
    || trimmed.startsWith('/>',)
    || trimmed.startsWith('}',)
    || trimmed.startsWith(']',)
    || trimmed.startsWith('![',)
    || trimmed.startsWith('```',)
    || trimmed.startsWith('|',)
    || quotedPath;
}

/**
 Share of a slice's non-blank lines that are structural.

 BLANK LINES ARE EXCLUDED FROM BOTH SIDES rather than counted as markup. A
 slice separated into paragraphs would otherwise read as more structural the
 more readable it is, which inverts the measurement.

 @param sourceText - slice's original

 @returns Fraction from 0 to 1, and 1 for a slice with no solid lines at all,
 since a slice with nothing in it has no prose to give up either

 @example
 ```ts
 const share = markupFraction({ sourceText, },);
 ```
 */
export function markupFraction(
  { sourceText, }: { readonly sourceText: string; },
): number {
  /**
   Lines carrying anything.
   */
  const solid = sourceText
    .split('\n',)
    .filter(function hasContent(line,): boolean {
      return line.trim() !== '';
    },);

  if (solid.length === 0)
    return 1;

  /**
   Those of them that are structure.
   */
  const structural = solid.filter(function isStructure(line,): boolean {
    return isStructuralLine({ line, },);
  },);

  return structural.length / solid.length;
}

/**
 Whether a slice's original is markup rather than prose, so it cannot have
 given a passage up.

 @param sourceText - slice's original

 @returns Whether this slice is disqualified as a relocation donor

 @example
 ```ts
 if (isMarkupOnly({ sourceText, },)) console.log('cannot be a donor',);
 ```
 */
export function isMarkupOnly(
  { sourceText, }: { readonly sourceText: string; },
): boolean {
  return markupFraction({ sourceText, },) >= MARKUP_LINE_SHARE;
}

//endregion Markup-only slice
