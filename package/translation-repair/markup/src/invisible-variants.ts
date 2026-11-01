//region Invisible variants
// CHARACTERS A READER CANNOT TELL FROM THEIR PLAIN COUNTERPART, folded at the
// point where a model's text enters the pipeline.
//
// The 2026-08-26 output reading found "non-binary" written with U+2011
// NON-BREAKING HYPHEN where the archive had the ASCII hyphen, and nothing
// between the model and the page had noticed. Measured over every archive
// page at the pin, U+2011 occurs 11 times in 92 pages and the no-break space
// and soft hyphen never, so a model writing one of these is almost always
// reproducing a typing artefact of its own rather than the archive's usage.
//
// WHAT THIS DOES NOT TOUCH. Typographic quotes (U+2018, U+2019, U+201C,
// U+201D), dashes (U+2013, U+2014) and the ellipsis (U+2026) are visible
// choices and the archive's own majority convention (1173 U+2019 across the
// corpus), so they pass through. The zero-width joiner (U+200D) is left alone
// because emoji sequences are built from it.
//
// APPLIED AT INTAKE, NOT AT PUBLISH. Both deciders judge the bytes that ship;
// folding after judging would ship bytes nobody judged. The fold is
// therefore applied where each lane first holds an answer, before any check
// reads it: the repair lane's wire readers, and `translator-answer-fold.ts`
// for the translate and consolidation lanes, which until ledger B112 folded
// only when building the slate, after the publication rule and the floor had
// read the unfolded answer. The corpus pass folds archive English before
// preparation. Candidate, incumbent, artifact and page all carry bytes the
// deciders and the checks saw.

/**
 Code point folded to its visible counterpart, with the reason.
 */
type Fold = {
  /**
   Character as a model writes it.
   */
  readonly from: string;

  /**
   What a reader would have typed.
   */
  readonly to: string;

  /**
   Name, for the finding.
   */
  readonly name: string;
};

/**
 Every fold, in the order applied. Order does not matter: no `to` is any
 other entry's `from`. SPELLED AS ESCAPES on purpose: as the characters
 themselves they are invisible in a diff, and the first version of this module's
 test lost its literal fixtures to the tool that wrote it, which is the failure
 the fold exists to catch.
 */
const FOLDS: readonly Fold[] = [
  {
    from: '\u2011',
    to: '-',
    name: 'U+2011',
  },
  {
    from: '\u00A0',
    to: ' ',
    name: 'U+00A0',
  },
  {
    from: '\u202F',
    to: ' ',
    name: 'U+202F',
  },
  {
    from: '\u00AD',
    to: '',
    name: 'U+00AD',
  },
  {
    from: '\u200B',
    to: '',
    name: 'U+200B',
  },
  {
    from: '\u2060',
    to: '',
    name: 'U+2060',
  },
  {
    from: '\uFEFF',
    to: '',
    name: 'U+FEFF',
  },
];

/**
 What folding a text produced.

 @example
 ```ts
 const { text, findings, } = foldInvisibleVariants({ text: answer, },);
 ```
 */
export type FoldedText = {
  /**
   Text with every invisible variant replaced by its plain counterpart.
   */
  readonly text: string;

  /**
   One finding per code point folded, naming it and how often, empty when
   nothing was folded.
   */
  readonly findings: readonly string[];
};

/**
 Counts how often `needle` occurs in `text`, as one linear pass.

 @param text - text scanned

 @param needle - single character looked for

 @returns Occurrences

 @example
 ```ts
 const hyphens = occurrences({ text, needle: '\u2011', },);
 ```
 */
function occurrences(
  {
    text,
    needle,
  }: {
    readonly text: string;
    readonly needle: string;
  },
): number {
  /**
   Occurrences seen so far.
   */
  let count = 0;

  /**
   Cursor, advanced past each occurrence.
   */
  let at = text.indexOf(needle,);
  while (at >= 0) {
    count += 1;
    at = text.indexOf(
      needle,
      at + needle.length,
    );
  }
  return count;
}

/**
 Replaces every invisible variant in a model's text with its plain
 counterpart, and says what it replaced.

 @param text - text as the model wrote it

 @returns Folded text and one finding per code point folded

 @example
 ```ts
 const folded = foldInvisibleVariants({ text: 'non\u2011binary', },);
 // folded.text === 'non-binary', folded.findings deep-equals ['invisible-variant-folded (U+2011 x1)']
 ```
 */
export function foldInvisibleVariants({ text, }: { readonly text: string; },): FoldedText {
  return FOLDS.reduce(
    function fold(
      folded: FoldedText,
      entry: Fold,
    ): FoldedText {
      /**
       How often this code point occurs in the text as it stands.
       */
      const count = occurrences({
        text: folded.text,
        needle: entry.from,
      },);
      if (count === 0)
        return folded;
      return {
        text: folded
          .text
          .replaceAll(
          entry.from,
          entry.to,
          ),
        findings: [
          ...folded.findings,
          `invisible-variant-folded (${entry.name} x${String(count,)})`,
        ],
      };
    },
    {
      text,
      findings: [],
    },
  );
}

//endregion Invisible variants
