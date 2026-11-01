import {
  COMMUNITY_GLOSSARY,
  type CommunityTerm,
  communityTermsIn,
  foldForGlossary,
  formStarts,
  renderingSpans,
  textCarriesForm,
  withoutHtmlComments,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { withoutGlossedTitles, } from './translate-han-title.ts';

//region Community term floor
// CLASS ONE HUNDRED NINETEEN (shi_Yumiaoya19, 2026-09-24). The glossary put
// the community's words on every sheet as evidence to weigh, and for 药娘 the
// judges weighed the archive translator's comment higher and kept the word
// in Han, where two earlier runs had written two different English forms.
// The owner ruled on 2026-09-24 that the page says "trans girl" or "trans
// woman". A candidate that keeps a glossary term the original carries in
// Han, or writes a form the entry refuses (the pinyin), is refused HERE
// before any judge reads it. A rendering the entry lists is not required:
// a rendering inflects and the judges still choose among renderings. The
// owner added the same day that the term is refused even where the existing
// translation keeps it, so the page's own text earns no exception; comments
// are cut on both sides. Every match reads word boundaries
// (`glossary-match.ts`, class one hundred eighty-six).

/**
 Accepted renderings quoted for a finding.

 @param entry - term whose renderings are listed

 @returns Renderings quoted and joined with "or"

 @example
 ```ts
 renderingList({ entry, },);
 // => '"trans girl" or "trans woman" or "trans women"'
 ```
 */
function renderingList(
  { entry, }: { readonly entry: CommunityTerm; },
): string {
  return entry.renderings
    .map(function quoted(rendering,): string {
      return `"${rendering}"`;
    },)
    .join(' or ',);
}

/**
 Whether an accepted rendering of the entry carries a refused-form occurrence
 as part of itself: it contains the occurrence ("whisker head cover" holding
 "head"), or it opens inside the occurrence and runs past its end ("inside
 her head" opening "inside her headpiece", class one hundred sixty-three). A
 rendering that ends inside the occurrence ("a minor" in "a minor trans
 girl") excuses nothing, and a rendering of another entry excuses nothing
 either, since it names another word. A rendering stands where the departure
 check reads it (`renderingSpans`), inflected and bounded, so a rendering
 that counts as written also excuses what it overlaps ("fossil-fuel cars"
 over "fuel car", ledger R2).

 @param entry - term whose renderings may excuse the occurrence

 @param folded - folded candidate text

 @param start - index the refused-form occurrence starts at

 @param end - index just past the refused-form occurrence

 @returns True where an accepted rendering carries the occurrence

 @example
 ```ts
 renderingCarries({ entry, folded: 'inside her head mask', start: 0, end: 15, },);
 // => true where "head mask" is an accepted rendering
 ```
 */
function renderingCarries(
  {
    entry,
    folded,
    start,
    end,
  }: {
    readonly entry: CommunityTerm;
    readonly folded: string;
    readonly start: number;
    readonly end: number;
  },
): boolean {
  return entry
    .renderings
    .some(function carries(rendering,): boolean {
      return renderingSpans({
        folded,
        rendering,
      },)
        .some(function covers({
          start: renderingStart,
          end: renderingEnd,
        },): boolean {
          /**
           Whether the rendering holds the whole occurrence.
           */
          const contains = (renderingStart <= start) && (renderingEnd >= end);
          /**
           Whether the rendering opens inside the occurrence and runs past it.
           */
          const continuesPast = (renderingStart > start)
            && (renderingStart < end)
            && (renderingEnd > end);
          return contains || continuesPast;
        },);
    },);
}

/**
 First form the entry refuses that a text writes at word boundaries, a plural
 included, where some occurrence is not carried by an accepted rendering
 (`renderingCarries`).

 @param entry - term whose refused forms are looked for

 @param text - candidate text with comments cut

 @returns Refused form found, empty where none is

 @example
 ```ts
 refusedFormIn({ entry, text: 'a little yaoniang', },);
 // => 'yaoniang'
 ```
 */
function refusedFormIn(
  {
    entry,
    text,
  }: {
    readonly entry: CommunityTerm;
    readonly text: string;
  },
): string {
  /**
   Candidate folded once for every form.
   */
  const folded = foldForGlossary({ text, },);
  /**
   Refused form the text writes, undefined where it writes none.
   */
  const found = entry
    .refusedForms
    .find(function written(refusedForm,): boolean {
      /**
       Refused form folded as the candidate is.
       */
      const form = foldForGlossary({ text: refusedForm, },);
      return formStarts({
        folded,
        form,
        end: 'plural',
      },)
        .some(function standsApart(start,): boolean {
          return !renderingCarries({
            entry,
            folded,
            start,
            end: start + form.length,
          },);
        },);
    },);
  return found ?? '';
}

/**
 Findings for glossary terms the original carries that a candidate keeps in
 Han or writes in a refused form.

 @param sourceText - original passage

 @param candidateText - rendering under the floor

 @param glossary - terms to hold; defaults to the corpus glossary

 @param noun - what the finding calls a term, so a word from the rendering
 glossary (`rendering-glossary.ts`) is not called the community's

 @returns One finding per term the candidate fails, empty where it passes

 @example
 ```ts
 const findings = communityTermFindings({ sourceText, candidateText, },);
 ```
 */
export function communityTermFindings(
  {
    sourceText,
    candidateText,
    glossary = COMMUNITY_GLOSSARY,
    noun = 'community term',
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly glossary?: readonly CommunityTerm[];
    readonly noun?: string;
  },
): readonly string[] {
  /**
   Terms the original carries outside its comments.
   */
  const carried = communityTermsIn({
    text: sourceText,
    glossary,
  },);
  if (carried.length === 0)
    return [];
  /**
   Candidate with its comments cut, and with the original's Han titles cut
   where the title floor accepts them as kept (glossed), since a word inside
   such a title is the title's, not a word left untranslated (ledger F-11).
   */
  const candidate = withoutGlossedTitles({
    sourceText,
    candidateText: withoutHtmlComments({ text: candidateText, },),
  },);
  return carried.flatMap(function findingsFor(entry,): readonly string[] {
    if (textCarriesForm({
      text: candidate,
      form: entry.term,
      end: 'inflected',
    },)) {
      return [
        `Your translation leaves the ${noun} ${entry.term} untranslated. Render it as ${
          renderingList({ entry, },)
        }: ${entry.why}.`,
      ];
    }
    /**
     Refused form the candidate writes, empty for none.
     */
    const refused = refusedFormIn({
      entry,
      text: candidate,
    },);
    if (refused === '')
      return [];
    return [
      `Your translation writes "${refused}" for the ${noun} ${entry.term}. Render it as ${
        renderingList({ entry, },)
      }: ${entry.why}.`,
    ];
  },);
}

//endregion Community term floor
