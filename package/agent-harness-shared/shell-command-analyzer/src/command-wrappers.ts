/**
 Programs that run another program named in their arguments.
 
 A guard that reads only the command name misses a mutation hidden behind
 one of these,
 so the table says which words of a wrapper belong to the wrapper
 and which ones hand a program to it.
 
 The entries live in two files, one per shape of wrapper:
 `program-wrappers-that-own-only-options.ts` holds the wrappers whose words
 before the program are their own options,
 and `program-wrappers-that-own-leading-words.ts` holds the ones that also
 consume a positional word of their own first.
 
 @module
 */

import type { CommandWrapper, } from './command-wrapper-record.ts';
import { PROGRAM_WRAPPERS_THAT_OWN_LEADING_WORDS, } from './program-wrappers-that-own-leading-words.ts';
import { PROGRAM_WRAPPERS_THAT_OWN_ONLY_OPTIONS, } from './program-wrappers-that-own-only-options.ts';

//region Wrapper table

/**
 Wrappers that agent command lines use to launch another program.
 */
const COMMAND_WRAPPERS: readonly CommandWrapper[] = [
  ...PROGRAM_WRAPPERS_THAT_OWN_ONLY_OPTIONS,
  ...PROGRAM_WRAPPERS_THAT_OWN_LEADING_WORDS,
];

//endregion Wrapper table

export { COMMAND_WRAPPERS, };
