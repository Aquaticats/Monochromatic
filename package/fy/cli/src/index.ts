#!/usr/bin/env node
// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import { message, } from '@optique/core/message';
// TODO: deprecate Optique
import {
  map,
  multiple,
} from '@optique/core/modifiers';
// TODO: deprecate Optique
import { argument, } from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';
// TODO: deprecate Optique
import { run, } from '@optique/run';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { coerceArg, } from './coerce.ts';
import { resolveSpecifier, } from './resolve.ts';

/**
 Logger root for cli-fy after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'cli-fy', },);



//region Arg parsing: positional: <specifier> <export> [args...]

/**
 TODO: deprecate Optique
 Value parser for the import specifier, displayed as SPECIFIER in help
 */
const specifierParser = string({ metavar: 'SPECIFIER', },);

/**
 TODO: deprecate Optique
 Value parser for the export name, displayed as EXPORT in help
 */
const exportParser = string({ metavar: 'EXPORT', },);

/**
 TODO: deprecate Optique
 Value parser for trailing call arguments, displayed as ARG in help
 */
const argParser = string({ metavar: 'ARG', },);

/**
 Parsed CLI arguments
 */
type CliArgs = {
  readonly specifier: string;
  readonly exportName: string;
  readonly callArgs: readonly string[];
};

/**
 TODO: deprecate Optique
 Top-level parser: <specifier> <export> [args...]
 */
const parser = map(
  object({
    specifier: argument(specifierParser,),
    exportName: argument(exportParser,),
    callArgs: multiple(argument(argParser,),),
  },),
  function toCliArgs(v: CliArgs,): CliArgs {
    return v;
  },
);

/**
 TODO: deprecate Optique
 Parsed result from process.argv
 */
const args = run(
  parser,
  {
    programName: 'cli-fy',
    args: process.argv
      .slice(2,),
    help: 'option',
    aboveError: 'help',
    brief: message`cli-fy - call any ESM export from the command line`,
    footer:
      message`Examples:\n  cli-fy lodash add 1 1\n  cli-fy @scope/pkg myFn arg1 arg2\n  cli-fy lodash-es/add default 1 1`,
  },
);

//endregion Arg parsing

//region Main execution: resolve, import, call, print

/**
 Tagged logger for the main execution flow.
 */
const rl = tagged({
  tag: 'main',
  l,
},);

rl.info(
  `specifier="${args.specifier}" export="${args.exportName}" args=[${
    args
      .callArgs
      .join(', ',)
  }]`,
);

/**
 Resolved file path for the import specifier
 */
const resolvedPath = await resolveSpecifier({ specifier: args.specifier, },);
rl.info(`resolved to ${resolvedPath}`,);

/**
 Dynamically imported module
 */
// oxlint-disable-next-line typescript/no-unsafe-type-assertion -- dynamic import yields unknown module shape
const mod = await import(resolvedPath) as Record<string, unknown>;

/**
 Target export value from the module
 */
const exportValue: unknown = mod[args.exportName];

if (exportValue === undefined) {
  /**
   Available export names for error message
   */
  const available = Object.keys(mod,)
    .join(', ',);
  throw new Error(
    `Export "${args.exportName}" not found in "${args.specifier}".\n`
      + `Available exports: ${available}`,
  );
}

if ((typeof exportValue) === 'function') {
  /**
   Coerced arguments for the function call
   */
  const coercedArgs = args.callArgs
    .map(function coerceCallArg(arg,) {
      return coerceArg({ arg, },);
    },);
  rl.info(`calling ${args.exportName}(${coercedArgs.map(String,)
    .join(', ',)})`,);

  /**
   Return value from calling the exported function
   */
  const result: unknown =
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- dynamic module call with unknown signature
    await (exportValue as (...fnArgs: readonly unknown[]) => unknown)(...coercedArgs,);
  console.log(result,);
}
else {
  if (args.callArgs
    .length
    > 0) {
    throw new Error(
      `Export "${args.exportName}" from "${args.specifier}" is not a function (got ${typeof exportValue}), `
        + `but ${String(args.callArgs
          .length,)} argument(s) were provided.`,
    );
  }
  console.log(exportValue,);
}

//endregion Main execution
