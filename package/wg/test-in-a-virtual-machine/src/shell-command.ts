/**
 Rendering of one command line for a POSIX shell.

 The guest runs every command through `sh -c`, and `mvm exec` joins its
 arguments with single spaces before that shell sees them, so a command is
 carried as one already-quoted string. Everything this package sends into a
 guest that is not a plain word goes through {@link quoteForPosixShell}.

 @module
 */

/**
 Renders one word so a POSIX shell reads it back unchanged.

 A word with no character the shell treats specially is returned as it is, so a
 command line stays readable. Every other word is wrapped in single quotes, with
 each embedded single quote closed, escaped and reopened.

 @param word - text to carry through the shell

 @returns the word as shell input

 @example
 ```ts
 quoteForPosixShell('plain'); // plain
 quoteForPosixShell('two words'); // 'two words'
 quoteForPosixShell("it's"); // 'it'\''s'
 ```
 */
export function quoteForPosixShell(word: string,): string {
  if (word === '')
    return "''";
  /**
   Whether every character of the word is one a shell leaves alone.
   */
  const plain = /^[A-Za-z0-9_./:=@%+-]+$/.test(word,);
  if (plain)
    return word;
  return `'${
    word.replaceAll(
      "'",
      String.raw`'\''`,
    )
  }'`;
}

/**
 Renders one command line from its words.

 @param words - program and its arguments, in order

 @returns the command line a shell runs

 @example
 ```ts
 renderCommandLine({ words: ['apt-get', 'install', '-y'] }); // apt-get install -y
 ```
 */
export function renderCommandLine({ words, }: { readonly words: readonly string[]; },): string {
  return words.map(function quote(word,): string {
    return quoteForPosixShell(word,);
  },)
    .join(' ',);
}

/**
 Renders a command whose variables are set for the program alone and whose
 output is appended, both streams, to one file in the guest.

 @param environment - variables set for the command alone

 @param words - program and its arguments

 @param logPath - guest file both streams are appended to

 @returns the command line a shell runs

 @example
 ```ts
 runLoggedCommand({ environment: {}, words: ['true'], logPath: '/work/logs/a.log' });
 ```
 */
export function runLoggedCommand(
  {
    environment,
    logPath,
    words,
  }: {
    readonly environment: Readonly<Record<string, string>>;
    readonly logPath: string;
    readonly words: readonly string[];
  },
): string {
  /**
   Assignment words that set the variables for the program alone.
   */
  const assignments: string[] = Object.entries(environment,)
    .map(function renderAssignment(
      [
        name,
        value,
      ],
    ): string {
      return `${name}=${quoteForPosixShell(value,)}`;
    },);
  return `${[
    ...assignments,
    ...words,
  ].map(function quote(word,): string {
    return quoteForPosixShell(word,);
  },)
    .join(' ',)} >> ${
    quoteForPosixShell(logPath,)
  } 2>&1`;
}
