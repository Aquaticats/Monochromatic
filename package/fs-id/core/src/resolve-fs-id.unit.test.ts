import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  FsIdResolutionError,
  isFsId,
  resolveFsId,
  UnsupportedFsIdPlatformError,
} from '../dist/final/node/index.mjs';
import {
  createFsIdResolver,
  type FsIdCommand,
  type FsIdResolverAdapters,
} from '../dist/final/node/testing.mjs';

/**
 Fixture adapter construction input.
 
 @example
 ```ts
 const input: FixtureInput = { platform: 'linux', commandResults: ['uuid'] };
 ```
 */
type FixtureInput = {
  readonly platform: NodeJS.Platform;
  readonly commandResults: readonly (string | Error)[];
  readonly deviceResults?: readonly (string | Error)[];
  readonly canonicalPaths?: Readonly<Record<string, string>>;
};

/**
 Observable fixture state.
 
 @example
 ```ts
 fixture.commands.length;
 ```
 */
type Fixture = {
  readonly adapters: FsIdResolverAdapters;
  readonly commands: FsIdCommand[];
  readonly warnings: { readonly path: string; readonly reason: string; }[];
};

/**
 Returns queued value or throws queued error.
 
 @param values - Fixture result queue
 
 @param index - Result index
 
 @returns String result
 
 @throws queued error or exhaustion error
 
 @example
 ```ts
 fixtureValue({ values: ['ok'], index: 0 }); // 'ok'
 ```
 */
function fixtureValue({
  values,
  index,
}: {
  readonly values: readonly (string | Error)[];
  readonly index: number;
},): string {
  /**
   Queued result before error narrowing.
   */
  const value = values[index];
  if (value === undefined)
    throw new Error('fixture result queue exhausted',);
  if (Error.isError(value,))
    throw value;
  return value;
}

/**
 Creates deterministic platform adapters with observable commands and warnings.
 
 @param platform - Platform branch
 
 @param commandResults - Ordered subprocess results
 
 @param deviceResults - Ordered Node-stat results
 
 @param canonicalPaths - Optional canonicalization map
 
 @returns Fixture adapters and observations
 
 @example
 ```ts
 const fixture = createFixture({ platform: 'linux', commandResults: ['uuid'] });
 ```
 */
function createFixture({
  platform,
  commandResults,
  deviceResults = [],
  canonicalPaths = {},
}: FixtureInput,): Fixture {
  /**
   Captured subprocess calls.
   */
  const commands: FsIdCommand[] = [];
  /**
   Captured mandatory degraded warnings.
   */
  const warnings: { readonly path: string; readonly reason: string; }[] = [];
  /**
   Mutable fixture queue cursors kept in one test-state object.
   */
  const indexes = {
    command: 0,
    device: 0,
  };
  /**
   Effect adapters under test.
   */
  const adapters: FsIdResolverAdapters = {
    platform: () => platform,
    canonicalizePath: async ({ path, }) => canonicalPaths[path] ?? path,
    run: async (command,) => {
      commands.push(command,);
      /**
       Queued command result selected before increment.
       */
      /**
       Current command result index advanced before queued error can throw.
       */
      const index = indexes.command;
      indexes.command += 1;
      return fixtureValue({
        values: commandResults,
        index,
      },);
    },
    deviceNumber: async () => {
      /**
       Queued device result selected before increment.
       */
      /**
       Current device result index advanced before queued error can throw.
       */
      const index = indexes.device;
      indexes.device += 1;
      return fixtureValue({
        values: deviceResults,
        index,
      },);
    },
    warn: warning => {
      warnings.push(warning,);
    },
  };
  return { adapters, commands, warnings, };
}

