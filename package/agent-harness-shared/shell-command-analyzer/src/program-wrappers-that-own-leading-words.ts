/**
 Program wrappers whose program word comes after words the wrapper owns.
 
 `flock` takes a lock file, `timeout` a duration,
 `taskset` an affinity mask, `chrt` a priority, and `ssh` a host,
 each before the program those words apply to.
 A reader that treats such a word as the program name stops one word short
 and never reaches the command behind it.
 
 @module
 */

import {
  type CommandWrapper,
  NO_OPTIONS,
} from './command-wrapper-record.ts';

//region Table

/**
 Wrappers that consume a leading positional word of their own.
 */
const PROGRAM_WRAPPERS_THAT_OWN_LEADING_WORDS: readonly CommandWrapper[] = [
  {
    names: new Set(['timeout',],),
    optionsWithValue: new Set([
      '-k',
      '-s',
      '--kill-after',
      '--signal',
    ],),
    payloadOptions: NO_OPTIONS,
    commandStringOptions: NO_OPTIONS,
    setenvOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 1,
  },
  {
    names: new Set(['flock',],),
    optionsWithValue: new Set([
      '-E',
      '-c',
      '-w',
      '--command',
      '--conflict-exit-code',
      '--length',
      '--start',
      '--timeout',
      '--wait',
    ],),
    payloadOptions: NO_OPTIONS,
    commandStringOptions: new Set([
      '-c',
      '--command',
    ],),
    setenvOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 1,
  },
  {
    names: new Set(['ssh',],),
    optionsWithValue: new Set([
      '-B',
      '-D',
      '-E',
      '-F',
      '-I',
      '-J',
      '-L',
      '-O',
      '-P',
      '-R',
      '-S',
      '-W',
      '-b',
      '-c',
      '-e',
      '-i',
      '-l',
      '-m',
      '-o',
      '-p',
      '-Q',
      '-w',
    ],),
    payloadOptions: NO_OPTIONS,
    commandStringOptions: NO_OPTIONS,
    setenvOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 1,
    joinsRemainderAsShellSource: true,
    launchesOnAnotherHost: true,
  },
  {
    names: new Set(['chrt',],),
    optionsWithValue: new Set([
      '-D',
      '-P',
      '-T',
      '--sched-deadline',
      '--sched-period',
      '--sched-runtime',
    ],),
    payloadOptions: NO_OPTIONS,
    commandStringOptions: NO_OPTIONS,
    setenvOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
    numericWordBeforeProgram: true,
  },
  {
    names: new Set(['taskset',],),
    optionsWithValue: NO_OPTIONS,
    payloadOptions: NO_OPTIONS,
    commandStringOptions: NO_OPTIONS,
    setenvOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
    numericWordBeforeProgram: true,
  },
];

//endregion Table

export { PROGRAM_WRAPPERS_THAT_OWN_LEADING_WORDS, };
