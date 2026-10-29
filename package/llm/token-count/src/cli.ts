#!/usr/bin/env node
// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import {
  multiple,
  optional,
} from '@optique/core/modifiers';
// TODO: deprecate Optique
import {
  argument,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';

import { countFileTokens, } from './client.ts';

//region CLI: parses args and counts tokens in files

/**
 Column width for right-aligning token counts in output.
 */
const PAD_WIDTH = 8;

/**
 TODO: deprecate Optique
 Optique parser for the token-count CLI.
 
 @example
 ```bash
 token-count CLAUDE.md
 token-count --model claude-haiku-4-5 file1.md file2.md
 ```
 */
const parser = object({
  model: optional(option(
    '--model',
    string({ metavar: 'MODEL', },),
  ),),
  files: multiple(
    argument(string({ metavar: 'FILE', },),),
  ),
},);

/**
 TODO: deprecate Optique
 Parsed CLI arguments
 */
const args = runSync(
  parser,
  {
    programName: 'token-count',
    help: 'option',
  },
);

if (args.files
  .length
  === 0)
  throw new Error('At least one FILE argument is required',);

/**
 Model override from `--model` flag, or `undefined` for the default.
 */
const model = ((typeof args.model) === 'string') ? args.model : undefined;
/**
 Configuration object passed to each `countFileTokens` call.
 */
const config = model !== undefined ? { model, } : {};

/**
 Token count results for all files, resolved concurrently.
 */
const results = await Promise.all(
  args.files
    .map(function countFile(filePath: string,) {
      return countFileTokens({
        filePath,
        config,
      },);
    },),
);

for (const result of results)
  console.log(`${String(result.inputTokens,)
    .padStart(PAD_WIDTH,)} ${result.filePath}`,);

if (results.length
  > 1) {
  /**
   Aggregate across files so the trailing summary line matches `wc -l`-style output.
   */
  const total = results.reduce(
    function sumTokens(
      sum,
      r,
    ) {
      return sum + r
        .inputTokens;
    },
    0,
  );
  console.log(`${String(total,)
    .padStart(PAD_WIDTH,)} total`,);
}

//endregion CLI
