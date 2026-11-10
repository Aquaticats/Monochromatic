// The folders of the unpacked runtime cache, read as the questions the bundle checks ask of them.
// What: whether a folder name below `<cache>/monochromatic-ide/runtime/` is one build's key folder, and whether a file
// below that runtime folder is a key folder's `last-used` marker.
// Why: each build unpacks its parsers into a folder named by a digest of its embedded files, and the application's own
// retention (src/runtime/retention.rs, `is_key`) removes only folders of that shape and nothing else. A check that
// read a folder of another shape as this build's cache would look for the parsers where they never are, and one that
// counted another build's marker file as an unpacked parser would report a cache holding files nobody wrote.
// The predicates are one pass over the characters and are tested in runtime-cache-key.test.ts.

/**
 The number of hexadecimal digits a key folder's name is made of.
 */
const runtimeCacheKeyLength = 16;

/**
 Whether a character is one of the lowercase hexadecimal digits a key is written with.
 *
 Upper-case digits are refused: the digest is written in lower case, so a folder named in upper case is not one this
 application created, exactly as the retention refuses it.
 *
 @param character - one character of the name.
 *
 @returns whether the character is in `0` to `9` or `a` to `f`.
 */
const isLowercaseHexDigit = (character: string): boolean => ((character >= '0') && (character <= '9'))
  || ((character >= 'a') && (character <= 'f'));

/**
 Whether `name` is the name of one build's key folder below the runtime cache: exactly 16 lowercase hexadecimal digits.
 *
 @param name - the folder name to read.
 *
 @returns whether the name has the shape this application gives a key folder.
 */
export const isRuntimeCacheKeyFolderName = (name: string): boolean => {
  if (name.length !== runtimeCacheKeyLength) {
    return false;
  }
  for (const character of name) {
    if (!isLowercaseHexDigit(character)) {
      return false;
    }
  }
  return true;
};
