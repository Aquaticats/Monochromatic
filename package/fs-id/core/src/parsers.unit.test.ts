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

import {
  assertFsId,
  isFsId,
} from '../dist/final/node/index.mjs';
import {
  createFsId,
  normalizeIdentityPayload,
  parseDfDevice,
  parseDiskutilVolumeUuid,
  parseFindmntUuid,
  parseVolumeSerial,
  windowsDriveRoot,
} from '../dist/final/node/testing.mjs';

/**
 Run length proving parser uses bounded-stack iteration.
 
 @example
 ```ts
 'A'.repeat(LONG_RUN);
 ```
 */
const LONG_RUN = 50_000;

/**
 Unsafe payload examples rejected by identity grammar.
 
 @example
 ```ts
 UNSAFE_PAYLOADS.includes('a:b');
 ```
 */
const UNSAFE_PAYLOADS: readonly string[] = [
  '',
  '-',
  'a:b',
  'a/b',
  String.raw`a\b`,
  'a b',
  'é',
  'a'.repeat(513,),
];

/**
 ASCII serial terminators and expected prefix.
 
 @example
 ```ts
 SERIAL_TERMINATORS[0];
 ```
 */
const SERIAL_TERMINATORS: readonly string[] = [
  ' ',
  '\t',
  '\n',
  '\r',
  '\f',
  '\v',
];

/**
 Source directory holding the shipped modules the loop-shape guard reads.
 
 Resolved from this file so the guard follows the package wherever it moves;
 the built bundle cannot answer the question, because bundling rewrites the
 loop forms the guard looks for.
 */
const SOURCE_DIR = import.meta.dirname;

/**
 Suffix marking a module that only runs tests.
 
 The guard covers shipped modules: test modules count rows and run lengths on
 purpose, and the guard's own header constant is itself such a literal.
 */
const TEST_MODULE_SUFFIX = '.test.ts';

/**
 Opening header of a counted loop with a mutable counter binding.
 
 A counted loop is the sanctioned form for an indexed or lookahead scan, so a
 justified one edits this guard: it fails until the module states why the
 counter is needed.
 */
const COUNTED_LOOP_HEADER = 'for (let';

/**
 Module whose device-path and field search the issue under this guard cites.
 */
const GUARDED_MODULE = 'parsers.ts';

/**
 Names of the shipped TypeScript modules in this package's source directory.
 
 @returns Module file names in sorted order, so a failure message is stable
 */
async function shippedModuleNames(): Promise<string[]> {
  /**
   Directory entries in filesystem order.
   */
  const entries = await readdir(SOURCE_DIR,);
  return entries.filter(function isShippedModule(name,): boolean {
    if (!name.endsWith('.ts',))
      return false;
    return !name.endsWith(TEST_MODULE_SUFFIX,);
  },)
    .toSorted();
}

/**
 Lists where one module text opens a counted loop.
 
 @param fileName - Module name repeated in each position
 
 @param source - Module text to scan
 
 @returns `name:line` positions in source order, empty when the text holds none
 
 @example
 ```ts
 countedLoopSites({ fileName: 'a.ts', source: 'for (let i = 0;' });
 ```
 */
function countedLoopSites({
  fileName,
  source,
}: {
  readonly fileName: string;
  readonly source: string;
},): string[] {
  return source.split('\n',)
    .flatMap(function sitesInLine(line, lineIndex,) {
      if (!line.includes(COUNTED_LOOP_HEADER,))
        return [];
      return [`${fileName}:${(lineIndex + 1)}`];
    },);
}

