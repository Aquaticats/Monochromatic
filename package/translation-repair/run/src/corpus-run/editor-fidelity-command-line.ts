import type { CommandLineSpec, } from './command-line-types.ts';

//region Editor fidelity command lines
// THE COMMAND LINES OF THE EDITOR FIDELITY RUNNERS, kept beside the table (`command-lines.ts`) which spreads them in, so
// the table stays under the file-length cap. The table's compile-time check still holds them to the runner list key for
// key. The value names are the ones the table gives the same flags in the other runners.

/**
 What the entry filter's value is, as every usage line names it.
 */
const ENTRY_IDS = 'entry ids';

/**
 What a cap's value is.
 */
const COUNT = 'count';

/**
 What a seatable-candidate list's value is.
 */
const SEATABLE_IDS = 'seatable ids';

/**
 What the editor fidelity runners read, by flag name.

 @example
 ```ts
 const pool: EditorFidelityCommandLines['editor-fidelity-pool-draw'] = EDITOR_FIDELITY_COMMAND_LINES['editor-fidelity-pool-draw'];
 ```
 */
export type EditorFidelityCommandLines = {
  readonly 'editor-fidelity-pool-draw': CommandLineSpec<'seed' | 'want' | 'out', never, 'check'>;
  readonly 'editor-fidelity-read': CommandLineSpec<never, never, never>;
  readonly 'editor-fidelity-probe': CommandLineSpec<
    'only' | 'cap' | 'damage' | 'candidates' | 'max-usd' | 'bench-spent-usd' | 'trials' | 'passes',
    never,
    'candidates-alone'
    >;
};

/**
 What each editor fidelity runner reads from its command line, by build entry name.

 @example
 ```ts
 const spec = EDITOR_FIDELITY_COMMAND_LINES['editor-fidelity-probe'];
 ```
 */
export const EDITOR_FIDELITY_COMMAND_LINES: EditorFidelityCommandLines = {
  'editor-fidelity-pool-draw': {
    valued: {
      seed: 'seed',
      want: COUNT,
      out: 'directory',
    },
    repeatable: {},
    switches: { check: true, },
    positionals: {
      names: [],
      least: 0,
      rest: false,
    },
  },
  'editor-fidelity-read': {
    valued: {},
    repeatable: {},
    switches: {},
    positionals: {
      names: ['arm log',],
      least: 1,
      rest: true,
    },
  },
  'editor-fidelity-probe': {
    valued: {
      only: ENTRY_IDS,
      cap: COUNT,
      damage: 'insertion|alteration',
      candidates: SEATABLE_IDS,
      'max-usd': 'dollars',
      'bench-spent-usd': 'dollars',
      trials: COUNT,
      passes: COUNT,
    },
    repeatable: {},
    switches: { 'candidates-alone': true, },
    positionals: {
      names: [],
      least: 0,
      rest: false,
    },
  },
};

//endregion Editor fidelity command lines
