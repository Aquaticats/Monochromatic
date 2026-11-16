/**
 The class three walk of the fourth review, made from every character Bash treats specially and every place in a word it can stand: a
 reader that guesses is worse than a reader that refuses, so each row must be one of two things and never a third. Either the reader
 gives exactly the value real Bash exports for the same line, or it refuses the line, and a refusal is a stop the run states.

 The characters, and the four places, come from the review's brief: what a shell treats specially, at the start of a word, inside it,
 at its end, and at its end just before a line break.

 @module
 */

import { spawnSync, } from 'node:child_process';
import { mkdtemp, rm, } from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  exportedCredentialValues,
  SecretEnvironmentUnreadableError,
  writtenValue,
} from '../dist/final/node/index.mjs';

/** The variable every row assigns. */
const NAME = 'STAND_IN_SHELL_TOKEN';

/** The word each special character is placed in. */
const WORD = 'madeupsecretvalue';

/** A folder for the shell's own side effects, so a row that is a redirection writes there and never in the package. */
const SHELL_JUNK = await mkdtemp(join(scratchRoot(), 'pi-shell-table-',),);

/**
 Every character Bash treats specially.
 */
const SPECIAL_CHARACTERS: readonly {
  /**
   What the character is called, so a failing row names it.
   */
  readonly called: string;

  /**
   The character itself.
   */
  readonly character: string;
}[] = [
  { called: 'a backslash', character: '\\', },
  { called: 'a single quote', character: '\'', },
  { called: 'a double quote', character: '"', },
  { called: 'a dollar sign', character: '$', },
  { called: 'a backtick', character: '`', },
  { called: 'a semicolon', character: ';', },
  { called: 'an ampersand', character: '&', },
  { called: 'a pipe', character: '|', },
  { called: 'a less-than sign', character: '<', },
  { called: 'a greater-than sign', character: '>', },
  { called: 'an opening parenthesis', character: '(', },
  { called: 'a closing parenthesis', character: ')', },
  { called: 'a hash', character: '#', },
  { called: 'a space', character: ' ', },
  { called: 'a tab', character: '\t', },
  { called: 'a line break', character: '\n', },
  { called: 'an exclamation mark', character: '!', },
  { called: 'an asterisk', character: '*', },
  { called: 'a question mark', character: '?', },
  { called: 'an opening bracket', character: '[', },
  { called: 'a tilde', character: '~', },
  { called: 'an opening brace', character: '{', },
];

/**
 The four places a character can stand in a word, each with how the row writes it. The last place puts the character at the end of
 the word and a line break after it, which is where a continuation would begin.
 */
const PLACES: readonly {
  /**
   What the place is called.
   */
  readonly called: string;

  /**
   The line the file holds, less the `export NAME=` part.

   @param character - the special character

   @returns what the assignment writes
   */
  readonly writes: (params: { readonly character: string }) => string;
}[] = [
  {
    called: 'first',
    writes: function first({ character, },) {
      return `${character}${WORD}`;
    },
  },
  {
    called: 'inside',
    writes: function inside({ character, },) {
      return `madeup${character}secretvalue`;
    },
  },
  {
    called: 'last',
    writes: function last({ character, },) {
      return `${WORD}${character}`;
    },
  },
  {
    called: 'last before a line break',
    writes: function lastBeforeALineBreak({ character, },) {
      return `${WORD}${character}\nnextline`;
    },
  },
];

/**
 What real Bash exports for one assignment line, and whether it accepted the line at all.

 @param written - the text after the `=`

 @returns what Bash exported, or that Bash refused the line
 */
function whatBashDoes({ written, }: { readonly written: string },): { readonly exported: string } | {
  readonly refused: true
} {
  /**
   A real shell, run with no environment of its own, printing what it exported.
   */
  // A blank line closes the assignment, so a line the row continues cannot swallow the print: a continuation removes exactly one
  // line break, and the blank line is the one after it. The closing `true` makes the status that of the assignment alone, so a
  // following line the row's own text all but continues is not mistaken for the shell refusing the assignment.
  const shell = spawnSync('bash', ['--noprofile', '--norc', '-c', `export ${NAME}=${written}\n\nprintf %s "$${NAME}"\ntrue`,], {
    encoding: 'utf8',
    cwd: SHELL_JUNK,
    env: {},
  },);
  // Bash's own exit status is the only thing that says whether it read the line: a line it refuses is a line no reader has to match.
  return (shell.status === 0) ? { exported: shell.stdout, } : { refused: true, };
}

await describe({
  name: 'a shell value is what the shell exports, or a refusal that stops the run',
  children: [
    ...SPECIAL_CHARACTERS.flatMap(function eachCharacter(character,) {
      return PLACES.map(function eachPlace(place,) {
        return it({
          name: `${character.called} ${place.called} is either the value Bash exports or a line this reader refuses`,
          fn: async () => {
            /**
             The line's text after the `=`.
             */
            const written = place.writes({ character: character.character, },);
            /**
             What Bash made of it.
             */
            const bash = whatBashDoes({ written, },);
            /**
             What this reader made of the same line.
             */
            const read = writtenValue({ written, },);
            /**
             What reading the file that holds the line arms, which is the whole of the reader's contract: a refused line throws.
             */
            function armedOfTheFile() {
              return exportedCredentialValues({
                text: `export ${NAME}=${written}\n`,
                fileName: '.bashrc',
                environment: {},
              },);
            }
            if ('value' in read) {
              // A value this reader gives must be exactly the shell's own, and a line the shell refused has no value to match.
              expect(bash,).toHaveProperty('exported',);
              expect(read.value,).toBe(('exported' in bash) ? bash.exported : undefined,);
            }
            else {
              // A line it does not read is one it states, and a line it reads is a line the shell also read.
              expect(read,).not.toHaveProperty('value',);
            }
            let armed;
            try {
              armed = armedOfTheFile();
            }
            catch (error) {
              // The run stops and says which file and line, which is the refusal this reader gives.
              expect(error,).toBeInstanceOf(SecretEnvironmentUnreadableError,);
              return;
            }
            // Nothing armed is a refusal. Anything armed must be exactly what the shell holds, because a value the shell never held
            // would be masked in place of the one it did.
            for (const value of armed) {
              if ('exported' in bash) {
                expect(value.value,).toBe(bash.exported,);
                continue;
              }
              // A shell that refused the line held no value this code can compare with, so only a value too short to mask, or none,
              // is acceptable there.
              expect(value.value.length,).toBeLessThan(8,);
            }
          },
        },);
      },);
    },),
    it({
      name: 'still reads the spellings a start-up file really writes, so refusing is not the only answer it has',
      fn: async () => {
        for (const written of [(WORD), `"${WORD}"`, `'${WORD}'`,]) {
          /**
           What Bash exports for that spelling.
           */
          const bash = whatBashDoes({ written, },);
          expect(bash,).toEqual({ exported: WORD, },);
          expect(writtenValue({ written, },),).toEqual({ value: WORD, },);
        }
      },
    },),
  ],
},);

// The shell's own side effects go with the folder, so a row that redirects leaves nothing behind.
await rm(SHELL_JUNK, { recursive: true, force: true, },);
