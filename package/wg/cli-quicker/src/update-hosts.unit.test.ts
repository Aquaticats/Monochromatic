import {
  readdir,
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
  NON_FETCH_HOSTS,
  PLAN_UPDATE_HOSTS,
  REDIRECT_UPDATE_HOSTS,
  UPDATE_HOSTS,
} from '../dist/final/node/update-hosts.data.mjs';

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
 Toolchain directories whose non-test sources name the hosts the toolchain fetches from.
 */
const TOOLCHAIN_SOURCE_DIRECTORIES: readonly string[] = [
  'package/file-enforcer/core/src/toolchain',
  'package/mpm-shadow/cli/src/toolchain',
];

/**
 Reports whether a file name is test or fixture material.

 @param name - File name.

 @returns Whether the file is not production source.

 @example
 ```ts
 isTestMaterial({ name: 'sync.unit.test.ts' }); // true
 ```
 */
function isTestMaterial({ name, }: { readonly name: string; },): boolean {
  return name.includes('test',) || name.includes('fixture',) || name.includes('fakes',);
}

/**
 Extracts the host of every `https://` literal in a text.

 @param text - Source text.

 @returns Hosts without scheme,
 path,
 or port.

 @example
 ```ts
 hostsIn({ text: 'fetch("https://pypi.org/simple")' }); // ['pypi.org']
 ```
 */
function hostsIn({ text, }: { readonly text: string; },): readonly string[] {
  return text
    .split('https://',)
    .slice(1,)
    .flatMap(function hostOf(rest,): readonly string[] {
      /**
       Host characters before the first character that cannot belong to a host.
       */
      const host = Array
        .from(rest,)
        .findIndex(function endsHost(character,): boolean {
          return !(((character >= 'a') && (character <= 'z'))
            || ((character >= '0') && (character <= '9'))
            || (character === '.')
            || (character === '-'));
        },);
      if ((host <= 0) || (rest.startsWith('.',)))
        return [];
      /**
       Candidate host,
       possibly ending in the sentence period of a comment.
       */
      const candidate = rest.slice(0, host,);
      return [candidate.endsWith('.',) ? candidate.slice(0, -1,) : candidate,];
    },);
}

/**
 Collects every `https://` host from the toolchain's production sources and lock.

 @returns Distinct hosts,
 sorted.

 @example
 ```ts
 await toolchainHosts();
 ```
 */
async function toolchainHosts(): Promise<readonly string[]> {
  /**
   Texts of the lock file and each production source file.
   */
  const texts = await Promise.all([
    readFile(join(REPOSITORY_ROOT, 'toolchain.lock.json',), 'utf8',),
    ...TOOLCHAIN_SOURCE_DIRECTORIES.map(async function directoryTexts(directory,): Promise<string> {
      /**
       Names in the directory.
       */
      const names = await readdir(join(REPOSITORY_ROOT, directory,),);
      /**
       Contents of each production source.
       */
      const contents = await Promise.all(names
        .filter(function isSource(name,): boolean {
          return (name.endsWith('.ts',) || name.endsWith('.rs',)) && (!isTestMaterial({ name, },));
        },)
        .map(function readSource(name,): Promise<string> {
          return readFile(join(REPOSITORY_ROOT, directory, name,), 'utf8',);
        },),);
      return contents.join('\n',);
    },),
  ],);
  return [...new Set(texts.flatMap(function textHosts(text,): readonly string[] {
    return hostsIn({ text, },);
  },),),].toSorted();
}

await describe({
  name: 'UPDATE_HOSTS',
  children: [
    it({
      name: 'covers every host the toolchain plan, lock and resolvers name',
      fn: async () => {
        /**
         Hosts named by the toolchain but absent from the wg list.
         */
        const missing = (await toolchainHosts()).filter(function isMissing(host,): boolean {
          return (!NON_FETCH_HOSTS.includes(host,)) && (!UPDATE_HOSTS.includes(host,));
        },);
        expect(missing,).toEqual([],);
      },
    },),
    it({
      name: 'lists every plan host the toolchain still names and no host twice',
      fn: async () => {
        /**
         Hosts the toolchain sources currently name.
         */
        const named = await toolchainHosts();
        expect(PLAN_UPDATE_HOSTS.filter((host,) => !named.includes(host,)),).toEqual([],);
        expect(new Set(UPDATE_HOSTS,).size,).toBe(UPDATE_HOSTS.length,);
      },
    },),
    it({
      name: 'adds the redirect hosts the plan never names and rejects look-alike hosts',
      fn: async () => {
        for (
          const host of [
            'objects.githubusercontent.com',
            'release-assets.githubusercontent.com',
            'static.crates.io',
            'index.crates.io',
          ]
        )
          expect(REDIRECT_UPDATE_HOSTS,).toContain(host,);
        for (const host of ['crates.io.evil.example', 'githubusercontent.com', 'raw.githubusercontent.com', 'example.com',])
          expect(UPDATE_HOSTS,).not.toContain(host,);
      },
    },),
  ],
},);
