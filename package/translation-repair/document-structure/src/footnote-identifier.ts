//region Footnote identifier
// One spelling for one footnote, so the parser and the raw scanners agree.
//
// Markdown decides that `[^Note]` and `[^note]` name the SAME footnote, and
// mdast hands back the folded spelling on every node it builds. Everything this
// package finds by scanning text instead, unresolved references and the
// per-slice mention counts the assembly guard attributes with, sees the source
// spelling. Left alone, the two never meet: the guard looks up a finding about
// `note` in a mention map keyed `Note`, finds nothing, blames no slice, and
// withdraws every repair in the document. Measured on 2026-08-15 against a
// two-replacement fixture: both were withdrawn, including one that touched no
// footnote at all.
//
// The corpus at pin a41fc60 cannot trigger this. All 209 of its GFM markers are
// numeric, where folding is identity. A model writing `[^note]` into a
// replacement triggers it immediately, which is why this is not left to the
// corpus to decide.

/**
 Characters Markdown collapses runs of inside a label.

 Exactly micromark's set, which is narrower than Unicode whitespace: a
 non-breaking space is a character of the label rather than a separator. The
 space is listed for completeness and substituting it is a no-op.
 */
const LABEL_WHITESPACE: readonly string[] = [
  '\t',
  '\n',
  '\r',
  ' ',
];

/**
 Folds one footnote label to the spelling mdast keys its nodes by.

 Reproduces `normalizeIdentifier(label).toLowerCase()`, which is what
 `mdast-util-gfm-footnote` applies to both references and definitions:
 whitespace runs collapse to one space, ends are trimmed, and the case fold
 runs down, up, then down again. That third pass is not decoration. Lowercase
 of an uppercased character is not always the character you started from, and
 one pass would fold two labels together that the parser keeps apart.

 @param identifier - label as written, between the marker's brackets

 @returns Same label in the spelling every mdast node carries

 @example
 ```ts
 const key = normalizeFootnoteIdentifier({ identifier: 'Note', },);
 ```
 */
export function normalizeFootnoteIdentifier(
  { identifier, }: { readonly identifier: string; },
): string {
  /**
   Label with every whitespace character standing as a plain space, which
   turns a run of them into a run of separators.

   Substituted whole-string per character rather than walked character by
   character: every character being replaced is ASCII, so no pass can land
   inside a surrogate pair or split a combining sequence.
   */
  const spaced = LABEL_WHITESPACE.reduce(
    function asSeparator(
      text,
      whitespace,
    ): string {
      return text.replaceAll(
        whitespace,
        ' ',
      );
    },
    identifier,
  );

  // Collapse and trim in one step: a run of separators yields empty pieces
  // between them, and dropping every empty piece drops the run's extra spaces
  // along with any at the ends.
  return spaced.split(' ',)
    .filter(function carriesText(piece,): boolean {
      return piece !== '';
    },)
    .join(' ',)
    .toLowerCase()
    .toUpperCase()
    .toLowerCase();
}

/**
 Whether a label relation names another footnote, as against respelling
 the one it starts from.

 The relabel planner, its closure, and the pass that reports the relabel
 each kept their own copy of this test (audit area six, 2026-09-28); three
 copies of what counts as a move could disagree about which rewrites exist.

 @param relation - label as the archive writes it and label it would carry

 @returns True when the two labels fold to different footnotes

 @example
 ```ts
 relabelsFootnote({ relation: { from: 'Note', to: 'note', }, },); // false
 relabelsFootnote({ relation: { from: '1', to: '2', }, },); // true
 ```
 */
export function relabelsFootnote(
  { relation, }: {
    readonly relation: {
      readonly from: string;
      readonly to: string;
    };
  },
): boolean {
  return normalizeFootnoteIdentifier({ identifier: relation.from, },)
    !== normalizeFootnoteIdentifier({ identifier: relation.to, },);
}

//endregion Footnote identifier
