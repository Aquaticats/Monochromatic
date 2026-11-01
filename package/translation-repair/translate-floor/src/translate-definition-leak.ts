import { footnoteMentions, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Footnote definition the passage does not carry
// MEASURED ON SHI_YUMIAOYA3 (2026-09-17): the translate lane's rendering of
// the opening section ended with the community-term note written out as a
// definition, though the original defines that note only in its last slice
// and the passage merely refers to it. Every slice floor passed it; the
// assembly then found the identifier defined twice, withdrew both carriers
// (the opening section and the definition itself), and the publish check
// refused the page for the link the withdrawn section carried. A definition
// belongs to the slice whose original defines it, so a candidate defining a
// note neither its original nor its page slice defines is refused before any
// judge reads it.

/**
 Identifiers a text defines, folded across both conventions since the two
 spell one note.

 @param text - passage to read

 @returns Identifiers with a definition in the text

 @example
 ```ts
 const defined = definedIdentifiers({ text: '[^1]: 那是它最喜欢的位置。', },);
 ```
 */
function definedIdentifiers({ text, }: { readonly text: string; },): ReadonlySet<string> {
  return new Set(footnoteMentions({ text, },)
    .filter(function isDefinition(mention,): boolean {
      return mention.role === 'definition';
    },)
    .map(function toIdentifier(mention,): string {
      return mention.identifier;
    },),);
}

/**
 Findings for a candidate defining a footnote neither the original passage
 nor the page slice it replaces defines.

 THE PAGE'S OWN NOTES STAY: the archive may define a note the original never
 had (accurate detail the translator added), and a candidate keeping that
 definition drops nothing; only a definition new to both sides is refused.

 @param sourceText - original slice

 @param pageText - page slice the candidate would replace, empty where none

 @param candidateText - candidate under validation

 @returns One finding per note the candidate alone defines

 @example
 ```ts
 const findings = definitionLeakFindings({ sourceText: '猫在打盹[^1]。', pageText: '', candidateText: 'The cat naps[^1].\n\n[^1]: A note.', },);
 ```
 */
export function definitionLeakFindings(
  {
    sourceText,
    pageText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Notes the original passage defines.
   */
  const sourceDefined = definedIdentifiers({ text: sourceText, },);
  /**
   Notes the page slice already defines.
   */
  const pageDefined = definedIdentifiers({ text: pageText, },);
  return [...definedIdentifiers({ text: candidateText, },),]
    .filter(function leaked(identifier,): boolean {
      return (!sourceDefined.has(identifier,)) && (!pageDefined.has(identifier,));
    },)
    .map(function toFinding(identifier,): string {
      return `Your translation defines footnote [^${identifier}], but the ORIGINAL passage only refers to it: `
        + 'the note is defined elsewhere on the page and rendered there. Keep the marker and leave the '
        + 'definition out.';
    },);
}

//endregion Footnote definition the passage does not carry
