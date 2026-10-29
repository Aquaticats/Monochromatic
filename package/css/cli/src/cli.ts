#!/usr/bin/env node
// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import { argument, } from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';
import {
  buildCss,
  type CssBuildOptions,
} from './index.ts';

//region CLI: parses args and runs the build

/**
 TODO: deprecate Optique
 Optique parser for the CSS build tool CLI.
 
 @example
 ```bash
 build-css src/main.css dist/bundle.css
 ```
 */
const parser = object({
  input: argument(string({ metavar: 'INPUT', },),),
  output: argument(string({ metavar: 'OUTPUT', },),),
},);

/**
 TODO: deprecate Optique
 Parsed CLI arguments cast to the shared build options type
 */
const args: CssBuildOptions = runSync(
  parser,
  {
    programName: 'build-css',
    help: 'option',
  },
);

console.log(`Building CSS: ${args.input} -> ${args.output}`,);
await buildCss(args,);
console.log('Build complete',);

//endregion CLI
