import { pinyin, } from 'pinyin-pro';

import {
  type FrontMatterBlock,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isAsciiLetter, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Directory id as a name
// `directory-id-name` refuses a page whose visible name is the directory id
// while the source's is not, written for archives whose metadata was never
// translated and still name the folder. On 2026-09-07 it refused the Huasheng
// page: the source names the person 椛笙, whose pinyin is Huasheng, which is
// the directory id, and the judges had chosen it by the identity rule. Of the
// 22 archives naming their directory, 7 do so because the id is a rendering
// of the name (Huasheng, lintong, Kotori, MioCardMeow, MocaKawai, noname,
// donotexist_A) and 7 because the handle stands where the name goes.
//
// The owner's decision of 2026-09-07 (`doc/decision/translation-repair-front-
// matter-guard.md`, addendum): the pinyin check and the alias exemption, read
// on both the original Chinese and the original English front matter, since
// there has to be an English rendering of the name in the front matter. So the
// id-equal name stands when the id is a pinyin reading of the source's name,
// when the source itself carries the id among its aliases, or when the page or
// the archive carries a Latin-script alias other than the id.

/**
 Separators an alias list is written with in the corpus: the comma, the
 full-width comma and the enumeration comma.

 MORE THAN `ALIAS_SEPARATOR` (`front-matter-translation.ts`) splits a
 translation's alias on, the comma alone: this reads originals too, which
 write the other two (ledger B97).
 */
const ALIAS_SEPARATORS = [
  ',',
  '，',
  '、',
];

/**
 Lower-cases a rendering and keeps its Latin letters only, so `Lin Tong`,
 `lintong` and `Lin-Tong` read the same and a name with no Latin letter
 reads as nothing.

 @param text - rendering to normalise

 @returns Its Latin letters, lower-cased

 @example
 ```ts
 latinLettersOf({ text: 'Lan Gou (blue dog)', },);
 // => 'langoubluedog'
 ```
 */
function latinLettersOf({ text, }: { readonly text: string; },): string {
  /**
   Text lower-cased, scanned by UTF-16 unit since only ASCII is kept.
   */
  const lowered = text.toLowerCase();

  /**
   Letters kept so far.
   */
  const kept: string[] = [];
  for (let index = 0; index < lowered.length; index += 1) {
    /**
     Unit under the cursor.
     */
    const character = lowered.charAt(index,);
    if (isAsciiLetter({ character, },))
      kept.push(character,);
  }
  return kept.join('',);
}

/**
 Reads the `name` of a front matter block, empty when absent or not a string.

 @param metadata - parsed front matter

 @returns Declared name

 @example
 ```ts
 nameOf({ metadata, },);
 ```
 */
function nameOf({ metadata, }: { readonly metadata: FrontMatterBlock; },): string {
  /**
   Parsed YAML, unknown until proven a record with a name.
   */
  const { data, } = metadata;
  if (!isJsonRecord(data,))
    return '';

  /**
   Whatever the YAML put under `name`.
   */
  const { name, } = data;
  return ((typeof name) === 'string') ? name : '';
}

/**
 Whether metadata still shows the directory id where a person's name goes.

 @param metadata - parsed front matter block

 @param entryId - directory id of the entry

 @returns Whether the visible name is the directory id

 @example
 ```ts
 namesDirectoryId({ metadata, entryId: 'Cat', },);
 ```
 */
export function namesDirectoryId(
  {
    metadata,
    entryId,
  }: {
    readonly metadata: FrontMatterBlock;
    readonly entryId: string;
  },
): boolean {
  return nameOf({ metadata, },) === entryId;
}

/**
 Reads the aliases of a front matter block, split on every separator the
 corpus uses and trimmed; empty when there are none.

 @param metadata - parsed front matter

 @returns Alias renderings in declared order

 @example
 ```ts
 aliasesOf({ metadata, },);
 ```
 */
function aliasesOf({ metadata, }: { readonly metadata: FrontMatterBlock; },): readonly string[] {
  /**
   Parsed YAML, unknown until proven a record with an alias.
   */
  const { data, } = metadata;
  if (!isJsonRecord(data,))
    return [];

  /**
   Whatever the YAML put under `info`.
   */
  const { info, } = data;
  if (!isJsonRecord(info,))
    return [];

  /**
   Whatever the YAML put under `alias`: a string in the corpus, a list in
   principle.
   */
  const { alias, } = info;

  /**
   Alias text joined on the plain comma when a list was given.
   */
  const joined = ((typeof alias) === 'string')
    ? alias
    : (Array.isArray(alias,)
      ? alias
        .map(String,)
        .join(',',)
      : '');

  return ALIAS_SEPARATORS
    .reduce<readonly string[]>(
    function splitOn(
      pieces,
      separator,
    ): readonly string[] {
      return pieces.flatMap(function split(piece,): readonly string[] {
        return piece.split(separator,);
      },);
    },
    [joined,],
    )
    .map(function trim(piece,): string {
      return piece.trim();
    },)
    .filter(function nonEmpty(piece,): boolean {
      return piece !== '';
    },);
}

/**
 Whether a Latin string is a pinyin reading of a name, every heteronym of
 every character allowed.

 Walks the name one character at a time keeping the set of positions in the
 string each reading could have reached, so the check is linear in the name
 and never enumerates the readings' product.

 @param name - name in the source script

 @param letters - Latin letters, lower-cased, tone marks absent

 @returns Whether some reading of the whole name spells exactly the letters

 @example
 ```ts
 readsAs({ name: '林童', letters: 'lintong', },);
 ```
 */
function readsAs(
  {
    name,
    letters,
  }: {
    readonly name: string;
    readonly letters: string;
  },
): boolean {
  if (letters === '')
    return false;

  return (function walk(): boolean {
    /**
     Positions in the letters some reading of the characters so far ends at.
     */
    let positions: readonly number[] = [0,];
    for (const character of name) {
      /**
       Every reading pinyin-pro knows for this character; a character it
       cannot read (punctuation, a Latin letter, a kana) yields itself, which
       spells nothing in the letters and so ends the walk here.
       */
      const readings = pinyin(
        character,
        {
          toneType: 'none',
          type: 'array',
          multiple: true,
        },
      )
        .map(function lettersOnly(reading,): string {
          return latinLettersOf({ text: reading, },);
        },);
      positions = [
        ...new Set(
          positions.flatMap(function advance(position,): readonly number[] {
            return readings
              .filter(function fits(reading,): boolean {
                return (reading !== '') && letters.startsWith(
                  reading,
                  position,
                );
              },)
              .map(function past(reading,): number {
                return position + reading.length;
              },);
          },),
        ),
      ];
      if (positions.length === 0)
        return false;
    }
    return positions.includes(letters.length,);
  })();
}

/**
 Whether a page may keep the directory id as its visible name although the
 source names the person otherwise.

 @param entryId - directory id

 @param source - original's front matter

 @param page - would-ship page's front matter

 @param archives - archive's front matter, the original English, as a list
 that is empty for a source-only insertion

 @returns Whether the id is a rendering of the name, the source's own alias,
 or stands beside an English rendering

 @example
 ```ts
 directoryIdNameStands({ entryId: 'Huasheng', source, page, archives: [archive,], },);
 ```
 */
export function directoryIdNameStands(
  {
    entryId,
    source,
    page,
    archives,
  }: {
    readonly entryId: string;
    readonly source: FrontMatterBlock;
    readonly page: FrontMatterBlock;
    readonly archives: readonly FrontMatterBlock[];
  },
): boolean {
  /**
   Directory id as letters.
   */
  const idLetters = latinLettersOf({ text: entryId, },);

  /**
   Whether the source itself carries the id among its aliases, so the handle
   is the person's own: the alias is the id character for character, or
   spells the id's Latin letters. An id with no Latin letter is spelled by no
   alias; comparing letters alone took any alias without one for it, a Han
   alias for an id of digits (ledger B131).
   */
  const sourceAliasIsId = aliasesOf({ metadata: source, },)
    .some(function isId(alias,): boolean {
      return (alias === entryId)
        || ((idLetters !== '') && (latinLettersOf({ text: alias, },) === idLetters));
    },);
  if (sourceAliasIsId)
    return true;

  // THE PINYIN CHECK: the id spells the source's name.
  if (readsAs({
    name: nameOf({ metadata: source, },),
    letters: idLetters,
  },))
    return true;

  // THE ALIAS EXEMPTION: an English rendering other than the id stands in the
  // page's or the archive's front matter.
  return [
    ...aliasesOf({ metadata: page, },),
    ...archives.flatMap(function aliases(archive,): readonly string[] {
      return aliasesOf({ metadata: archive, },);
    },),
  ].some(function isOtherRendering(alias,): boolean {
    /**
     Alias as letters, empty for an alias in another script.
     */
    const letters = latinLettersOf({ text: alias, },);
    return (letters !== '') && (letters !== idLetters);
  },);
}

//endregion Directory id as a name
