import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { UPDATE_HOSTS, } from '../dist/final/node/update-hosts.data.mjs';
import {
  POLICY_PROCESS_WATCH,
  POLICY_UPDATE_HOSTS,
  policiesNotInForceWithoutMark,
  policyNotInForce,
  updatePolicyNotice,
} from '../dist/final/node/update-policy-notice.mjs';

/**
 Directory of this package's sources.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Directory of the Rust companion's sources.
 */
const COMPANION_SOURCE_DIRECTORY = join(SOURCE_DIRECTORY, '..', '..', 'cli-quicker-exempt', 'src',);

/**
 Words that name a switch for the update policies.
 */
const SWITCH_WORDS: readonly string[] = [
  'MONOCHROMATIC_TOOLCHAIN_CLEARNET',
  'MONOCHROMATIC_TOOLCHAIN_IN_UPDATES_SLICE',
  'WG_QUICKER_NO_UPDATE',
  'WG_QUICKER_DISABLE',
  'opt-out',
  'optOut',
];

/**
 Reads the production source files of a directory.

 @param directory - Directory to read.

 @param extension - File extension including the dot.

 @returns File name and text pairs.

 @example
 ```ts
 await productionSources({ directory, extension: '.ts' });
 ```
 */
async function productionSources(
  {
    directory,
    extension,
  }: {
    readonly directory: string;
    readonly extension: string;
  },
): Promise<readonly (readonly [string, string])[]> {
  return Promise.all(
    (await readdir(directory,))
      .filter(function isProduction(name,): boolean {
        return name.endsWith(extension,) && (!name.includes('test',)) && (!name.includes('fixture',));
      },)
      .map(async function read(name,): Promise<readonly [string, string]> {
        return [name, await readFile(join(directory, name,), 'utf8',),];
      },),
  );
}

await describe({
  name: updatePolicyNotice.name,
  children: [
    it({
      name: 'names the traffic that leaves, every update host, the reason, and that nothing turns it off',
      fn: async () => {
        /**
         Notice printed on every start.
         */
        const notice = updatePolicyNotice({ hosts: UPDATE_HOSTS, },);
        for (
          const phrase of [
            'POLICY NOTICE',
            'without any switch',
            'OUTSIDE the WireGuard tunnel',
            'Update entry points',
            'mpm-shadow install, update and outdated',
            'Toolchain install roots',
            'Update hosts, for every program including browsers',
            'Why:',
            'must not use this tool',
          ]
        )
          expect(notice,).toContain(phrase,);
        for (const host of UPDATE_HOSTS)
          expect(notice,).toContain(host,);
      },
    },),
  ],
},);

await describe({
  name: policiesNotInForceWithoutMark.name,
  children: [
    it({
      name: 'names both policies and the reason they cannot be enforced',
      fn: async () => {
        /**
         Report for a start without an exemption mark.
         */
        const report = policiesNotInForceWithoutMark();
        expect(report,).toContain('POLICY NOT IN FORCE',);
        expect(report,).toContain(POLICY_PROCESS_WATCH,);
        expect(report,).toContain(POLICY_UPDATE_HOSTS,);
        expect(report,).toContain('No ExemptMark is configured',);
        expect(policyNotInForce({ policy: POLICY_UPDATE_HOSTS, detail: 'DNS refresh failed', },),).toBe(
          `POLICY NOT IN FORCE: ${POLICY_UPDATE_HOSTS}: DNS refresh failed`,
        );
      },
    },),
  ],
},);

await describe({
  name: 'start path',
  children: [
    it({
      name: 'prints the notice on every up with a mark and the unenforced report without one',
      fn: async () => {
        /**
         Command entry point source.
         */
        const entry = await readFile(join(SOURCE_DIRECTORY, 'index.ts',), 'utf8',);
        expect(entry,).toContain('updatePolicyNotice({ hosts: UPDATE_HOSTS, },)',);
        expect(entry,).toContain('policiesNotInForceWithoutMark()',);
        expect(entry,).toContain('will use the tunnel.',);
      },
    },),
  ],
},);

await describe({
  name: 'always-on policy',
  children: [
    it({
      name: 'has no switch, flag or environment variable that turns a bypass off in either wg package',
      fn: async () => {
        /**
         Production sources of both wg packages.
         */
        const sources = [
          ...await productionSources({ directory: SOURCE_DIRECTORY, extension: '.ts', },),
          ...await productionSources({ directory: COMPANION_SOURCE_DIRECTORY, extension: '.rs', },),
        ];
        /**
         Sources that mention a switch word.
         */
        const offenders = sources.flatMap(function mentionsSwitch([name, text,],): readonly string[] {
          return SWITCH_WORDS.filter(function appears(word,): boolean {
            return text.includes(word,);
          },).map(function describeHit(word,): string {
            return `${name}: ${word}`;
          },);
        },);
        expect(offenders,).toEqual([],);
      },
    },),
  ],
},);
