/**
 Tests for SemVer ordering and Cargo caret compatibility.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  caretCompatible,
  compareVersions,
  parseVersion,
  VersionSyntaxError,
} from './version-order.ts';

/**
 Sign of a comparison.

 @param left - first version

 @param right - second version

 @returns -1, 0, or 1

 @example
 ```ts
 order({ left: '1.0.0', right: '2.0.0', }); // -1
 ```
 */
function order({ left, right, }: { readonly left: string; readonly right: string; },): number {
  return Math.sign(compareVersions({ left, right, },),);
}

await describe({
  name: parseVersion.name,
  children: [
    it({
      name: 'drops build metadata and splits prerelease identifiers',
      fn: async () => {
        expect(parseVersion('0.11.1+wasi-snapshot-preview1',),).toEqual({ major: 0, minor: 11, patch: 1, prerelease: [], },);
        expect(parseVersion('1.0.0-rc.1+build.5',),).toEqual({ major: 1, minor: 0, patch: 0, prerelease: ['rc', '1',], },);
        expect(parseVersion('0.38.0+1.3.281',),).toEqual({ major: 0, minor: 38, patch: 0, prerelease: [], },);
      },
    },),
    it({
      name: 'rejects text that is not SemVer',
      fn: async () => {
        for (const bad of ['', '1', '1.2', '1.2.3.4', '01.2.3', '1.2.x', '1.2.3-', '1.2.3-a..b', '-1.2.3', '1.2.3 ', ' 1.2.3', '1.2.3\n', '9007199254740993.0.0', '1.2.3/../4',]) {
          expect(function parseBad(): void {
            parseVersion(bad,);
          },).toThrow(VersionSyntaxError,);
        }
      },
    },),
  ],
},);

await describe({
  name: compareVersions.name,
  children: [
    it({
      name: 'orders cores numerically, not as text',
      fn: async () => {
        expect(order({ left: '1.9.0', right: '1.10.0', },),).toBe(-1,);
        expect(order({ left: '0.2.10', right: '0.2.9', },),).toBe(1,);
        expect(order({ left: '2.0.0', right: '2.0.0', },),).toBe(0,);
      },
    },),
    it({
      name: 'ranks a prerelease below its release and orders identifiers by SemVer rules',
      fn: async () => {
        expect(order({ left: '1.0.0-rc.1', right: '1.0.0', },),).toBe(-1,);
        expect(order({ left: '1.0.0-alpha', right: '1.0.0-alpha.1', },),).toBe(-1,);
        expect(order({ left: '1.0.0-alpha.1', right: '1.0.0-alpha.beta', },),).toBe(-1,);
        expect(order({ left: '1.0.0-beta.2', right: '1.0.0-beta.11', },),).toBe(-1,);
        expect(order({ left: '1.0.0-beta.11', right: '1.0.0-rc.1', },),).toBe(-1,);
      },
    },),
    it({
      name: 'ignores build metadata',
      fn: async () => {
        expect(order({ left: '0.11.1+wasi-snapshot-preview1', right: '0.11.1', },),).toBe(0,);
        expect(order({ left: '0.14.2+wasi-0.2.4', right: '0.11.1+wasi-snapshot-preview1', },),).toBe(1,);
      },
    },),
  ],
},);

await describe({
  name: caretCompatible.name,
  children: [
    it({
      name: 'keeps the major for 1.x, the minor for 0.x, and the exact patch for 0.0.x',
      fn: async () => {
        expect(caretCompatible({ version: '1.4.2', candidate: '1.0.0', },),).toBe(true,);
        expect(caretCompatible({ version: '2.0.0', candidate: '1.9.9', },),).toBe(false,);
        expect(caretCompatible({ version: '0.4.2', candidate: '0.4.0', },),).toBe(true,);
        expect(caretCompatible({ version: '0.4.2', candidate: '0.3.9', },),).toBe(false,);
        expect(caretCompatible({ version: '0.0.3', candidate: '0.0.2', },),).toBe(false,);
        expect(caretCompatible({ version: '0.0.3', candidate: '0.0.3+build', },),).toBe(true,);
      },
    },),
  ],
},);