await describe({
  name: '',
  children: [
    describe({
      name: normalizeIdentityPayload.name,
      children: [
        it({
          name: 'normalizes safe ASCII payload',
          fn: async () => {
            expect(normalizeIdentityPayload(' 1A2B-3C4D\n',),).toBe('1a2b-3c4d',);
          },
        },),
        ...UNSAFE_PAYLOADS.map(function unsafePayloadTest(payload,) {
          return it({
            name: `rejects unsafe payload ${JSON.stringify(payload.slice(0, 20,),)}`,
            fn: async () => {
              expect(() => normalizeIdentityPayload(payload,),).toThrow(TypeError,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: createFsId.name,
      children: [
        it({
          name: 'qualifies normalized payload with source',
          fn: async () => {
            expect(createFsId({ source: 'fs-uuid', payload: 'ABCD', },),).toBe('fs-uuid_abcd',);
          },
        },),
        it({
          name: 'throws for unsafe payload',
          fn: async () => {
            expect(() => createFsId({ source: 'fs-uuid', payload: 'a:b', },),).toThrow(TypeError,);
          },
        },),
      ],
    },),
    describe({
      name: assertFsId.name,
      children: [
        it({
          name: 'narrows valid generated identity and rejects invalid value',
          fn: async () => {
            expect(() => assertFsId('fs-uuid_abcd',),).not.toThrow();
            expect(() => assertFsId('fs-uuid_a:b',),).toThrow(TypeError,);
          },
        },),
      ],
    },),
    describe({
      name: isFsId.name,
      children: [
        it({
          name: 'accepts every generated source prefix',
          fn: async () => {
            expect([
              'fs-uuid_abcd',
              'volume-uuid_abcd',
              'volume-serial_1a2b-3c4d',
              'f-fsid_abcd',
              'device-number_1',
            ].every(function valueIsFsId(value,): boolean {
              return isFsId(value,);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects unknown prefix and colon',
          fn: async () => {
            expect(isFsId('unknown_abcd',),).toBe(false,);
            expect(isFsId('fs-uuid_a:b',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: parseFindmntUuid.name,
      children: [
        it({
          name: 'parses trimmed UUID',
          fn: async () => {
            expect(parseFindmntUuid(' ABCD-1234\n',),).toBe('abcd-1234',);
          },
        },),
        it({
          name: 'rejects absent UUID marker',
          fn: async () => {
            expect(() => parseFindmntUuid('-\n',),).toThrow(TypeError,);
          },
        },),
      ],
    },),
    describe({
      name: parseDfDevice.name,
      children: [
        it({
          name: 'parses device field independent of localized headers and mount spaces',
          fn: async () => {
            expect(parseDfDevice(
              'Système 1024-blocs Utilisé Disponible Capacité Monté sur\n'
                + '/dev/disk3s1 10 5 5 50% /Volumes/Name With Spaces\n',
            ),).toBe('/dev/disk3s1',);
          },
        },),
        it({
          name: 'reads a tab-separated row as the same device field',
          fn: async () => {
            expect(parseDfDevice('/dev/disk4s2\t10\t5\t5\t50%\t/Volumes/Data\n',),)
              .toBe('/dev/disk4s2',);
          },
        },),
        it({
          name: 'rejects absent non-device and unsafe device rows',
          fn: async () => {
            expect(() => parseDfDevice('Filesystem Mounted on\noverlay /\n',),).toThrow(TypeError,);
            expect(() => parseDfDevice('Header\n/dev/disk;bad 1 1 0 /\n',),).toThrow(TypeError,);
          },
        },),
      ],
    },),
    describe({
      name: parseDiskutilVolumeUuid.name,
      children: [
        it({
          name: 'parses invariant VolumeUUID plist key independent of display language',
          fn: async () => {
            expect(parseDiskutilVolumeUuid(
              '<plist><dict><key>Localized Display Name</key><string>Volumen</string>'
                + '<key>VolumeUUID</key><string>ABCD-1234</string></dict></plist>',
            ),).toBe('abcd-1234',);
          },
        },),
        it({
          name: 'rejects absent malformed unterminated and unsafe plist values',
          fn: async () => {
            expect(() => parseDiskutilVolumeUuid('<plist><dict></dict></plist>',),).toThrow(TypeError,);
            expect(() => parseDiskutilVolumeUuid('<key>VolumeUUID</key><integer>1</integer>',),)
              .toThrow(TypeError,);
            expect(() => parseDiskutilVolumeUuid('<key>VolumeUUID</key><string>abcd',),)
              .toThrow(TypeError,);
            expect(() => parseDiskutilVolumeUuid('<key>VolumeUUID</key><string>a:b</string>',),)
              .toThrow(TypeError,);
          },
        },),
      ],
    },),
    describe({
      name: parseVolumeSerial.name,
      children: [
        it({
          name: 'returns empty when label is absent or has no token',
          fn: async () => {
            expect(parseVolumeSerial('',),).toBe('',);
            expect(parseVolumeSerial('Volume has no label',),).toBe('',);
            expect(parseVolumeSerial('Serial Number is  \t',),).toBe('',);
          },
        },),
        it({
          name: 'matches case-insensitively and skips inline whitespace',
          fn: async () => {
            expect(parseVolumeSerial('SERIAL NUMBER IS \t 1A2B-3C4D\r\n',),).toBe('1A2B-3C4D',);
          },
        },),
        ...SERIAL_TERMINATORS.map(function serialTerminatorTest(terminator,) {
          return it({
            name: `stops at ${JSON.stringify(terminator,)}`,
            fn: async () => {
              expect(parseVolumeSerial(`Serial Number is AB${terminator}CD`,),).toBe('AB',);
            },
          },);
        },),
        it({
          name: 'handles long token and whitespace without recursion',
          fn: async () => {
            /**
             Long serial payload.
             */
            const token = 'A'.repeat(LONG_RUN,);
            expect(parseVolumeSerial(`Serial Number is ${token}`,),).toBe(token,);
            expect(parseVolumeSerial(`Serial Number is${' '.repeat(LONG_RUN,)}XY`,),).toBe('XY',);
          },
        },),
      ],
    },),
    describe({
      name: windowsDriveRoot.name,
      children: [
        it({
          name: 'normalizes slash and letter case',
          fn: async () => {
            expect(windowsDriveRoot('c:/repo',),).toBe('C:\\',);
            expect(windowsDriveRoot(String.raw`D:\repo`,),).toBe('D:\\',);
          },
        },),
        it({
          name: 'rejects relative UNC and command-shaped roots',
          fn: async () => {
            expect(() => windowsDriveRoot('repo',),).toThrow();
            expect(() => windowsDriveRoot(String.raw`\\server\share`,),).toThrow();
            expect(() => windowsDriveRoot('C:&calc',),).toThrow();
          },
        },),
      ],
    },),
    describe({
      name: 'loop shape',
      children: [
        it({
          name: 'positive control: the guard names a counted loop it is given',
          fn: async () => {
            /**
             Text holding one counted loop on its first line.
             */
            const sample = `${COUNTED_LOOP_HEADER} index = 0; index < 1; index += 1) {}\n`;
            expect(countedLoopSites({
              fileName: 'sample.ts',
              source: sample,
            },),).toEqual(['sample.ts:1',],);
          },
        },),
        it({
          name: 'holds no counted loop without an index need',
          fn: async () => {
            /**
             Shipped modules this package compiles.
             */
            const names = await shippedModuleNames();
            // Positive control: the scan reaches the module whose loops the
            // issue cites, so an empty offender list means this package has
            // none rather than that the directory read nothing.
            expect(names,).toContain(GUARDED_MODULE,);
            /**
             `name:line` positions of counted loops, one entry per module.
             */
            const positionsPerModule = await Promise.all(names.map(
              async function modulePositions(name,) {
                /**
                 Module text as written in `src`.
                 */
                const source = await readFile(
                  join(
                    SOURCE_DIR,
                    name,
                  ),
                  'utf8',
                );
                return countedLoopSites({
                  fileName: name,
                  source,
                },);
              },
            ),);
            expect(positionsPerModule.flat(),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
