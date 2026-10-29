#!/usr/bin/env node
/**
 `cachyos-image`: prints the CachyOS base image for this host.
 See {@link runCachyosImageCli} for the arguments and exit codes.
 */

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { runCachyosImageCli, } from '../cachyos-image-cli.ts';

/**
 Writes one resolution-trail line to stderr, so stdout carries only the image.

 @param level - Severity label

 @returns a logging function for that level
 */
function stderrLine(level: string,): (message: string,) => void {
  return function write(message: string,): void {
    process.stderr
      .write(`[${level}] ${message}\n`,);
  };
}

/**
 Logger that keeps stdout clean for the printed image.
 */
const stderrLogger: Logger = {
  debug: stderrLine('debug',),
  error: stderrLine('error',),
  fatal: stderrLine('fatal',),
  flush: function flush(): Promise<void> {
    return Promise.resolve();
  },
  info: stderrLine('info',),
  trace: stderrLine('trace',),
  warn: stderrLine('warn',),
};

/**
 Outcome of this invocation.
 */
const result = await runCachyosImageCli({
  args: process.argv
    .slice(2,),
  resolveOptions: { l: stderrLogger, },
},);

process.stdout
  .write(result.stdout,);
process.stderr
  .write(result.stderr,);
process.exitCode = result.exitCode;
