#!/usr/bin/env node
// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import { message, } from '@optique/core/message';
// TODO: deprecate Optique
import {
  optional,
  withDefault,
} from '@optique/core/modifiers';
// TODO: deprecate Optique
import {
  argument,
  flag,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import {
  choice,
  integer,
  string,
} from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';

import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import {
  shouldColor,
  type ColorMode,
} from './color.ts';
import {
  DEFAULT_MAX_DEEPEN_COMMITS,
  DEFAULT_MAX_PACK_BYTES,
  DEFAULT_MAX_PROBE_SECONDS,
} from './constants.ts';
import { serializeSnapshot, } from './serialize.ts';
import { detectSource, } from './source.ts';
import {
  estimate,
  type EstimateOptions,
} from './stream.ts';



//region Arg parsing

/**
 TODO: deprecate Optique
 Top-level argument parser: an optional SOURCE positional plus tuning flags.
 */
const parser = object({
  source: optional(
    argument(string({ metavar: 'SOURCE', },),),
  ),
  defaultBranchOnly: withDefault(
    flag('--default-branch-only',),
    false,
  ),
  maxProbeSeconds: withDefault(
    option(
      '--max-probe-seconds',
      integer({ min: 1, },),
    ),
    DEFAULT_MAX_PROBE_SECONDS,
  ),
  maxDeepenCommits: withDefault(
    option(
      '--max-deepen-commits',
      integer({ min: 1, },),
    ),
    DEFAULT_MAX_DEEPEN_COMMITS,
  ),
  maxPackBytes: withDefault(
    option(
      '--max-pack-bytes',
      integer({ min: 1, },),
    ),
    DEFAULT_MAX_PACK_BYTES,
  ),
  color: withDefault(
    option(
      '--color',
      choice([
        'auto',
        'always',
        'never',
      ],),
    ),
    'auto',
  ),
},);

/**
 TODO: deprecate Optique
 Parsed CLI arguments.
 */
const args = runSync(
  parser,
  {
    programName: 'git-clone-size',
    args: process.argv
      .slice(2,),
    help: 'option',
    aboveError: 'help',
    brief: message`git-clone-size - estimate a repo's shallow/full clone size ratio without a full clone`,
    footer:
      message`Output is JSONL (one EstimateSnapshot per line). Examples: git-clone-size . | git-clone-size https://github.com/owner/repo.git | git-clone-size --default-branch-only --color=never <url>`,
  },
);

//endregion Arg parsing

//region Main execution

/**
 Tagged logger for the CLI driver (stderr only; stdout stays JSONL-pure).
 */
const rl = tagged({
  tag: 'main',
  l: logger,
},);

/**
 Process working directory, used when no SOURCE is given.
 */
const cwd = process.cwd();

/**
 Resolved source: a remote URL, or a local path defaulting to cwd.
 */
const source = await detectSource({
  cwd,
  ...args.source === undefined ? {} : { input: args.source, },
},);

/**
 Whether to ANSI-highlight the JSONL (auto by TTY, overridable by `--color`).
 */
const colorOn = shouldColor({
  mode: args.color satisfies ColorMode,
  stream: process.stdout,
},);

/**
 SIGINT abort controller: a first Ctrl-C aborts the probes so the stream
 finalizes with the current best snapshot rather than dying mid-clone.
 */
const abortController = new AbortController();
process.once(
  'SIGINT',
  function onSigint(): void {
    rl.debug('SIGINT received; aborting probes and finalizing',);
    abortController.abort();
  },
);

/**
 Runtime options derived from the parsed arguments.
 */
const options: EstimateOptions = {
  defaultBranchOnly: args.defaultBranchOnly,
  maxDeepenCommits: args.maxDeepenCommits,
  maxPackBytes: args.maxPackBytes,
  maxProbeSeconds: args.maxProbeSeconds,
  signal: abortController.signal,
};

rl.debug(`estimating ${source.kind === 'local' ? source.path : source.url}`,);

for await (const snapshot of estimate({
  source,
  options,
},)) {
  process.stdout
    .write(`${serializeSnapshot({
      snapshot,
      colorOn,
    },)}\n`,);
}

//endregion Main execution
