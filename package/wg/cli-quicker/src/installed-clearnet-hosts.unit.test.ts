import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CLEARNET_HOSTS, } from '../dist/final/node/bypass-host-refresh.mjs';
import {
  CLEARNET_HOSTS_DATA_SCHEMA_VERSION,
  clearnetHostsFromData,
  createInstalledClearnetHostsReader,
  defaultClearnetHostsDataPath,
  parseClearnetHostsData,
} from '../dist/final/node/installed-clearnet-hosts.mjs';
import { PROVIDER_HOSTS, } from '../dist/final/node/provider-hosts.data.mjs';
import { UPDATE_HOSTS, } from '../dist/final/node/update-hosts.data.mjs';

/**
 Data of the lists this build compiles in,
 as the data file states them.
 */
const BUILD_LISTS = {
  schemaVersion: CLEARNET_HOSTS_DATA_SCHEMA_VERSION,
  updateHosts: {
    plan: ['crates.io',],
    redirect: ['index.crates.io',],
    nonFetch: ['pnpm.io',],
  },
  providerBaseUrls: [
    { provider: 'deepseek', baseUrl: 'https://api.deepseek.com', },
  ],
};

/**
 Copies the build lists with the given hosts as the update plan.

 @param plan - Plan hosts of the copy.

 @returns A data value the file accepts.

 @example
 ```ts
 withPlan(['crates.io', 'pypi.org']);
 ```
 */
function withPlan({ plan, }: { readonly plan: readonly string[]; },): unknown {
  return {
    ...BUILD_LISTS,
    updateHosts: {
      ...BUILD_LISTS.updateHosts,
      plan,
    },
  };
}

/**
 Creates a scratch directory removed when the test ends.

 @returns Absolute path of the new directory.

 @example
 ```ts
 const root = await scratchDirectory();
 ```
 */
async function scratchDirectory(): Promise<string> {
  return await mkdtemp(join(scratchRoot(), 'wg-quicker-clearnet-hosts-',),);
}

/**
 Reports whether a thrown value is this module's parse error.

 @param error - Thrown value.

 @returns Whether the value names itself as the host-list error.

 @example
 ```ts
 isHostListError({ error });
 ```
 */
function isHostListError({ error, }: { readonly error: unknown; },): boolean {
  return (error instanceof Error) && (error.name === 'ClearnetHostsDataError');
}

