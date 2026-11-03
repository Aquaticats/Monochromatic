#!/usr/bin/env node
/**
 What:
  Prints the summary of each `strace` record it is given, so the work a wrapped commit adds shows beside the native commit's.
 Why:
  `native-perf.ts --trace-directory <dir>` leaves one record per sample and one per commit service;
  `native-perf-trace-summary.ts` sums them. Run: `node bin/native-perf-trace.ts <record>...`.

 In TS you'd write (pseudocode):
 ```ts
 for (const file of files) console.log(summarizeTrace(await readFile(file, 'utf8')));
 ```
 */
import { readFile } from 'node:fs/promises';
import { basename } from 'node:path';
import { argv } from 'node:process';

import {
  type Group,
  summarizeTrace,
  total,
} from './native-perf-trace-summary.ts';

/**
 How many of the slowest calls and paths each summary lists.
 */
const LISTED = 8;

/**
 Format a group as `count x, seconds ms`.

 @param group - the group

 @returns the text
 */
function show(group: Group): string {
  return `${String(group.count)} calls, ${(group.seconds * 1000).toFixed(1)} ms`;
}

/**
 The entries of a group table with the most call time, formatted one per line.

 @param table - groups by key

 @returns the lines
 */
function top(table: Readonly<Record<string, Group>>): readonly string[] {
  return Object.entries(table)
    .toSorted(function byTime(
      [, left],
      [, right],
    ) {
      return right.seconds - left.seconds;
    })
    .slice(
      0,
      LISTED,
    )
    .map(function line([
      key,
      group,
    ]) {
      return `    ${key}: ${show(group)}`;
    });
}

/**
 The entries of a byte table with the most bytes, formatted one per line.

 @param table - bytes by key

 @returns the lines
 */
function topBytes(table: Readonly<Record<string, number>>): readonly string[] {
  return Object.entries(table)
    .toSorted(function byBytes(
      [, left],
      [, right],
    ) {
      return right - left;
    })
    .slice(
      0,
      LISTED,
    )
    .map(function line([
      key,
      bytes,
    ]) {
      return `    ${key}: ${String(bytes)} bytes`;
    });
}

for (const file of argv.slice(2)) {
  /**
   Summary of one record.
   */
  const summary = summarizeTrace(await readFile(
    file,
    'utf8',
  ));
  console.log(`== ${basename(file)}`);
  console.log(`  wall ${(summary.wallSeconds * 1000).toFixed(0)} ms (traced; only the counts are evidence)`);
  console.log(`  syncs ${show(total(summary.syncs))}: ${JSON.stringify(summary.syncs)}`);
  console.log(`  bytes read ${String(summary.bytesRead)}, written ${String(summary.bytesWritten)}`);
  console.log(`  files created ${String(summary.filesCreated)}, renames ${String(summary.renames)}, removals ${String(summary.removals)}`);
  console.log(`  processes ${String(Object.values(summary.processes).reduce((a, b) => a + b, 0))}: ${JSON.stringify(summary.processes)}`);
  console.log('  most bytes read by path:');
  console.log(topBytes(summary.readByPath).join('\n'));
  console.log('  most bytes written by path:');
  console.log(topBytes(summary.writtenByPath).join('\n'));
  console.log('  programs in start order:');
  console.log(summary.commands.map(function indent(command) {
    return `    ${command}`;
  }).join('\n'));
  console.log('  synced paths:');
  console.log(top(summary.syncedPaths).join('\n'));
  console.log('  slowest calls by name:');
  console.log(top(summary.byCall).join('\n'));
  console.log('  slowest paths:');
  console.log(top(summary.byPath).join('\n'));
}
