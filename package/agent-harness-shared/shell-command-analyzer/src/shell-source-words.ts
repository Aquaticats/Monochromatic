/**
 Spells command words back as shell source.
 
 A guard that reads a command line another program assembles needs every word
 to survive that second reading unchanged,
 so each word is quoted for the grammar that will parse it again.
 The words come from the parser with their quotes already removed,
 so this is the only spelling that keeps their values.
 
 @module
 */

//region Quoting

/**
 Spells one word so re-reading it as shell source yields the same value.
 
 @param word - argument value after shell quote removal
 
 @returns shell source spelling of that one word
 
 @example
 ```ts
 shellQuotedWord(`it's`); // `'it'\\''s'`
 ```
 */
function shellQuotedWord(word: string,): string {
  /**
   The word with every embedded quote closed, escaped, and reopened.
   */
  const escaped = word.replaceAll(
    `'`,
    `'\\''`,
  );
  return `'${escaped}'`;
}

/**
 Spells a program and its arguments as one shell source line.
 
 The line names exactly those words and runs exactly that program:
 quoting leaves no expansion, redirection, or operator for the shell to read.
 
 @param words - program name followed by its arguments
 
 @returns shell source line naming those words
 
 @example
 ```ts
 shellSourceLine(['/usr/bin/git', 'commit', '-m', 'a b']);
 // `'/usr/bin/git' 'commit' '-m' 'a b'`
 ```
 */
function shellSourceLine(words: readonly string[],): string {
  return words
    .map(function quotedWord(word,): string {
      return shellQuotedWord(word,);
    },)
    .join(' ',);
}

//endregion Quoting

export {
  shellQuotedWord,
  shellSourceLine,
};
