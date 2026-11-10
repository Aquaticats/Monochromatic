/**
 The words a landing is asked for: one command line, its subcommands, and the options each of them takes.

 Results are text on standard output, warnings on standard error, and misuse is status 2, as
 `doc/decision/cli-output-and-exit-codes.md` decides.

 @module
 */

import { parseArgs, } from 'node:util';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { CheckKind, } from './check-a-branch.ts';
import {
  SIDES_TO_KEEP,
  type SideToKeep,
} from './conflict-regions.ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  type Bounds,
  boundsOf,
  requiredTextOption,
  textOption,
  textsOption,
} from './options.ts';
import {
  PROGRAM_NAME,
  USAGE,
} from './usage.ts';

export type { Bounds, } from './options.ts';

/**
 What one command line asks for.
 */
export type Invocation =
  /**
   Land one branch on `main`.
   */
  | {
    readonly command: 'land';
    readonly branch: string;
    readonly message: string;
    readonly mainCheckout?: string;
    readonly bounds: Bounds;
  }
  /**
   Check what an agent's worktree looks like combined with `main`.
   */
    | {
      readonly command: 'check';
      readonly worktree: string;
      readonly check: CheckKind;
      readonly tasks: readonly string[];
      readonly mainCheckout?: string;
      readonly bounds: Bounds;
    }
  /**
   Land each branch a list file names.
   */
    | {
      readonly command: 'list';
      readonly list: string;
      readonly only: readonly string[];
      readonly mainCheckout?: string;
      readonly bounds: Bounds;
    }
  /**
   Commit the merge open in a checkout.
   */
    | {
      readonly command: 'commit-open-merge';
      readonly message: string;
      readonly worktree: string;
      readonly paths: readonly string[];
      readonly mainCheckout?: string;
      readonly bounds: Bounds;
    }
  /**
   Settle a merge's conflicts by keeping one side, or describe them.
   */
    | {
      readonly command: 'resolve-conflicts';
      readonly worktree: string;
      readonly side: SideToKeep;
      readonly files: readonly string[];
      readonly mainCheckout?: string;
      readonly bounds: Bounds;
    }
  /**
   Report on every worktree.
   */
    | {
      readonly command: 'worktrees';
      readonly mainCheckout?: string;
      readonly bounds: Bounds;
    }
  /**
   Remove the worktrees nothing would be lost by removing.
   */
    | {
      readonly command: 'remove-worktrees';
      readonly apply: boolean;
      readonly mainCheckout?: string;
      readonly bounds: Bounds;
    }
  /**
   What the caller asked to read.
   */
    | {
      readonly command: 'help';
      readonly text: string
    };

/**
 One option a subcommand takes.
 */
type OptionShape = {
  readonly type: 'string' | 'boolean';
  readonly multiple?: boolean;
};

/**
 The options every subcommand takes.
 */
const SHARED_OPTIONS: Readonly<Record<string, OptionShape>> = {
  'main-checkout': { type: 'string', },
  'git-timeout-seconds': { type: 'string', },
  'commit-tries': { type: 'string', },
  'commit-pause-seconds': { type: 'string', },
  'commit-timeout-seconds': { type: 'string', },
  'help': { type: 'boolean', },
};

/**
 The options each subcommand adds to the shared ones, so a word it does not take is refused instead of ignored.
 */
const OWN_OPTIONS: Readonly<Record<string, Readonly<Record<string, OptionShape>>>> = {
  'land': {
    'branch': { type: 'string', },
    'message': { type: 'string', },
  },
  'check': {
    'worktree': { type: 'string', },
    'check': { type: 'string', },
    'task': {
      type: 'string',
      multiple: true,
    },
    'filter': {
      type: 'string',
      multiple: true,
    },
  },
  'list': {
    'list': { type: 'string', },
    'only': {
      type: 'string',
      multiple: true,
    },
  },
  'commit-open-merge': {
    'message': { type: 'string', },
    'worktree': { type: 'string', },
    'path': {
      type: 'string',
      multiple: true,
    },
  },
  'resolve-conflicts': {
    'worktree': { type: 'string', },
    'side': { type: 'string', },
    'path': {
      type: 'string',
      multiple: true,
    },
  },
  'worktrees': {},
  'remove-worktrees': {
    'apply': { type: 'boolean', },
  },
};

/**
 Reads the subcommand a command line names.

 @param args - the words after the program's name

 @returns what the command line asks for

 @throws LandingConfigurationError when the subcommand or one of its options is wrong

 @example
 ```ts
 const invocation = parseCommandLine({ args: ['land', '--branch=fix/a', '--message=m',], },);
 ```
 */
export function parseCommandLine({ args, }: {
  readonly args: readonly string[];
},): Invocation {
  /**
   The subcommand, which is the first word.
   */
  const [name, ...rest] = args;
  if ((name === undefined) || (name === '--help')
    || (name === '-h')) {
    return {
      command: 'help',
      text: USAGE,
    };
  }
  /**
   The parsed options, which refuse anything the subcommand does not take.
   */
  const values = parseOptions({
    name,
    rest,
  },);
  if (values.help === true) {
    return {
      command: 'help',
      text: USAGE,
    };
  }
  return invocationOf({
    name,
    values,
  },);
}

