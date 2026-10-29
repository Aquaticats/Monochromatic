#!/usr/bin/env node
/**
 Executable entry point for OpenCodeReview Issue adapter.
 
 @module
 */

import process from 'node:process';

import { runCli, } from './cli-entry.ts';

/**
 Settled command exit status.
 */
const status = await runCli({
  arguments: process.argv
    .slice(2,),
  cwd: process.cwd(),
  streams: {
    stdin: process.stdin,
    stdout: process.stdout,
    stderr: process.stderr,
  },
},);
process.exitCode = status;
