import {
  allInInputOrder,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type CorpusPin,
  isMissingCorpusObject,
} from './corpus-source.ts';
import { extractDeclaredIdentity, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  carriesHan,
  codePointCount,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Corpus name index
// CLASS SEVENTY-EIGHT (shi_Yumiaoya, 2026-09-22). Two people the page names
// by their handles died in the same month; the corpus holds an entry for
// each, whose front matter declares the English name (`name: Magic Chewing
// Gum`, `alias: Chewing Gum, jjlin1219`; `alias: Danpian`). The page's own
// archive never carried the paragraph, so the class seventy-one glossary
// (`page-name-glossary.ts`) had nothing to read, and three of five
// consolidation candidates transliterated the first handle as
// "Xiaoguantang", a pinyin of nothing, which the judges weighed as "a
// phonetic transliteration" against "Chewing Gum" and chose. Nothing told
// the bench the corpus had already named the person.
//
// EVERY ENTRY'S FRONT MATTER IS READABLE AT THE PIN. The index pairs each
// Han name or alias an entry's original declares with every form its own
// translation declares, and the lines name those the entry's source text
// carries, the entry's own identity left out since the identity block
// already carries it. Evidence the sheets weigh beside the page's own
// renderings, not a floor: measured over the pinned corpus, 岁月, 不存在 and
// 贴贴 are declared handles other pages carry as ordinary words (灰暗岁月里,
// 不存在的真相, 贴贴图), and Chinese prose marks no word boundary to tell a
// mention from a word, so the heading says an ordinary word stays a word
// and the judges decide.

/**
 Longest declared form read as a name, in code points; a longer one is a
 phrase.
 */
const LONGEST_NAME = 24;

/**
 Shortest declared form worth matching, in code points; one Han character
 matches too much ordinary text to name anyone.
 */
const SHORTEST_NAME = 2;

/**
 Separators the corpus joins aliases with: the ASCII comma the corpus itself
 uses and the full-width comma an author may have typed.
 */
const ALIAS_SEPARATORS = [
  ',',
  '，',
];

/**
 Heading of the sheet block.
 */
const HEADING = 'NAMES OF OTHER PEOPLE IN THIS ARCHIVE whose declared handle this text carries, as their own '
  + 'entries render them (where the text means the person, render them as their entry does, never a '
  + 'transliteration; where the handle is an ordinary word here, it is a word and stays translated):';

/**
 One name as an entry's original declares it and every form its translation
 declares.

 @example
 ```ts
 const name: CorpusName = { source: '猫糖', renderings: ['Cat Candy',], entryId: 'gum', };
 ```
 */
export type CorpusName = {
  /**
   Name or alias as the entry's original declares it.
   */
  readonly source: string;

  /**
   Every name and alias the entry's translation declares, in declaration
   order.
   */
  readonly renderings: readonly string[];

  /**
   Entry whose front matter declares the pair.
   */
  readonly entryId: string;
};

/**
 One entry's texts as read at the pin.
 */
type CorpusEntryTexts = {
  /**
   Entry directory name.
   */
  readonly id: string;

  /**
   Whole original document.
   */
  readonly sourceText: string;

  /**
   Whole translation document.
   */
  readonly targetText: string;
};

/**
 Name and alias forms one document's front matter declares, in declaration
 order, each alias list split on the corpus's separators.

 @param text - whole document

 @returns Declared forms, trimmed, empty ones dropped

 @example
 ```ts
 declaredForms({ text: '---\nname: 猫糖\ninfo:\n  alias: 糖糖, 猫\n---\n', },); // ['猫糖', '糖糖', '猫']
 ```
 */
function declaredForms({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Declared identity read off the front matter, empty without one.
   */
  const split = splitFrontMatter({ text, },);
  /**
   Declared identity, empty without front matter.
   */
  const identity = extractDeclaredIdentity({
    data: split.frontMatter
      ?.data,
  },);
  /**
   Declared fields that can carry a name.
   */
  const fields = [
    identity.name,
    identity.alias,
  ];
  return fields.flatMap(function splitField(field,): readonly string[] {
    if (field === undefined)
      return [];
    /**
     Field with every separator made the ASCII comma.
     */
    const unified = ALIAS_SEPARATORS.reduce(
      function unify(
        joined,
        separator,
      ): string {
        return joined.replaceAll(
          separator,
          ',',
        );
      },
      field,
    );
    /**
     Forms between the separators.
     */
    const forms = unified.split(',',);
    return forms
      .map(function trimForm(form,): string {
        return form.trim();
      },)
      .filter(function nonEmpty(form,): boolean {
        return form !== '';
      },);
  },);
}

/**
 Index of every Han name or alias the entries' originals declare against
 the forms their translations declare.

 @param entries - entries with both documents read

 @returns One name per Han form, in entry then declaration order; an entry
 whose translation declares nothing, or whose original declares no Han form,
 contributes none

 @example
 ```ts
 const names = corpusNamesOf({ entries, },);
 ```
 */
export function corpusNamesOf(
  { entries, }: { readonly entries: readonly CorpusEntryTexts[]; },
): readonly CorpusName[] {
  /**
   Every entry's names, before the shared forms are dropped.
   */
  const names = entries.flatMap(function toNames(entry,): readonly CorpusName[] {
    /**
     Forms the translation declares.
     */
    const renderings = declaredForms({ text: entry.targetText, },);
    if (renderings.length === 0)
      return [];
    /**
     Forms already indexed for this entry.
     */
    const seen = new Set<string>();
    /**
     Forms the original declares.
     */
    const sourceForms = declaredForms({ text: entry.sourceText, },);
    return sourceForms.flatMap(function toName(form,): readonly CorpusName[] {
      if (seen.has(form,))
        return [];
      if (!carriesHan({ text: form, },))
        return [];
      /**
       Length in code points.
       */
      const length = codePointCount({ text: form, },);
      if ((length < SHORTEST_NAME) || (length > LONGEST_NAME))
        return [];
      seen.add(form,);
      return [{
        source: form,
        renderings,
        entryId: entry.id,
      },];
    },);
  },);
  // A FORM TWO ENTRIES DECLARE NAMES NOBODY: measured over the pinned corpus,
  // 猫猫 is declared by two entries and carried by 26 of 92 pages as the
  // ordinary word, and the corpus cannot say which person a page means.
  /**
   Entries declaring each form.
   */
  const declaring = new Map<string, number>();
  for (const name of names) {
    declaring.set(
      name.source,
      (declaring.get(name.source,) ?? 0) + 1,
    );
  }
  return names.filter(function oneEntry(name,): boolean {
    return declaring.get(name.source,) === 1;
  },);
}

/**
 Identity-context lines naming the people of other entries this text
 carries by a declared Han form, as their own entries render them: a heading
 and one line per entry, its longest carried form.

 @param text - whole original document of the entry being prepared

 @param names - corpus name index

 @param ownId - entry being prepared, whose own names the identity block
 already carries

 @returns Sheet lines, empty when the text names nobody

 @example
 ```ts
 const lines = corpusNameLines({ text: sourceText, names, ownId: 'gum', },);
 ```
 */
export function corpusNameLines(
  {
    text,
    names,
    ownId,
  }: {
    readonly text: string;
    readonly names: readonly CorpusName[];
    readonly ownId: string;
  },
): readonly string[] {
  /**
   Longest carried form per entry, in first-seen entry order.
   */
  const carried = new Map<string, CorpusName>();
  for (const name of names) {
    if (name.entryId === ownId)
      continue;
    if (!text.includes(name.source,))
      continue;
    /**
     Form already kept for this entry.
     */
    const kept = carried.get(name.entryId,);
    if ((kept === undefined) || (codePointCount({ text: name.source, },) > codePointCount({ text: kept.source, },)))
      carried.set(
        name.entryId,
        name,
      );
  }
  if (carried.size === 0)
    return [];
  return [
    HEADING,
    ...[...carried.values(),].map(function toLine(name,): string {
      /**
       Renderings quoted and joined.
       */
      const quoted = name.renderings
        .map(function quote(rendering,): string {
          return `"${rendering}"`;
        },)
        .join(', ',);
      return `- ${name.source} (entry ${name.entryId}): ${quoted}`;
    },),
  ];
}

/**
 Reads every entry's two documents at the pin and indexes their declared
 names; an entry missing either side contributes none.

 @param pin - corpus checkout and commit

 @param listPeople - entry lister: `listCorpusPeople` in a run, a test's own
 otherwise. REQUIRED, since a default read the clone (ledger M43, X24)

 @param readFile - document reader: `readCorpusFile` in a run, a test's own
 otherwise, required likewise

 @returns Corpus name index

 @throws {@link CorpusReadError} when the listing or a read fails for a
 reason other than the object being absent at the pin

 @example
 ```ts
 const names = await readCorpusNames({ pin, listPeople: listCorpusPeople, readFile: readCorpusFile, },);
 ```
 */
export async function readCorpusNames(
  {
    pin,
    listPeople,
    readFile,
  }: {
    readonly pin: CorpusPin;
    readonly listPeople: (args: { readonly pin: CorpusPin; },) => Promise<readonly string[]>;
    readonly readFile: (args: {
      readonly pin: CorpusPin;
      readonly relPath: string;
    },) => Promise<string>;
  },
): Promise<readonly CorpusName[]> {
  /**
   Entry ids at the pin.
   */
  const ids = await listPeople({ pin, },);
  /**
   Both documents of every entry, absent where either side is not at the pin.
   */
  const entries = await allInInputOrder({
    members: ids.map(async function readEntry(id,): Promise<readonly CorpusEntryTexts[]> {
      try {
        /**
         Both sides read together.
         */
        const [
          sourceText,
          targetText,
        ] = await allInInputOrder({
          members: [
            readFile({
              pin,
              relPath: `people/${id}/page.md`,
            },),
            readFile({
              pin,
              relPath: `people/${id}/page.en.md`,
            },),
          ],
        },);
        return [{
          id,
          sourceText,
          targetText,
        },];
      } catch (error) {
        if (isMissingCorpusObject(error,))
          return [];
        throw error;
      }
    },),
  },);
  return corpusNamesOf({ entries: entries.flat(), },);
}

//endregion Corpus name index
