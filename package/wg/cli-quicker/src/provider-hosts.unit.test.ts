import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  PROVIDER_BASE_URLS,
  PROVIDER_HOSTS,
} from '../dist/final/node/provider-hosts.data.mjs';
import { UPDATE_HOSTS, } from '../dist/final/node/update-hosts.data.mjs';

/**
 Repository root,
 four levels above this source file (`package/wg/cli-quicker/src`).
 */
const REPOSITORY_ROOT = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 Repository sources that name a host of a listed provider:
 the NeuralWatt listing constants and `DeepSeek` client the Pi plugins and the
 search tool reach through.
 A provider that starts using another host in one of these files fails the
 scan below until that host is listed.
 */
const PROVIDER_SOURCE_FILES: readonly string[] = [
  'package/agent-harness-shared/neuralwatt-listing/src/constants.ts',
  'package/search-fetch/core/src/deepseek-client.ts',
];

/**
 Pi provider ids the owner named,
 each of which must have a recorded base URL.
 */
const NAMED_PROVIDER_IDS: readonly string[] = [
  'deepseek',
  'neuralwatt',
  'opencode-go',
  'qwen-token-plan-cn',
  'qwen-token-plan-individual',
  'synthetic',
];

/**
 Extracts the host of every `https://` literal in a text.

 @param text - Source text.

 @returns Hosts without scheme,
 path,
 or port.

 @example
 ```ts
 hostsIn({ text: 'baseUrl: "https://api.deepseek.com"' }); // ['api.deepseek.com']
 ```
 */
function hostsIn({ text, }: { readonly text: string; },): readonly string[] {
  return text
    .split('https://',)
    .slice(1,)
    .flatMap(function hostOf(rest,): readonly string[] {
      /**
       Index of the first character that cannot belong to a host.
       */
      const end = Array
        .from(rest,)
        .findIndex(function endsHost(character,): boolean {
          return !(((character >= 'a') && (character <= 'z'))
            || ((character >= '0') && (character <= '9'))
            || (character === '.')
            || (character === '-'));
        },);
      return end <= 0 ? [] : [rest.slice(0, end,),];
    },);
}

/**
 Reports whether a text is a bare hostname:
 lowercase letters,
 digits,
 dots and dashes,
 with no scheme,
 port,
 path or wildcard.

 @param host - Candidate host.

 @returns Whether the text can be a host rule's DNS name.

 @example
 ```ts
 isBareHost({ host: 'api.synthetic.new' }); // true
 ```
 */
function isBareHost({ host, }: { readonly host: string; },): boolean {
  return host.length > 0
    && host.includes('.',)
    && (!host.startsWith('.',))
    && (!host.endsWith('.',))
    && Array.from(host,)
      .every(function isHostCharacter(character,): boolean {
        return ((character >= 'a') && (character <= 'z'))
          || ((character >= '0') && (character <= '9'))
          || (character === '.')
          || (character === '-');
      },);
}

await describe({
  name: 'PROVIDER_HOSTS',
  children: [
    it({
      name: 'derives one bare host per recorded provider base URL, with no duplicate and no look-alike',
      fn: () => {
        for (const entry of PROVIDER_BASE_URLS)
          expect(new URL(entry.baseUrl,).protocol,).toBe('https:',);
        /**
         Hosts the recorded base URLs name.
         */
        const recorded = PROVIDER_BASE_URLS.map(function hostOf(entry,): string {
          return new URL(entry.baseUrl,).hostname;
        },);
        expect(PROVIDER_HOSTS.toSorted(),).toEqual([...new Set(recorded,),].toSorted(),);
        for (const host of PROVIDER_HOSTS)
          expect(isBareHost({ host, },),).toBe(true,);
      },
    },),
    it({
      name: 'records a base URL for every provider the owner named',
      fn: () => {
        /**
         Provider ids the list covers.
         */
        const covered = PROVIDER_BASE_URLS.map(function idOf(entry,): string {
          return entry.provider;
        },);
        expect(NAMED_PROVIDER_IDS.filter((id,) => !covered.includes(id,)),).toEqual([],);
      },
    },),
    it({
      name: 'stays apart from the update hosts, which leave the tunnel for another reason',
      fn: () => {
        expect(PROVIDER_HOSTS.filter((host,) => UPDATE_HOSTS.includes(host,)),).toEqual([],);
        expect(new Set(PROVIDER_HOSTS,).size,).toBe(PROVIDER_HOSTS.length,);
      },
    },),
    it({
      name: 'covers every provider host the repository\'s own provider sources name',
      fn: async () => {
        /**
         Hosts the repository sources name,
         minus the ones they only cite as documentation.
         */
        const named = new Set(PROVIDER_HOSTS,);
        for (const file of PROVIDER_SOURCE_FILES) {
          /**
           Text of one provider source.
           */
          const text = await readFile(join(REPOSITORY_ROOT, file,), 'utf8',);
          for (const host of hostsIn({ text, },))
            expect(named.has(host,),).toBe(true,);
        }
      },
    },),
    it({
      name: 'rejects look-alike hosts that a prefix or suffix match would accept',
      fn: () => {
        for (const host of ['api.deepseek.com.evil.example', 'deepseek.com', 'synthetic.new', 'opencode.ai.example',])
          expect(PROVIDER_HOSTS,).not.toContain(host,);
      },
    },),
  ],
},);
