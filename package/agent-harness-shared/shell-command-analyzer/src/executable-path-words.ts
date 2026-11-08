/**
 Reads one shell command word as an executable path.
 
 Command words and option values share the same quoting rules,
 so path shape is decided from the parsed word value alone.
 
 @module
 */

//region Constants

/**
 Path separators accepted on any host.
 
 Both are read so a Windows-style path stays one word while the guard runs on Linux.
 */
const PATH_SEPARATORS: readonly string[] = [
  '/',
  '\\',
];

//endregion Constants

//region Path reads

/**
 Reads the offset of the last path separator in a word.
 
 @param word - command or option word
 
 @returns offset of the last `/` or `\\`, or -1 when the word carries neither
 
 @example
 ```ts
 lastPathSeparatorIndex('/usr/bin/git'); // 8
 ```
 */
function lastPathSeparatorIndex(word: string,): number {
  /**
   Offset of the last forward slash, or -1 when absent.
   */
  const slashIndex = word.lastIndexOf('/',);
  /**
   Offset of the last backslash, or -1 when absent.
   */
  const backslashIndex = word.lastIndexOf('\\',);
  return Math.max(
    slashIndex,
    backslashIndex,
  );
}

/**
 Reads the last path segment of a command word.
 
 @param word - command or option word
 
 @returns final path segment, or the whole word when it carries no separator
 
 @example
 ```ts
 lastPathSegment('/usr/bin/git'); // 'git'
 ```
 */
function lastPathSegment(word: string,): string {
  /**
   Offset of the last path separator, or -1 when the word carries none.
   */
  const separatorIndex = lastPathSeparatorIndex(word,);
  return separatorIndex === (-1)
    ? word
    : word.slice(separatorIndex + 1,);
}

/**
 Reports whether a word spells a path instead of a bare name looked up on PATH.
 
 @param word - command word to inspect
 
 @returns whether the word carries a path separator
 
 @example
 ```ts
 looksLikeExecutablePath('./git'); // true
 looksLikeExecutablePath('git'); // false
 ```
 */
function looksLikeExecutablePath(word: string,): boolean {
  return PATH_SEPARATORS
    .some(function carriesSeparator(separator,): boolean {
      return word.includes(separator,);
    },);
}

/**
 Normalizes one executable name for path comparison.
 
 @param name - executable name or path word
 
 @param platform - runtime platform controlling case sensitivity
 
 @returns comparable executable name
 
 @example
 ```ts
 normalizeExecutableName({ name: 'GIT.EXE', platform: 'win32' }); // 'git.exe'
 ```
 */
function normalizeExecutableName({
  name,
  platform,
}: {
  readonly name: string;
  readonly platform: NodeJS.Platform;
},): string {
  return platform === 'win32'
    ? name.toLowerCase()
    : name;
}

//endregion Path reads

export {
  lastPathSegment,
  looksLikeExecutablePath,
  normalizeExecutableName,
};
