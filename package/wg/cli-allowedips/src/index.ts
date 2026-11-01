#!/usr/bin/env node

/**
 CLI entry point for generating a WireGuard `AllowedIPs` value.
 
 @example
 ```sh
 wg-allowedips --allowed allowed.txt --disallowed disallowed.txt
 ```
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { parseArgs, } from 'node:util';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { generateAllowedIps, } from '@monochromatic-dev/wg-core-allowedips/ts';

import { CliUsageError, } from './errors.ts';

/**
 Required input paths parsed from command-line arguments.
 */
type CliPaths = {
  readonly allowedPath: string;
  readonly disallowedPath: string;
};

/**
 Module logger for command lifecycle.
 */
const l = tagged({ tag: 'wg-allowedips', },);

/**
 Parses the exact two-option command contract.
 
 @param argv - Arguments after runtime and script path.
 
 @returns Required allowed and disallowed file paths.
 
 @throws {@link CliUsageError} when either required option is absent.
 
 @example
 ```ts
 parseCliPaths({ argv: ['--allowed', 'a.txt', '--disallowed', 'd.txt'] });
 ```
 */
function parseCliPaths({ argv, }: { readonly argv: readonly string[]; },): CliPaths {
  /**
   Values parsed by Node's strict built-in argument parser.
   */
  const { values, } = parseArgs({
    args: [...argv,],
    allowPositionals: false,
    strict: true,
    options: {
      allowed: { type: 'string', },
      disallowed: { type: 'string', },
    },
  },);
  if (values.allowed === undefined) {
    l.error('missing required option --allowed',);
    throw new CliUsageError('Missing required option: --allowed',);
  }
  if (values.disallowed === undefined) {
    l.error('missing required option --disallowed',);
    throw new CliUsageError('Missing required option: --disallowed',);
  }
  return {
    allowedPath: values.allowed,
    disallowedPath: values.disallowed,
  };
}

/**
 Arguments after runtime and script path.
 */
const processArguments = process.argv
  .slice(2,);
/**
 Parsed file paths from process arguments.
 */
const {
  allowedPath,
  disallowedPath,
} = parseCliPaths({ argv: processArguments, },);
l.debug(`reading allowed input from ${allowedPath}`,);
l.debug(`reading disallowed input from ${disallowedPath}`,);
/**
 Complete allowed and disallowed file texts read concurrently.
 */
const [allowedText, disallowedText,] = await Promise.all([
  readFile(
    allowedPath,
    'utf8',
  ),
  readFile(
    disallowedPath,
    'utf8',
  ),
],);
/**
 Exact stdout payload generated from both files.
 */
const output = await generateAllowedIps({
  allowedText,
  disallowedText,
},);
l.debug(`writing ${String(output.length,)} stdout byte(s)`,);
process
  .stdout
  .write(output,);