await describe({
  name: '',
  children: [
    await describe({
      name: clearnetHostsFromData.name,
      children: [
        it({
          name: 'carries both lists, with every provider host derived from its recorded base URL',
          fn: async () => {
            /**
             Hosts the build lists name.
             */
            const hosts = clearnetHostsFromData({
              path: 'data.json',
              value: {
                schemaVersion: CLEARNET_HOSTS_DATA_SCHEMA_VERSION,
                updateHosts: {
                  plan: ['crates.io',],
                  redirect: ['index.crates.io',],
                  nonFetch: ['pnpm.io',],
                },
                providerBaseUrls: [
                  { provider: 'deepseek', baseUrl: 'https://api.deepseek.com', },
                  { provider: 'deepseek', baseUrl: 'https://api.deepseek.com/v1', },
                ],
              },
            },);
            expect(hosts,).toEqual([
              'crates.io',
              'index.crates.io',
              'api.deepseek.com',
            ],);
          },
        },),
        it({
          name: 'never carries a documentation-only host as a rule',
          fn: async () => {
            expect(clearnetHostsFromData({
              path: 'data.json',
              value: BUILD_LISTS,
            },),).not.toContain('pnpm.io',);
          },
        },),
        ...([
          ['a schema version this build does not read', { ...BUILD_LISTS, schemaVersion: 2, },],
          ['no update hosts field', { ...BUILD_LISTS, updateHosts: undefined, },],
          ['an empty plan', withPlan({ plan: [], },),],
          ['a plan of no update host and no provider host', {
            ...BUILD_LISTS,
            updateHosts: { plan: [], redirect: [], nonFetch: ['pnpm.io',], },
            providerBaseUrls: [],
          },],
          ['a host with a scheme', withPlan({ plan: ['https://crates.io',], },),],
          ['a host with a port', withPlan({ plan: ['crates.io:443',], },),],
          ['a host in capitals', withPlan({ plan: ['Crates.io',], },),],
          ['a single-label host', withPlan({ plan: ['crates',], },),],
          ['a label beginning with a hyphen', withPlan({ plan: ['-crates.io',], },),],
          ['a label ending with a hyphen', withPlan({ plan: ['crates-.io',], },),],
          ['an empty label', withPlan({ plan: ['crates..io',], },),],
          ['a host longer than the DNS limit', withPlan({ plan: [`${'a'.repeat(60,)}.`.repeat(5,) + 'io',], },),],
          ['a base URL that is not https', {
            ...BUILD_LISTS,
            providerBaseUrls: [{ provider: 'deepseek', baseUrl: 'http://api.deepseek.com', },],
          },],
          ['a base URL that is not a URL', {
            ...BUILD_LISTS,
            providerBaseUrls: [{ provider: 'deepseek', baseUrl: 'not a url', },],
          },],
          ['a provider entry without a base URL', {
            ...BUILD_LISTS,
            providerBaseUrls: [{ provider: 'deepseek', },],
          },],
          ['a data value that is not an object', [],],
        ] as const).map(function rejects([what, value,],) {
          return it({
            name: `refuses ${what}`,
            fn: async () => {
              /**
               Thrown value,
               which must name the list and the file.
               */
              let thrown: unknown;
              try {
                clearnetHostsFromData({ path: '/run/wg-quicker/data.json', value, },);
              }
              catch (error) {
                thrown = error;
              }
              expect(isHostListError({ error: thrown, },),).toBe(true,);
              expect(String(thrown,),).toContain('/run/wg-quicker/data.json',);
            },
          },);
        },),
      ],
    },),
    await describe({
      name: parseClearnetHostsData.name,
      children: [
        it({
          name: 'reads the same hosts from text',
          fn: async () => {
            expect(parseClearnetHostsData({
              path: 'data.json',
              text: JSON.stringify(BUILD_LISTS,),
            },),).toEqual(['crates.io', 'index.crates.io', 'api.deepseek.com',],);
          },
        },),
        it({
          name: 'refuses text that is not JSON',
          fn: async () => {
            /**
             Thrown value,
             which must name the list and the file.
             */
            let thrown: unknown;
            try {
              parseClearnetHostsData({ path: '/run/wg-quicker/data.json', text: '{', },);
            }
            catch (error) {
              thrown = error;
            }
            expect(isHostListError({ error: thrown, },),).toBe(true,);
            expect(String(thrown,),).toContain('/run/wg-quicker/data.json',);
          },
        },),
      ],
    },),
    await describe({
      name: defaultClearnetHostsDataPath.name,
      children: [
        it({
          name: 'names the data file the build copies beside the route watcher',
          fn: async () => {
            /**
             Absolute path of the copied data file.
             */
            const path = defaultClearnetHostsDataPath();
            expect(path.endsWith('/clearnet-hosts.data.json',),).toBe(true,);
            expect(path,).toContain('dist/final/node',);
          },
        },),
        it({
          name: 'reads the lists the build shipped, which are the lists this build compiled in',
          fn: async () => {
            /**
             Result of reading the copied file.
             */
            const read = await createInstalledClearnetHostsReader({},)();
            expect(read.kind,).toBe('read',);
            if (read.kind === 'read') {
              expect(read.hosts.toSorted(),).toEqual(CLEARNET_HOSTS.toSorted(),);
              expect(read.hosts,).toContain(UPDATE_HOSTS[0],);
              expect(read.hosts,).toContain(PROVIDER_HOSTS[0],);
            }
          },
        },),
      ],
    },),
    await describe({
      name: createInstalledClearnetHostsReader.name,
      children: [
        it({
          name: 'returns the new list of a data file that changed, without a restart',
          fn: async () => {
            /**
             Scratch directory holding the data file.
             */
            const root = await scratchDirectory();
            try {
              /**
               Data file the reader follows.
               */
              const path = join(root, 'clearnet-hosts.data.json',);
              await writeFile(path, JSON.stringify(withPlan({ plan: ['crates.io',], },),),);
              /**
               One reader, kept across both reads like a watcher keeps its own.
               */
              const read = createInstalledClearnetHostsReader({ path, },);
              /**
               First read, before the file changes.
               */
              const first = await read();
              await writeFile(path, JSON.stringify(withPlan({ plan: ['crates.io', 'pypi.org',], },),),);
              /**
               Second read of the same reader after the file changed.
               */
              const second = await read();
              expect(first.kind,).toBe('read',);
              expect(second.kind,).toBe('read',);
              if ((first.kind === 'read') && (second.kind === 'read')) {
                expect(first.hosts,).not.toContain('pypi.org',);
                expect(second.hosts,).toContain('pypi.org',);
              }
            }
            finally {
              await rm(root, { recursive: true, force: true, },);
            }
          },
        },),
        it({
          name: 'names a data file that is not there instead of returning an empty list',
          fn: async () => {
            /**
             Scratch directory without a data file.
             */
            const root = await scratchDirectory();
            try {
              /**
               Result of reading a path that holds nothing.
               */
              const read = await createInstalledClearnetHostsReader({
                path: join(root, 'absent.json',),
              },)();
              expect(read.kind,).toBe('failed',);
              if (read.kind === 'failed')
                expect(read.failure,).toContain('absent.json',);
            }
            finally {
              await rm(root, { recursive: true, force: true, },);
            }
          },
        },),
        it({
          name: 'refuses a symbolic link at the data path',
          fn: async () => {
            /**
             Scratch directory holding both the target and the link.
             */
            const root = await scratchDirectory();
            try {
              /**
               Regular file the link points at.
               */
              const target = join(root, 'real.json',);
              await writeFile(target, JSON.stringify(BUILD_LISTS,),);
              /**
               Link planted where the reader looks.
               */
              const link = join(root, 'clearnet-hosts.data.json',);
              await symlink(target, link,);
              /**
               Result of reading the link.
               */
              const read = await createInstalledClearnetHostsReader({ path: link, },)();
              expect(read.kind,).toBe('failed',);
              if (read.kind === 'failed')
                expect(read.failure,).toContain('symbolic link',);
            }
            finally {
              await rm(root, { recursive: true, force: true, },);
            }
          },
        },),
        it({
          name: 'refuses a path that is not a regular file',
          fn: async () => {
            /**
             Scratch directory holding a directory where a file is expected.
             */
            const root = await scratchDirectory();
            try {
              /**
               Path holding a directory, which no root task reads as a list.
               */
              const path = join(root, 'clearnet-hosts.data.json',);
              await mkdir(path,);
              /**
               Result of reading a directory.
               */
              const read = await createInstalledClearnetHostsReader({ path, },)();
              expect(read.kind,).toBe('failed',);
              if (read.kind === 'failed')
                expect(read.failure,).toContain('regular file',);
            }
            finally {
              await rm(root, { recursive: true, force: true, },);
            }
          },
        },),
        it({
          name: 'refuses a data file a group or another user could write',
          fn: async () => {
            /**
             Scratch directory holding a world-writable data file.
             */
            const root = await scratchDirectory();
            try {
              /**
               Data file anyone on the machine could replace.
               */
              const path = join(root, 'clearnet-hosts.data.json',);
              await writeFile(path, JSON.stringify(BUILD_LISTS,),);
              await chmod(path, 0o666,);
              /**
               Result of reading a data file anyone could have written.
               */
              const read = await createInstalledClearnetHostsReader({ path, },)();
              expect(read.kind,).toBe('failed',);
              if (read.kind === 'failed')
                expect(read.failure,).toContain('write',);
            }
            finally {
              await rm(root, { recursive: true, force: true, },);
            }
          },
        },),
        it({
          name: 'refuses a data file owned by an identity the tunnel does not run for',
          fn: async () => {
            /**
             Scratch directory holding a data file of this user.
             */
            const root = await scratchDirectory();
            try {
              /**
               Data file this test owns.
               */
              const path = join(root, 'clearnet-hosts.data.json',);
              await writeFile(path, JSON.stringify(BUILD_LISTS,),);
              /**
               Result of reading a data file of an owner nobody allowed.
               */
              const read = await createInstalledClearnetHostsReader({
                path,
                allowedOwnerUids: [999_999,],
              },)();
              expect(read.kind,).toBe('failed',);
              if (read.kind === 'failed')
                expect(read.failure,).toContain('uid',);
            }
            finally {
              await rm(root, { recursive: true, force: true, },);
            }
          },
        },),
      ],
    },),
  ],
},);
