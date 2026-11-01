import {
  archiveContributorNameForms,
  collectIdentityLines,
  declaredNamePairs,
  sourcePronounLines,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { entryNoteLines, } from './entry-notes.ts';
import { pageNameLines, } from './page-name-glossary.ts';
import type { RepairDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { glossaryTermLines, } from './rendering-glossary.ts';

//region Identity context lines
// THE PAGE'S DECLARED IDENTITY AS SHEET LINES, assembled in one place (ledger
// B28). Preparation built these inline, so the page title lexicon, which runs
// before preparation and settles lines that join them, was asked for a work's
// official English title without the web lookups and notes that establish
// one. Preparation and the lexicon now read the same assembly; the lexicon
// passes every context line but its own.

/**
 Declared names, pronoun, contributors, notes, community words, the page's own
 names and the caller's context lines, in the order every sheet reads them.

 @param sourceDocument - whole original, parsed

 @param targetDocument - whole archive, parsed

 @param sourceText - whole original as written

 @param targetText - whole archive as written

 @param contextLines - evidence lines bought outside preparation (web lookups
 of the works the original names, other entries' names, settled titles),
 appended last

 @param sourceEntryInfo - parsed `info.yml` of the original's corpus entry,
 carrying the recorded death; absent when the caller has none

 @returns Identity lines, none when the page declares and carries nothing

 @example
 ```ts
 const lines = pageIdentityLines({ sourceDocument, targetDocument, sourceText, targetText, contextLines: [], },);
 ```
 */
export function pageIdentityLines(
  {
    sourceDocument,
    targetDocument,
    sourceText,
    targetText,
    contextLines,
    sourceEntryInfo,
  }: {
    readonly sourceDocument: RepairDocument;
    readonly targetDocument: RepairDocument;
    readonly sourceText: string;
    readonly targetText: string;
    readonly contextLines: readonly string[];
    readonly sourceEntryInfo?: unknown;
  },
): readonly string[] {
  /**
   Both sides' front matter, where names and handles are declared.
   */
  const frontMatterData = {
    sourceData: sourceDocument.frontMatter
      ?.data,
    targetData: targetDocument.frontMatter
      ?.data,
  };
  return [
    ...collectIdentityLines({
      ...frontMatterData,
      sourceEntryInfo,
    },),
    // THE PRONOUN THE ORIGINAL USES FOR ITS SUBJECT, read off the whole
    // document, so a sheet judging one subjectless sentence knows who it is
    // about (the Toka_ls "they" of 2026-09-02).
    ...sourcePronounLines({ text: sourceText, },),
    ...archiveContributorNameForms({ text: targetText, },)
      .map(function contributorLine(name,): string {
        return `target contributor: ${name}`;
      },),
    // THE NOTES BOTH DOCUMENTS CARRY, footnote definitions and editors'
    // comments, which establish vocabulary for the terms they name (the owner's
    // rule of 2026-09-02) and sit where no slice would show them.
    ...entryNoteLines({
      sourceDocument,
      targetDocument,
    },),
    // THE COMMUNITY'S WORDS AND THE ORDINARY WORDS WHOSE CALQUE READS BADLY
    // (the owner's decisions of 2026-09-09 and 2026-09-25,
    // `rendering-glossary.ts`), so every sheet knows the English the page uses.
    ...glossaryTermLines({ text: sourceText, },),
    // HOW THIS PAGE RENDERS ITS PEOPLE AND LINKED TITLES (class seventy-one,
    // `page-name-glossary.ts`): the archive's link text under a shared href
    // and its signature spellings, so a name inside a paragraph is written
    // as the page writes it everywhere else.
    ...pageNameLines({
      sourceText,
      targetText,
      // A linked title that names the declared person says so (class eighty-six).
      declared: declaredNamePairs(frontMatterData,),
    },),
    ...contextLines,
  ];
}

//endregion Identity context lines
