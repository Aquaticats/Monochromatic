/**
 Recognizes a file the shell reads as a script.
 
 A shell script is recognized by its name or by what its first line asks for:
 this repository's scripts carry no single extension,
 and a file called `cleanup` with `#!/bin/bash` is read by a shell all the same.
 
 @module
 */

//region Constants

/**
 Extensions whose files a shell reads as scripts.
 */
const SHELL_SCRIPT_EXTENSIONS: ReadonlySet<string> = new Set([
  '.bash',
  '.bats',
  '.dash',
  '.ksh',
  '.sh',
  '.zsh',
]);

/**
 Program names a shebang line can name, after its folder is dropped.
 */
const SHELL_SHEBANG_PROGRAM_NAMES: ReadonlySet<string> = new Set([
  'ash',
  'bash',
  'dash',
  'ksh',
  'mksh',
  'sh',
  'zsh',
]);

/**
 First two characters of a line that names the program reading the file.
 */
const SHEBANG_MARKER = '#!';

/**
 Program that runs the program named after its own options, as `#!/usr/bin/env bash` does.
 */
const ENV_PROGRAM_NAME = 'env';

/**
 Characters that separate the words of a shebang line.
 */
const SHEBANG_WORD_SEPARATORS: ReadonlySet<string> = new Set([
  '\t',
  ' ',
]);

/**
 Character that ends the first line of a file.
 */
const LINE_BREAK = '\n';

/**
 Character that ends a file name's extension.
 */
const EXTENSION_SEPARATOR = '.';

/**
 One path segment separator, which an extension search stops at.
 */
const PATH_SEPARATOR = '/';

/**
 Sentinel returned when a first line names no program at all.
 */
const NO_SHEBANG_PROGRAM: unique symbol = Symbol('first line names no program',);

//endregion Constants

//region Readers

/**
 Reads the first line of a text.
 
 @param text - file text
 
 @returns the first line, without its line break
 
 @example
 ```ts
 firstLineOf('#!/bin/bash\necho hi\n'); // '#!/bin/bash'
 ```
 */
function firstLineOf(text: string,): string {
  /**
   Offset of the first line break, or -1 when the text is one line.
   */
  const lineBreakOffset = text.indexOf(LINE_BREAK,);
  return lineBreakOffset === (-1)
    ? text
    : text.slice(
      0,
      lineBreakOffset,
    );
}

/**
 Splits one shebang line into its words.
 
 @param line - shebang line after its marker
 
 @returns the line's words, empty when it holds none
 
 @example
 ```ts
 shebangWords(' /usr/bin/env  bash '); // ['/usr/bin/env', 'bash']
 ```
 */
function shebangWords(line: string,): readonly string[] {
  return (function readsCharacters(): readonly string[] {
    /**
     Words read so far.
     */
    const words: string[] = [];
    /**
     Characters of the word being read.
     */
    let word = '';
    for (const character of line) {
      if (SHEBANG_WORD_SEPARATORS.has(character,)) {
        if (word.length > 0)
          words.push(word,);
        word = '';
        continue;
      }
      word += character;
    }
    if (word.length > 0)
      words.push(word,);
    return words;
  })();
}

/**
 Reads the program a shebang line asks to read the file.
 
 The program is read through `env` and its options,
 so `#!/usr/bin/env -S bash -e` names bash.
 
 @param text - file text
 
 @returns last path segment of that program, or {@link NO_SHEBANG_PROGRAM} when the first line names none
 
 @example
 ```ts
 shellShebangProgramName('#!/usr/bin/env bash\n'); // 'bash'
 shellShebangProgramName('#!/bin/sh -e\n'); // 'sh'
 ```
 */
function shellShebangProgramName(text: string,): string | typeof NO_SHEBANG_PROGRAM {
  /**
   First line of the file, which carries the shebang when there is one.
   */
  const firstLine = firstLineOf(text,);
  if (!firstLine.startsWith(SHEBANG_MARKER,))
    return NO_SHEBANG_PROGRAM;
  /**
   Words of the shebang line after its marker.
   */
  const words = shebangWords(firstLine
    .slice(SHEBANG_MARKER.length,),);
  /**
   Program named first, which `env` replaces with the one after its options.
   */
  const firstProgram = words.at(0,);
  if (firstProgram === undefined)
    return NO_SHEBANG_PROGRAM;
  if (lastPathSegmentOfProgram(firstProgram,) !== ENV_PROGRAM_NAME)
    return lastPathSegmentOfProgram(firstProgram,);
  for (const word of words.slice(1,)) {
    if (word.startsWith('-',) || word.includes('=',))
      continue;
    return lastPathSegmentOfProgram(word,);
  }
  return NO_SHEBANG_PROGRAM;
}

/**
 Reads the last path segment of one shebang word.
 
 @param word - program word from a shebang line
 
 @returns final path segment, or the whole word when it carries no separator
 
 @example
 ```ts
 lastPathSegmentOfProgram('/usr/bin/bash'); // 'bash'
 ```
 */
function lastPathSegmentOfProgram(word: string,): string {
  /**
   Offset of the last separator, or -1 when the word carries none.
   */
  const separatorOffset = word.lastIndexOf(PATH_SEPARATOR,);
  return separatorOffset === (-1)
    ? word
    : word.slice(separatorOffset + 1,);
}

/**
 Reads the extension of one file path, in lower case.
 
 @param path - file path
 
 @returns extension including its dot, or empty text when the name carries none
 
 @example
 ```ts
 fileExtensionOf('/repo/script.SH'); // '.sh'
 ```
 */
function fileExtensionOf(path: string,): string {
  /**
   Final path segment, whose dot characters are the only ones the extension can use.
   */
  const name = path.slice(path.lastIndexOf(PATH_SEPARATOR,) + 1,);
  /**
   Offset of the extension's dot, or -1 when the name carries none.
   */
  const dotOffset = name.lastIndexOf(EXTENSION_SEPARATOR,);
  return dotOffset <= 0
    ? ''
    : name
      .slice(dotOffset,)
      .toLowerCase();
}

//endregion Readers

//region Predicates

/**
 Reports whether a file is one a shell reads as a script.
 
 @param path - file path
 
 @param text - the file's text, which carries its shebang when it has one
 
 @returns whether the name or the first line makes this a shell script
 
 @example
 ```ts
 isShellScriptFile({ path: '/repo/clean.sh', text: 'echo hi\n' }); // true
 isShellScriptFile({ path: '/repo/cleanup', text: '#!/bin/bash\necho hi\n' }); // true
 ```
 */
function isShellScriptFile({
  path,
  text,
}: {
  readonly path: string;
  readonly text: string;
},): boolean {
  if (SHELL_SCRIPT_EXTENSIONS.has(fileExtensionOf(path,),))
    return true;
  /**
   Program the first line names, when it names one.
   */
  const shebangProgram = shellShebangProgramName(text,);
  return ((typeof shebangProgram) === 'string')
    && SHELL_SHEBANG_PROGRAM_NAMES.has(shebangProgram,);
}

//endregion Predicates

export {
  fileExtensionOf,
  firstLineOf,
  isShellScriptFile,
  NO_SHEBANG_PROGRAM,
  shellShebangProgramName,
};
