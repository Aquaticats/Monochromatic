// The folders of the unpacked runtime cache, read as the questions the bundle checks ask of them.
// What: whether a folder name below `<cache>/monochromatic-ide/runtime/` is one build's key folder, and whether a file
// below that runtime folder is a key folder's `last-used` marker, as a path relative to the runtime folder.
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

/**
 The name of the marker file each key folder carries, whose modification time is the folder's last use.
 */
const lastUsedMarkerName = 'last-used';

/**
 The character a key folder's name and the files inside it are divided by.
 */
const pathSeparator = '/';

/**
 Whether `relativePath` is a key folder's `last-used` marker, as a path below the runtime folder.
 *
 Every start renews the marker of its own key folder, so a start that unpacked anything leaves a file below the
 runtime folder that is neither a key folder's name nor one of those markers.
 *
 @param relativePath - the path below the runtime folder to read.
 *
 @returns whether the path is the marker of a key folder.
 */
export const isRuntimeCacheLastUsedMarker = (relativePath: string): boolean => {
  if (relativePath.length !== runtimeCacheKeyLength + pathSeparator.length + lastUsedMarkerName.length) {
    return false;
  }
  if (!relativePath.endsWith(pathSeparator + lastUsedMarkerName)) {
    return false;
  }
  return isRuntimeCacheKeyFolderName(relativePath.slice(0, runtimeCacheKeyLength));
};
