import { footnoteMentions, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Footnote marker the passage carries
// CLASS NINETY-TWO (XingZ626, 2026-09-23). The original's line about the
// rhythm-game song carries [^3], and the archive never rendered that note.
// The repair lane wrote the line from the archive's text without the marker,
// the contest chose that lane, the slate's proposals carrying the marker lost
// the gate 2 to 1, and no floor had asked for it. The definition, translated
// in the admitted tail (class fifty-nine), was then trimmed as an orphan at
// both assemblies and the note left the page, where XingZ625 had shipped it
// only because the translate lane won the same slice. A marker the ORIGINAL
// passage carries is the passage's structure, like its links and its hard
// breaks: a candidate without it is refused before any judge reads it, and
// the finding tells the writer to keep the marker and leave the definition
// out. The other direction stays open: the page may cite a note the original
// never had (the page's own apparatus, class eighty-five), so a marker the
// candidate carries beyond the original's is no fault here.

/**
 Identifiers a text cites, folded across both conventions since the two
 spell one note.

 @param text - passage to read

 @returns Identifiers with a reference in the text

 @example
 ```ts
 const cited = citedIdentifiers({ text: '猫在窗台上打盹[^3]。', },);
 ```
 */
function citedIdentifiers({ text, }: { readonly text: string; },): ReadonlySet<string> {
  return new Set(footnoteMentions({ text, },)
    .filter(function isReference(mention,): boolean {
      return mention.role === 'reference';
    },)
    .map(function toIdentifier(mention,): string {
      return mention.identifier;
    },),);
}

/**
 Findings for a candidate that drops a footnote marker the original
 passage carries.

 @param sourceText - original slice

 @param candidateText - candidate under validation

 @returns One finding per note the original cites and the candidate does not

 @example
 ```ts
 const findings = droppedMarkerFindings({ sourceText: '猫在窗台上打盹[^3]。', candidateText: 'The cat naps.', },);
 ```
 */
export function droppedMarkerFindings(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Notes the candidate cites.
   */
  const candidateCited = citedIdentifiers({ text: candidateText, },);
  return [...citedIdentifiers({ text: sourceText, },),]
    .filter(function dropped(identifier,): boolean {
      return !candidateCited.has(identifier,);
    },)
    .map(function toFinding(identifier,): string {
      return `Your translation drops footnote ${identifier}: the ORIGINAL passage carries the marker [^${identifier}] `
        + 'and your translation does not. Keep the marker where the note is cited, since the note is defined '
        + 'elsewhere on the page and loses its place without it, and leave the definition out.';
    },);
}

//endregion Footnote marker the passage carries