/**
 Parses a subcommand's options, refusing any it does not take.

 @param name - the subcommand

 @param rest - the words after it

 @returns the parsed values

 @throws LandingConfigurationError when a word is not an option the subcommand takes

 @example
 ```ts
 const values = parseOptions({ name: 'land', rest: ['--branch=fix/a',], },);
 ```
 */
function parseOptions({
  name,
  rest,
}: {
  readonly name: string;
  readonly rest: readonly string[];
},): Readonly<Record<string, unknown>> {
  /**
   The options this subcommand takes, which a subcommand itself is checked against.
   */
  const own = OWN_OPTIONS[name];
  if (own === undefined) {
    throw new LandingConfigurationError(`${name}: there is no such subcommand; run ${PROGRAM_NAME} --help`,);
  }
  try {
    return parseArgs({
      args: [...rest,],
      options: {
        ...SHARED_OPTIONS,
        ...own,
      },
      strict: true,
      allowPositionals: false,
    },)
      .values as Readonly<Record<string, unknown>>;
  }
  catch (caught) {
    throw new LandingConfigurationError(
      `${name}: ${caughtValueText(caught,)}; run ${PROGRAM_NAME} --help`,
    );
  }
}

/**
 Builds what one subcommand asks for from its options.

 @param name - the subcommand

 @param values - its parsed options

 @returns the invocation

 @throws LandingConfigurationError when a value the subcommand needs is missing or wrong

 @example
 ```ts
 const invocation = invocationOf({ name: 'list', values, },);
 ```
 */
function invocationOf({
  name,
  values,
}: {
  readonly name: string;
  readonly values: Readonly<Record<string, unknown>>;
},): Invocation {
  /**
   The checkout main is landed on, when the caller names one.
   */
  const mainCheckout = textOption({
    values,
    name: 'main-checkout',
  },);
  /**
   What bounds this run's commands.
   */
  const bounds = boundsOf({ values, },);
  /**
   The fields every invocation carries.
   */
  const shared = {
    bounds,
    ...(mainCheckout.found ? { mainCheckout: mainCheckout.value, } : {}),
  };
  if (name === 'land') {
    return {
      command: 'land',
      branch: requiredTextOption({
        values,
        name: 'branch',
      },),
      message: requiredTextOption({
        values,
        name: 'message',
      },),
      ...shared,
    };
  }
  if (name === 'check') {
    return {
      command: 'check',
      worktree: requiredTextOption({
        values,
        name: 'worktree',
      },),
      check: checkKindOf(requiredTextOption({
        values,
        name: 'check',
      },),),
      tasks: textsOption({
        values,
        name: 'task',
      },),
      ...shared,
    };
  }
  if (name === 'resolve-conflicts') {
    /**
     The side kept when a path names none.
     */
    const side = textOption({
      values,
      name: 'side',
    },);
    return {
      command: 'resolve-conflicts',
      worktree: requiredTextOption({
        values,
        name: 'worktree',
      },),
      side: parseSideToKeep(side.found ? side.value : 'incoming',),
      files: textsOption({
        values,
        name: 'path',
      },),
      ...shared,
    };
  }
  if (name === 'commit-open-merge') {
    /**
     The worktree whose merge is committed, empty for the main checkout.
     */
    const worktree = textOption({
      values,
      name: 'worktree',
    },);
    return {
      command: 'commit-open-merge',
      message: requiredTextOption({
        values,
        name: 'message',
      },),
      worktree: worktree.found ? worktree.value : '',
      paths: textsOption({
        values,
        name: 'path',
      },),
      ...shared,
    };
  }
  if (name === 'list') {
    return {
      command: 'list',
      list: requiredTextOption({
        values,
        name: 'list',
      },),
      only: textsOption({
        values,
        name: 'only',
      },),
      ...shared,
    };
  }
  if (name === 'remove-worktrees') {
    return {
      command: 'remove-worktrees',
      apply: values.apply === true,
      ...shared,
    };
  }
  return {
    command: 'worktrees',
    ...shared,
  };
}

/**
 Reads which check a caller asks for.

 @param text - the option's text

 @returns the check

 @throws LandingConfigurationError when it is not one of the four

 @example
 ```ts
 const check = checkKindOf('wrapper',);
 ```
 */
function checkKindOf(text: string,): CheckKind {
  if ((text === 'wrapper') || (text === 'package')
    || (text === 'tasks')
    || (text === 'none')) return text;
  throw new LandingConfigurationError(`--check=${text}: this must be wrapper, package, tasks or none`,);
}

/**
 Reads which side of a conflict region to keep.

 @param text - the option's text, or the side a path names after its `=`

 @returns the side

 @throws LandingConfigurationError when it is not one of the four

 @example
 ```ts
 const side = parseSideToKeep('incoming',);
 ```
 */
export function parseSideToKeep(text: string,): SideToKeep {
  /**
   The side that text names.
   */
  const namedSide = SIDES_TO_KEEP.find(function isNamed(side: SideToKeep,) {
    return side === text;
  },);
  if (namedSide === undefined) {
    throw new LandingConfigurationError(`--side=${text}: this must be one of ${SIDES_TO_KEEP.join(', ')}`,);
  }
  return namedSide;
}
