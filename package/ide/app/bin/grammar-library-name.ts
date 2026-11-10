// The grammar library names a runtime manifest lists, checked as the build script writes them.
// What: a name of ASCII letters, digits, underscores and hyphens, then the shared-object suffix, and nothing more.
// Why: the bundle inspection validates every entry of runtime/manifest.json before it looks for the library, and did
// so through a pattern the repository lint bans where one walk over the name answers the same question.
// Tested in grammar-library-name.test.ts.
import { isWordCharacter } from './scan-characters.ts';

/** Suffix every grammar library name ends with. */
const librarySuffix = '.so';

/** Character that separates the words of a grammar's name. */
const hyphen = '-';

/**
 Whether a manifest entry names a grammar library.
 
 @param name - One entry of the manifest's grammar list.
 
 @returns Whether the entry is a stem of word characters and hyphens followed by the shared-object suffix.
 
 @example
 ```ts
 isGrammarLibraryName('c-sharp.so'); // true
 isGrammarLibraryName('.so'); // false
 ```
 */
export function isGrammarLibraryName(name: string): boolean {
  if (!name.endsWith(librarySuffix)) return false;
  const stemLength = name.length - librarySuffix.length;
  // A name with no stem before the suffix names no library.
  if (stemLength === 0) return false;
  for (let index = 0; index < stemLength; index++) {
    const character = name[index] ?? '';
    if (!isWordCharacter(character) && character !== hyphen) return false;
  }
  return true;
}
