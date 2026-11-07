/**
 The holder scan: which paths a writer could hold open, and the root command that looks for them.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import type { GuidedConfig, } from './guided-types.ts';

/**
 The folder names directly below the external worktrees folder, sorted.

 @param config - the configuration
 
 @returns the names

 @example
 ```ts
 import { externalNames, } from './guided-holders.ts';
 
 const result = await externalNames(config,);
 ```
 */
export async function externalNames(config: GuidedConfig,): Promise<readonly string[]> {
  if (!await pathExists(config.externalWorktrees,)) {
    return [];
  }
  return (await readdir(
    config.externalWorktrees,
    { withFileTypes: true, },
  )).filter(function isFolder(entry,) {
    return entry.isDirectory();
  },)
    .map(function nameOf(entry,) {
      return entry.name;
    },)
    .toSorted();
}

/**
 The `-lname` patterns of the holder scan: every place a writer could hold open (the repository, the external
 worktrees, and each registered worktree elsewhere, with its spelling on the 860).

 @param config - the configuration
 
 @returns one pattern per line, sorted, without duplicates

 @example
 ```ts
 import { holderPatterns, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = await holderPatterns(config,);
 ```
 */
export async function holderPatterns(config: GuidedConfig,): Promise<readonly string[]> {
  /**
   The patterns the holder scan matches.
   */
  const patterns = new Set<string>([
    `${config.repo}*`,
    `${config.externalWorktrees}*`,
  ],);
  /**
   The path of the porcelain listing taken before the freeze.
   */
  const porcelain = join(
    config.wiz,
    'porcelain-start.txt',
  );
  if (await pathExists(porcelain,)) {
    for (const line of (await readFile(
      porcelain,
      'utf8',
    )).split('\n',)) {
      /**
       The path this step works on.
       */
      const path = line.startsWith('worktree ',) ? line.slice('worktree '.length,) : '';
      if ((path !== '') && (!path.startsWith(config.repo,))
        && (!path.startsWith(config.externalWorktrees,))) {
        patterns.add(`${path}*`,);
        if (path.startsWith(`${config.homeDir}/temp/`,)) {
          patterns.add(`${config.drive}/temp/${path.slice(`${config.homeDir}/temp/`.length,)}*`,);
        }
      }
    }
  }
  return [...patterns,].toSorted();
}

/**
 Builds the root holder-scan command for an output name.

 @param config - the configuration
 
 @param name - output name under the state folder
 
 @returns the shell text

 @example
 ```ts
 import { holderScanCommand, } from './guided-holders.ts';
 
 const result = await holderScanCommand({ config, name, },);
 ```
 */
export async function holderScanCommand(
  {
    config,
    name,
  }: {
    readonly config: GuidedConfig;
    readonly name: string;
  },
): Promise<string> {
  /**
   The patterns the holder scan matches.
   */
  const patterns = (await holderPatterns(config,)).map(function toTest(pattern,) {
    return `-lname '${pattern}'`;
  },)
    .join(' -o ',);
  return `sudo /usr/bin/find /proc -maxdepth 3 \\( -name cwd -o -path '/proc/*/fd/*' \\) \\( ${patterns} \\) -printf '%p %l\\n' > ${config.wiz}/${name}.txt 2> ${config.wiz}/${name}.err`;
}
