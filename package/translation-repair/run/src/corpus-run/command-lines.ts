import { CARD_PROVIDERS, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  EDITOR_FIDELITY_COMMAND_LINES,
  type EditorFidelityCommandLines,
} from './editor-fidelity-command-line.ts';
import { REQUIRABLE_PROVIDERS, } from './requirable-providers.ts';
import type {
  CommandLineFor,
  CommandLineSpec,
  PositionalSpec,
} from './command-line-types.ts';

//region Command lines
// WHAT EVERY RUNNER READS FROM ITS COMMAND LINE, keyed by its build entry
// name (`build-entries.ts`), which is also the name `reportingRefusals` is
// given and the name its refusals start with (ledger B75).
//
// ONE TABLE RATHER THAN A DECLARATION BESIDE EACH RUNNER, for three readers:
// `reportingRefusals` reads a runner's line against its entry before the
// runner starts, the build-entries case holds the table to the runner list so
// no runner goes undeclared, and the mise-description case holds each task's
// description to the flags declared here, so a description cannot offer a
// flag the command refuses or leave out one it reads. A runner module cannot
// export its own declaration for those cases to import, since an entry that
// exports is bundled as a shared chunk and its `import.meta.main` is then
// false (`roster-card-ask.ts`).
//
// THE FLAG NAMES ARE WRITTEN TWICE, in `CommandLines` and in the table, and
// the compiler holds the two to each other key for key: the package builds its
// declarations in isolation, which cannot infer an exported table's type from
// its values. A runner absent from either cannot call `reportingRefusals`,
// since its name is not a key.

/**
 What each runner reads, by flag name: the flags written once with a value,
 those written any number of times, and the switches.

 @example
 ```ts
 const pass: CommandLines['corpus-pass'] = COMMAND_LINES['corpus-pass'];
 ```
 */
export type CommandLines = EditorFidelityCommandLines & {
  readonly 'audit-sensitivity': CommandLineSpec<never, never, never>;
  readonly 'budget-sample': CommandLineSpec<never, never, never>;
  readonly 'cache-account-audit': CommandLineSpec<never, 'runs-under', never>;
  readonly 'cap-census': CommandLineSpec<never, never, never>;
  readonly 'checker-sensitivity': CommandLineSpec<never, never, never>;
  readonly 'corpus-pass': CommandLineSpec<'only' | 'require-providers', never, 'plan'>;
  readonly 'coverage-census': CommandLineSpec<'baseline', 'source', never>;
  readonly 'coverage-control-probe': CommandLineSpec<'only', never, never>;
  readonly 'coverage-probe': CommandLineSpec<'only' | 'cap', never, never>;
  readonly 'damage-sample': CommandLineSpec<never, never, never>;
  readonly 'displacement-probe': CommandLineSpec<never, never, never>;
  readonly 'draw-sample': CommandLineSpec<never, never, 'final'>;
  readonly 'editor-calibrate': CommandLineSpec<'candidates', never, 'candidates-alone'>;
  readonly 'editor-standing-read': CommandLineSpec<never, never, never>;
  readonly 'editor-width-probe': CommandLineSpec<never, never, never>;
  readonly 'judge-fidelity-probe': CommandLineSpec<
    'only' | 'cap' | 'damage' | 'candidates',
    never,
    'context' | 'candidates-alone'
    >;
  readonly 'ledger-report': CommandLineSpec<'model', never, never>;
  readonly 'meter-report': CommandLineSpec<never, never, never>;
  readonly 'model-catalog': CommandLineSpec<never, never, never>;
  readonly 'model-health': CommandLineSpec<never, never, never>;
  readonly 'probe-relabel': CommandLineSpec<never, never, never>;
  readonly 'probe-sensitivity': CommandLineSpec<never, never, never>;
  readonly 'probe-verify': CommandLineSpec<never, never, never>;
  readonly 'producer-calibrate': CommandLineSpec<'candidates', never, 'candidates-alone'>;
  readonly 'reader-measure': CommandLineSpec<'model' | 'sample' | 'pictures', 'trusted', never>;
  readonly 'recall-benchmark': CommandLineSpec<never, never, 'plan'>;
  readonly 'repeat-census': CommandLineSpec<never, never, never>;
  readonly 'rendering-audit-settled': CommandLineSpec<'archive' | 'clone' | 'only' | 'cap', never, never>;
  readonly 'rendering-audit-settled-report': CommandLineSpec<'run' | 'against', never, never>;
  readonly 'roster-bench': CommandLineSpec<never, never, never>;
  readonly 'roster-card': CommandLineSpec<never, never, never>;
  readonly 'run-timing-report': CommandLineSpec<never, never, never>;
  readonly 'score-agreement': CommandLineSpec<'sheet' | 'manifest' | 'pre-grades', never, never>;
  readonly 'score-attribution': CommandLineSpec<never, never, never>;
  readonly 'score-crosscheck': CommandLineSpec<never, never, never>;
  readonly 'score-probe': CommandLineSpec<'repair-sheet' | 'manifest', never, never>;
  readonly 'score-verify': CommandLineSpec<never, never, never>;
  readonly 'sentinel-probe': CommandLineSpec<never, never, never>;
  readonly 'slice-census': CommandLineSpec<never, never, never>;
  readonly 'slice-cost-report': CommandLineSpec<never, never, never>;
  readonly 'spend-report': CommandLineSpec<never, never, never>;
  readonly 'translate-probe': CommandLineSpec<never, never, never>;
  readonly 'verify-published': CommandLineSpec<never, never, never>;
  readonly 'window-trial-probe': CommandLineSpec<never, never, never>;
};