await describe({
  name: createFsIdResolver.name,
  children: [
    //region Linux

    it({
      name: 'resolves stable Linux filesystem UUID',
      fn: async () => {
        /**
         Stable Linux fixture.
         */
        const fixture = createFixture({ platform: 'linux', commandResults: ['ABCD-1234\n',], },);
        /**
         Resolved identity.
         */
        const result = await createFsIdResolver({ adapters: fixture.adapters, })({ path: '/repo', },);
        expect(result,).toEqual({ value: 'fs-uuid_abcd-1234', stable: true, source: 'fs-uuid', },);
        expect(fixture.commands,).toEqual([{
          command: 'findmnt',
          args: ['--target', '/repo', '--output=UUID', '--noheadings',],
        },],);
        expect(fixture.warnings,).toEqual([],);
      },
    },),
    it({
      name: 'degrades Linux failure to warned f_fsid',
      fn: async () => {
        /**
         Linux fallback fixture.
         */
        const fixture = createFixture({
          platform: 'linux',
          commandResults: [new Error('findmnt failed',), 'A281DFD5\n',],
        },);
        /**
         Degraded identity.
         */
        const result = await createFsIdResolver({ adapters: fixture.adapters, })({ path: '/repo', },);
        expect(result.value,).toBe('f-fsid_a281dfd5',);
        expect(result.stable,).toBe(false,);
        expect(result.source,).toBe('f-fsid',);
        expect(result.reason,).toContain('findmnt failed',);
        expect(fixture.warnings,).toHaveLength(1,);
      },
    },),
    it({
      name: 'throws when Linux preferred and degraded mechanisms fail',
      fn: async () => {
        /**
         Total Linux failure fixture.
         */
        const fixture = createFixture({
          platform: 'linux',
          commandResults: [new Error('preferred',), new Error('fallback',),],
        },);
        /**
         Captured total failure.
         */
        let caught: unknown;
        try {
          await createFsIdResolver({ adapters: fixture.adapters, })({ path: '/repo', },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(FsIdResolutionError,);
      },
    },),

    //endregion Linux

    //region macOS

    it({
      name: 'resolves stable macOS Volume UUID',
      fn: async () => {
        /**
         Stable macOS fixture.
         */
        const fixture = createFixture({
          platform: 'darwin',
          commandResults: [
            'Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/disk3s1 10 5 5 50% /\n',
            '<?xml version="1.0"?><plist><dict><key>VolumeUUID</key><string>ABCD-1234</string></dict></plist>',
          ],
        },);
        /**
         Stable identity.
         */
        const result = await createFsIdResolver({ adapters: fixture.adapters, })({ path: '/repo', },);
        expect(result,).toEqual({ value: 'volume-uuid_abcd-1234', stable: true, source: 'volume-uuid', },);
        expect(fixture.commands,).toEqual([
          { command: 'df', args: ['-P', '/repo',], },
          { command: 'diskutil', args: ['info', '-plist', '/dev/disk3s1',], },
        ],);
      },
    },),
    it({
      name: 'degrades macOS missing UUID to warned device number',
      fn: async () => {
        /**
         macOS fallback fixture.
         */
        const fixture = createFixture({
          platform: 'darwin',
          commandResults: [
            'Filesystem Mounted on\n/dev/disk3s1 /\n',
            '<plist><dict></dict></plist>',
            '2049\n',
          ],
        },);
        /**
         Degraded identity.
         */
        const result = await createFsIdResolver({ adapters: fixture.adapters, })({ path: '/repo', },);
        expect(result.value,).toBe('device-number_2049',);
        expect(result.stable,).toBe(false,);
        expect(fixture.commands[2],).toEqual({ command: 'stat', args: ['-f', '%d', '/repo',], },);
        expect(fixture.warnings,).toHaveLength(1,);
      },
    },),
    it({
      name: 'throws when macOS preferred and degraded mechanisms fail',
      fn: async () => {
        /**
         Total macOS failure fixture.
         */
        const fixture = createFixture({
          platform: 'darwin',
          commandResults: [new Error('preferred',), new Error('fallback',),],
        },);
        /**
         Captured total failure.
         */
        let caught: unknown;
        try {
          await createFsIdResolver({ adapters: fixture.adapters, })({ path: '/repo', },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(FsIdResolutionError,);
      },
    },),

    //endregion macOS

    //region Windows

    it({
      name: 'resolves stable Windows volume serial with safe command text',
      fn: async () => {
        /**
         Stable Windows fixture.
         */
        const fixture = createFixture({
          platform: 'win32',
          commandResults: ['1A2B-3C4D\r\n',],
        },);
        /**
         Stable identity.
         */
        const result = await createFsIdResolver({ adapters: fixture.adapters, })({ path: String.raw`c:\repo`, },);
        expect(result,).toEqual({
          value: 'volume-serial_1a2b-3c4d',
          stable: true,
          source: 'volume-serial',
        },);
        expect(fixture.commands,).toEqual([{
          command: 'powershell.exe',
          args: [
            '-NoLogo',
            '-NoProfile',
            '-NonInteractive',
            '-Command',
            `(Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='C:'").VolumeSerialNumber`,
          ],
        },],);
      },
    },),
    it({
      name: 'degrades Windows serial failure to warned device number',
      fn: async () => {
        /**
         Windows fallback fixture.
         */
        const fixture = createFixture({
          platform: 'win32',
          commandResults: [new Error('vol failed',),],
          deviceResults: ['99',],
        },);
        /**
         Degraded identity.
         */
        const result = await createFsIdResolver({ adapters: fixture.adapters, })({ path: String.raw`C:\repo`, },);
        expect(result.value,).toBe('device-number_99',);
        expect(result.stable,).toBe(false,);
        expect(fixture.warnings,).toHaveLength(1,);
      },
    },),
    it({
      name: 'rejects zero Windows device fallback that cannot distinguish volumes',
      fn: async () => {
        /**
         Unusable Windows device fixture.
         */
        const fixture = createFixture({
          platform: 'win32',
          commandResults: [new Error('preferred',),],
          deviceResults: ['0',],
        },);
        /**
         Captured unusable fallback.
         */
        let caught: unknown;
        try {
          await createFsIdResolver({ adapters: fixture.adapters, })({ path: String.raw`C:\repo`, },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(FsIdResolutionError,);
        expect(fixture.warnings,).toHaveLength(0,);
      },
    },),
    it({
      name: 'throws when Windows preferred and degraded mechanisms fail',
      fn: async () => {
        /**
         Total Windows failure fixture.
         */
        const fixture = createFixture({
          platform: 'win32',
          commandResults: [new Error('preferred',),],
          deviceResults: [new Error('fallback',),],
        },);
        /**
         Captured total failure.
         */
        let caught: unknown;
        try {
          await createFsIdResolver({ adapters: fixture.adapters, })({ path: String.raw`C:\repo`, },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(FsIdResolutionError,);
      },
    },),

    //endregion Windows

    //region Dispatch and memoization

    it({
      name: 'throws on unsupported platform without spawning',
      fn: async () => {
        /**
         Unsupported fixture.
         */
        const fixture = createFixture({ platform: 'aix', commandResults: [], },);
        /**
         Captured unsupported-platform error.
         */
        let caught: unknown;
        try {
          await createFsIdResolver({ adapters: fixture.adapters, })({ path: '/repo', },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(UnsupportedFsIdPlatformError,);
        expect(fixture.commands,).toEqual([],);
      },
    },),
    it({
      name: 'resolves aliases and distinct mount paths independently',
      fn: async () => {
        /**
         Two-mount fixture with one alias.
         */
        const fixture = createFixture({
          platform: 'linux',
          commandResults: ['UUID-A', 'UUID-A', 'UUID-B',],
          canonicalPaths: {
            '/alias-a': '/mount/a',
            '/mount/a': '/mount/a',
            '/mount/b': '/mount/b',
          },
        },);
        /**
         Memoized resolver.
         */
        const resolver = createFsIdResolver({ adapters: fixture.adapters, },);
        /**
         Alias resolution.
         */
        const alias = await resolver({ path: '/alias-a', },);
        /**
         Canonical duplicate resolution.
         */
        const duplicate = await resolver({ path: '/mount/a', },);
        /**
         Distinct mount resolution.
         */
        const other = await resolver({ path: '/mount/b', },);
        expect(alias,).toEqual(duplicate,);
        expect(other.value,).toBe('fs-uuid_uuid-b',);
        expect(fixture.commands,).toHaveLength(3,);
      },
    },),
    it({
      name: 'observes a replacement volume mounted at same canonical path',
      fn: async () => {
        /**
         Same-path replacement fixture.
         */
        const fixture = createFixture({
          platform: 'linux',
          commandResults: ['UUID-A', 'UUID-B',],
        },);
        /**
         Fresh resolver.
         */
        const resolver = createFsIdResolver({ adapters: fixture.adapters, },);
        /**
         Identity before simulated replacement.
         */
        const before = await resolver({ path: '/mount', },);
        /**
         Identity after simulated replacement.
         */
        const after = await resolver({ path: '/mount', },);
        expect(before.value,).toBe('fs-uuid_uuid-a',);
        expect(after.value,).toBe('fs-uuid_uuid-b',);
        expect(fixture.commands,).toHaveLength(2,);
      },
    },),
    it({
      name: 'retries failed resolution on later call',
      fn: async () => {
        /**
         Failure then success fixture.
         */
        const fixture = createFixture({
          platform: 'linux',
          commandResults: [new Error('preferred',), new Error('fallback',), 'RECOVERED',],
        },);
        /**
         Memoized resolver.
         */
        const resolver = createFsIdResolver({ adapters: fixture.adapters, },);
        try {
          await resolver({ path: '/repo', },);
        }
        catch (error) {
          expect(error,).toBeInstanceOf(FsIdResolutionError,);
        }
        /**
         Successful retry.
         */
        const recovered = await resolver({ path: '/repo', },);
        expect(recovered.value,).toBe('fs-uuid_recovered',);
        expect(fixture.commands,).toHaveLength(3,);
      },
    },),

    //endregion Dispatch and memoization

    //region Built consumer boundary

    it({
      name: 'resolves current repository through built production artifact',
      fn: async () => {
        /**
         Real host resolution.
         */
        const result = await resolveFsId({ path: process.cwd(), },);
        expect(isFsId(result.value,),).toBe(true,);
        expect(result.value.includes(':',),).toBe(false,);
        expect(typeof result.stable,).toBe('boolean',);
      },
    },),

    //endregion Built consumer boundary
  ],
},);
