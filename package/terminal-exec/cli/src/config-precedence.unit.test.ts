/**
 Precedence tests for `parseConfigFiles`.

 Config paths arrive in descending priority order (the user's
 `~/.config/xdg-terminals.list` first, system files after), and every output
 must follow that order: entries, execarg defaults, and exclusion markers. This
 file pins the first-match-wins reading, matching the reference
 `xdg-terminal-exec` script, whose default lookup takes the first matching
 directive and whose markers are ignored once an id is resolved.

 @module
 */

import {
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseConfigFiles, } from '../dist/final/node/launch.mjs';

/**
 Terminal id shared by the conflicting directive fixtures.
 */
const TERMINAL_ID = 'com.example.terminal.desktop';

/**
 Id shared by the conflicting exclusion fixtures.
 */
const EXCLUDED_ID = 'com.example.other.desktop';

/**
 User-file entry, listed before the system file's entry.
 */
const USER_ENTRY = 'com.example.user.desktop';

/**
 System-file entry, listed after the user file's entry.
 */
const SYSTEM_ENTRY = 'com.example.system.desktop';

/**
 Writes one fixture config file into a fresh scratch directory.

 @param name - File name inside the new directory.

 @param text - File contents.

 @returns Absolute path of the written file.
 */
async function fixtureFile({
  name,
  text,
}: {
  readonly name: string;
  readonly text: string;
},): Promise<string> {
  /**
   Fresh directory for this fixture, under the shared scratch root.
   */
  const directory = await mkdtemp(join(scratchRoot(), 'terminal-exec-config-',),);
  /**
   Absolute path of the file handed back to the parser.
   */
  const path = join(
    directory,
    name,
  );
  await writeFile(
    path,
    text,
    'utf8',
  );
  return path;
}

/**
 Higher-priority (user) config contents: a default, an exclusion, and an entry.
 */
const USER_CONFIG = [
  `/execarg_default:${TERMINAL_ID}:-e`,
  `-${EXCLUDED_ID}`,
  USER_ENTRY,
  '',
].join('\n',);

/**
 Lower-priority (system) config contents, contradicting the user lines above.
 */
const SYSTEM_CONFIG = [
  `/execarg_default:${TERMINAL_ID}:--`,
  `+${EXCLUDED_ID}`,
  SYSTEM_ENTRY,
  '',
].join('\n',);

/**
 Parses one user-then-system pair whose files contradict each other.

 @returns Configuration merged across both files.
 */
async function parseUserThenSystem() {
  return await parseConfigFiles({
    paths: [
      await fixtureFile({
        name: 'user-xdg-terminals.list',
        text: USER_CONFIG,
      },),
      await fixtureFile({
        name: 'system-xdg-terminals.list',
        text: SYSTEM_CONFIG,
      },),
    ],
  },);
}

await describe({
  name: parseConfigFiles.name,
  children: [
    it({
      name: 'keeps the higher-priority execarg default',
      fn: async () => {
        /**
         Configuration merged from the user file, then the system file.
         */
        const config = await parseUserThenSystem();
        /**
         Default merged for the conflicting terminal id.
         */
        const defaultArg = config.execArgDefaults.get(TERMINAL_ID,);
        expect(defaultArg,).toBe('-e',);
      },
    },),

    it({
      name: 'keeps the higher-priority exclusion marker',
      fn: async () => {
        /**
         Configuration merged from the user file, then the system file.
         */
        const config = await parseUserThenSystem();
        expect(config.excludedIds.has(EXCLUDED_ID,),).toBe(true,);
      },
    },),

    it({
      name: 'lists explicit entries in path priority order',
      fn: async () => {
        /**
         Configuration merged from the user file, then the system file.
         */
        const config = await parseUserThenSystem();
        expect(config.entryIds,).toEqual([USER_ENTRY, SYSTEM_ENTRY,],);
      },
    },),

    it({
      name: 'keeps a higher-priority inclusion over a lower-priority exclusion',
      fn: async () => {
        /**
         Configuration whose user file includes the id the system file excludes.
         */
        const config = await parseConfigFiles({
          paths: [
            await fixtureFile({
              name: 'user-xdg-terminals.list',
              text: `+${EXCLUDED_ID}\n`,
            },),
            await fixtureFile({
              name: 'system-xdg-terminals.list',
              text: `-${EXCLUDED_ID}\n`,
            },),
          ],
        },);
        expect(config.excludedIds.has(EXCLUDED_ID,),).toBe(false,);
      },
    },),
  ],
},);
