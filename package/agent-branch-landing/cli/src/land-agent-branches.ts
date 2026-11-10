#!/usr/bin/env node
/**
 `land-agent-branches`: lands agents' branches on `main` in the main checkout. See `README.md` for the subcommands, the list
 file, and the checks each branch kind gets.

 @module
 */

import { argv, } from 'node:process';
import { runTheCommandLine, } from './run-the-command-line.ts';

process.exitCode = await runTheCommandLine({
  args: argv.slice(2,),
  workingFolder: process.cwd(),
},);
