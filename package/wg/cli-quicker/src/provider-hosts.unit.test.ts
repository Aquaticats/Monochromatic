import {
  readFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

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
 the NeuralWatt listing constants the Pi plugin reads,
 and the `DeepSeek` client the search tool reaches.
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
 Hosts a prefix or suffix match would wrongly accept,
 so the list is checked for exact entries.
 */
const LOOKALIKE_HOSTS: readonly string[] = [
  'api.deepseek.com.evil.example',
  'deepseek.com',
  'synthetic.new',
  'opencode.ai.example',
];

/**
 Reports whether one character may appear in a hostname.

 @param character - Candidate character.

 @returns Whether the character is a lowercase letter,
 a digit,
 a dot,
 or a dash.

 @example
 ```ts
 isHostCharacter({ character: 'a' }); // true
 ```
 */
function isHostCharacter({ character, }: { readonly character: string; },): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= '0') && (character <= '9'))
    || (character === '.')
    || (character === '-');
}

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
          return !isHostCharacter({ character, },);
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
  if ((host.length === 0) || (!host.includes('.',)))
    return false;
  if (host.startsWith('.',) || host.endsWith('.',))
    return false;
  return Array
    .from(host,)
    .every(function isHostLetter(character,): boolean {
      return isHostCharacter({ character, },);
    },);
}

/**
 Hosts the recorded base URLs name,
 in the order they are recorded.

 @returns One host per entry.

 @example
 ```ts
 recordedHosts();
 ```
 */
function recordedHosts(): readonly string[] {
  return PROVIDER_BASE_URLS.map(function hostOf(entry,): string {
    return new URL(entry.baseUrl,).hostname;
  },);
}

await describe({
  name: 'PROVIDER_HOSTS',
  children: [
    it({
      name: 'derives one bare host per recorded provider base URL, with no duplicate and no look-alike',
      fn: async () => {
        for (const entry of PROVIDER_BASE_URLS)
          expect(new URL(entry.baseUrl,).protocol,).toBe('https:',);
        expect(PROVIDER_HOSTS.toSorted(),).toEqual([...new Set(recordedHosts(),),].toSorted(),);
        for (const host of PROVIDER_HOSTS)
          expect(isBareHost({ host, },),).toBe(true,);
      },
    },),
    it({
      name: 'records a base URL for every provider the owner named',
      fn: async () => {
        /**
         Provider ids the list covers.
         */
        const covered = new Set(PROVIDER_BASE_URLS.map(function idOf(entry,): string {
          return entry.provider;
        },),);
        expect(NAMED_PROVIDER_IDS.filter((id,) => !covered.has(id,)),).toEqual([],);
      },
    },),
    it({
      name: 'stays apart from the update hosts, which leave the tunnel for another reason',
      fn: async () => {
        expect(PROVIDER_HOSTS.filter((host,) => UPDATE_HOSTS.includes(host,)),).toEqual([],);
        expect(new Set(PROVIDER_HOSTS,).size,).toBe(PROVIDER_HOSTS.length,);
      },
    },),
    it({
      name: 'covers every provider host the repository\'s own provider sources name',
      fn: async () => {
        /**
         Hosts the list carries,
         as a set the scan can ask.
         */
        const listed = new Set(PROVIDER_HOSTS,);
        /**
         Texts of the provider sources,
         read together.
         */
        const texts = await Promise.all(PROVIDER_SOURCE_FILES.map(function readSource(file,): Promise<string> {
          return readFile(join(REPOSITORY_ROOT, file,), 'utf8',);
        },),);
        for (const text of texts) {
          for (const host of hostsIn({ text, },))
            expect(listed.has(host,),).toBe(true,);
        }
      },
    },),
    it({
      name: 'rejects look-alike hosts that a prefix or suffix match would accept',
      fn: async () => {
        for (const host of LOOKALIKE_HOSTS)
          expect(PROVIDER_HOSTS,).not.toContain(host,);
      },
    },),
  ],
},);