/**
 Positions of a command that reads none.
 */
const NO_POSITIONALS: PositionalSpec = {
  names: [],
  least: 0,
  rest: false,
};

/**
 Declaration of a command that reads nothing from its command line, so any
 argument at all is refused rather than ignored.
 */
const NO_ARGUMENTS: CommandLineSpec<never, never, never> = {
  valued: {},
  repeatable: {},
  switches: {},
  positionals: NO_POSITIONALS,
};

/**
 Positions of a report that reads one or more logs.
 */
const LOG_FILES: PositionalSpec = {
  names: ['log file',],
  least: 1,
  rest: true,
};

/**
 Position of a bench that reads how many slices to run, defaulting when left
 off.
 */
const SLICE_COUNT: PositionalSpec = {
  names: ['slices',],
  least: 0,
  rest: false,
};

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
 What each runner reads from its command line, by build entry name.

 @example
 ```ts
 const spec = COMMAND_LINES['corpus-pass'];
 ```
 */
export const COMMAND_LINES: CommandLines = {
  'audit-sensitivity': NO_ARGUMENTS,
  'budget-sample': NO_ARGUMENTS,
  'cache-account-audit': {
    ...NO_ARGUMENTS,
    repeatable: { 'runs-under': 'directory holding runs directories', },
  },
  'cap-census': {
    ...NO_ARGUMENTS,
    positionals: {
      names: ['log file or directory',],
      least: 1,
      rest: true,
    },
  },
  'checker-sensitivity': NO_ARGUMENTS,
  'corpus-pass': {
    ...NO_ARGUMENTS,
    valued: {
      only: ENTRY_IDS,
      'require-providers': `providers of ${REQUIRABLE_PROVIDERS.join(', ',)}`,
    },
    switches: { plan: true, },
  },
  'coverage-census': {
    ...NO_ARGUMENTS,
    valued: { baseline: 'census.json', },
    repeatable: { source: 'src/file.ts', },
    positionals: {
      names: ['unit test file',],
      least: 0,
      rest: true,
    },
  },
  'coverage-control-probe': {
    ...NO_ARGUMENTS,
    valued: { only: ENTRY_IDS, },
  },
  'coverage-probe': {
    ...NO_ARGUMENTS,
    valued: {
      only: ENTRY_IDS,
      cap: COUNT,
    },
  },
  'damage-sample': NO_ARGUMENTS,
  'displacement-probe': NO_ARGUMENTS,
  'draw-sample': {
    ...NO_ARGUMENTS,
    switches: { final: true, },
  },
  'editor-calibrate': {
    ...NO_ARGUMENTS,
    valued: { candidates: SEATABLE_IDS, },
    switches: { 'candidates-alone': true, },
    positionals: SLICE_COUNT,
  },
  ...EDITOR_FIDELITY_COMMAND_LINES,
  'editor-standing-read': {
    ...NO_ARGUMENTS,
    positionals: {
      names: ['artifact directory',],
      least: 0,
      rest: true,
    },
  },
  'editor-width-probe': {
    ...NO_ARGUMENTS,
    positionals: {
      names: [
        'slices',
        'draw',
      ],
      least: 0,
      rest: false,
    },
  },
  'judge-fidelity-probe': {
    ...NO_ARGUMENTS,
    valued: {
      only: ENTRY_IDS,
      cap: COUNT,
      damage: 'deletion|insertion|alteration',
      candidates: SEATABLE_IDS,
    },
    switches: {
      context: true,
      'candidates-alone': true,
    },
  },
  'ledger-report': {
    ...NO_ARGUMENTS,
    valued: { model: 'seat id', },
  },
  'meter-report': {
    ...NO_ARGUMENTS,
    positionals: LOG_FILES,
  },
  'model-catalog': NO_ARGUMENTS,
  'model-health': NO_ARGUMENTS,
  'probe-relabel': NO_ARGUMENTS,
  'probe-sensitivity': NO_ARGUMENTS,
  'probe-verify': NO_ARGUMENTS,
  'producer-calibrate': {
    ...NO_ARGUMENTS,
    valued: { candidates: SEATABLE_IDS, },
    switches: { 'candidates-alone': true, },
    positionals: SLICE_COUNT,
  },
  'reader-measure': {
    ...NO_ARGUMENTS,
    valued: {
      model: 'seat id',
      sample: COUNT,
      pictures: 'directory holding people directories',
    },
    repeatable: { trusted: 'directory holding runs directories', },
  },
  'recall-benchmark': {
    ...NO_ARGUMENTS,
    switches: { plan: true, },
  },
  'repeat-census': NO_ARGUMENTS,
  'rendering-audit-settled': {
    ...NO_ARGUMENTS,
    valued: {
      archive: 'directory',
      clone: 'directory',
      only: ENTRY_IDS,
      cap: COUNT,
    },
  },
  'rendering-audit-settled-report': {
    ...NO_ARGUMENTS,
    valued: {
      run: 'run file',
      against: 'run file',
    },
  },
  'roster-bench': {
    ...NO_ARGUMENTS,
    positionals: SLICE_COUNT,
  },
  'roster-card': {
    ...NO_ARGUMENTS,
    positionals: {
      names: [
        CARD_PROVIDERS.join('|',),
        'served id',
      ],
      least: 2,
      rest: false,
    },
  },
  'run-timing-report': {
    ...NO_ARGUMENTS,
    positionals: LOG_FILES,
  },
  'score-agreement': {
    ...NO_ARGUMENTS,
    valued: {
      sheet: 'graded sheet',
      manifest: 'sample manifest',
      'pre-grades': 'pre-grades file',
    },
  },
  'score-attribution': NO_ARGUMENTS,
  'score-crosscheck': NO_ARGUMENTS,
  'score-probe': {
    ...NO_ARGUMENTS,
    valued: {
      'repair-sheet': 'graded repair sheet',
      manifest: 'repair manifest',
    },
  },
  'score-verify': NO_ARGUMENTS,
  'sentinel-probe': {
    ...NO_ARGUMENTS,
    positionals: {
      names: ['entry id',],
      least: 0,
      rest: true,
    },
  },
  'slice-census': NO_ARGUMENTS,
  'slice-cost-report': {
    ...NO_ARGUMENTS,
    positionals: {
      names: ['log file',],
      least: 1,
      rest: false,
    },
  },
  'spend-report': {
    ...NO_ARGUMENTS,
    positionals: LOG_FILES,
  },
  'translate-probe': NO_ARGUMENTS,
  'verify-published': NO_ARGUMENTS,
  'window-trial-probe': NO_ARGUMENTS,
};

/**
 Name of a runner, as built and as typed.

 @example
 ```ts
 const command: CommandName = 'corpus-pass';
 ```
 */
export type CommandName = keyof CommandLines;

/**
 The command line a runner's body receives, offering exactly the flags its
 entry declares.

 @example
 ```ts
 async function main({ line, }: { readonly line: CommandLineOf<'corpus-pass'>; },): Promise<void> {}
 ```
 */
export type CommandLineOf<Command extends CommandName> = CommandLineFor<CommandLines[Command]>;

//endregion Command lines
