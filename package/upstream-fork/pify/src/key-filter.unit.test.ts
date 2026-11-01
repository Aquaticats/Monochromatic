/**
 Tests for member selection: `include` / `exclude` patterns, the default
 `Sync` / `Stream` exclusion, descriptor handling, and the two upstream
 selection quirks (shared first-touch decision cache and `Object.prototype`
 key leakage), exercised through the public `pify` entry point.
 
 @module
 */

/* oxlint-disable promise/prefer-await-to-callbacks -- External-boundary mirror: this package wraps node-style callback functions, so its fixtures and wrapped-function declarations implement the callback pattern the tests exercise; see DECISION.callback-capture.md. */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pify, } from '../dist/final/neutral/index.mjs';

/**
 Classifies one member of a pified module by calling it with the fork's call
 shape.
 
 @param member - Member value read from a pified module.
 
 @returns `promisified` when the call returned a thenable, `raw` when it
 returned a plain value, or `throws <Name>` when it threw.
 */
function memberShape(member: unknown): string {
  try {
    /**
     Call result of the member with one harmless `{ args }` call shape.
     */
    const result = (member as (call: { readonly args: readonly unknown[]; }) => unknown)({
      args: [],
    },);
    /**
     Whether the call result exposes a `then` member: the promise-shape probe.
     */
    const resultThenable = (typeof (result as { then?: unknown; }).then) === 'function';
    return resultThenable ? 'promisified' : 'raw';
  }
  catch (error) {
    return `throws ${(error as Error).name}`;
  }
}

await describe({
  name: 'key-filter',
  children: [
    //region Pattern selection

    it({
      name: 'include selects only its members',
      fn: async () => {
        /**
         Module with three fixture members.
         */
        const module = {
          method1(callback: (error: unknown, value: unknown) => void): void {
            callback(null, '1',);
          },
          method2(callback: (error: unknown, value: unknown) => void): void {
            callback(null, '2',);
          },
          method3(): string {
            return '3';
          },
        };
        const pified = pify({
          input: module,
          options: {
            include: [
              'method1',
              'method2',
            ],
          },
        },);
        expect(memberShape(pified.method1,),).toBe('promisified',);
        expect(memberShape(pified.method2,),).toBe('promisified',);
        expect(memberShape(pified.method3,),).toBe('raw',);
      },
    },),

    it({
      name: 'exclude leaves its members untouched',
      fn: async () => {
        const module = {
          method1(callback: (error: unknown, value: unknown) => void): void {
            callback(null, '1',);
          },
          method3(): string {
            return '3';
          },
        };
        const pified = pify({
          input: module,
          options: {
            exclude: ['method3',],
          },
        },);
        expect(memberShape(pified.method1,),).toBe('promisified',);
        expect(memberShape(pified.method3,),).toBe('raw',);
      },
    },),

    it({
      name: 'include wins over exclude',
      fn: async () => {
        const module = {
          method1(callback: (error: unknown, value: unknown) => void): void {
            callback(null, '1',);
          },
          method2(callback: (error: unknown, value: unknown) => void): void {
            callback(null, '2',);
          },
        };
        const pified = pify({
          input: module,
          options: {
            include: [
              'method1',
              'method2',
            ],
            exclude: ['method2',],
          },
        },);
        expect(memberShape(pified.method1,),).toBe('promisified',);
        expect(memberShape(pified.method2,),).toBe('promisified',);
      },
    },),

    it({
      name: 'regexp patterns match member names',
      fn: async () => {
        const module = {
          loadAlpha(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'alpha',);
          },
          loadBeta(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'beta',);
          },
        };
        const pified = pify({
          input: module,
          options: {
            // oxlint-disable-next-line no-restricted-syntax/no-regex -- member-name suffix pattern under test is a bounded one-pass literal; mirrors upstream pify's documented `Array<string | RegExp>` include entries
            include: [/Alpha$/u,] as unknown as readonly (keyof typeof module)[],
          },
        },) as unknown as {
          loadAlpha: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>;
          loadBeta: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>;
        };
        expect(memberShape(pified.loadAlpha,),).toBe('promisified',);
        expect(memberShape(pified.loadBeta,),).toBe('throws TypeError',);
      },
    },),

    it({
      name: 'excludes Sync and Stream members by default',
      fn: async () => {
        const module = {
          read(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'p',);
          },
          readSync(): string {
            return 'raw';
          },
          createStream(): string {
            return 'raw';
          },
        };
        const pified = pify({
          input: module,
        },);
        expect(memberShape(pified.read,),).toBe('promisified',);
        expect(memberShape(pified.readSync,),).toBe('raw',);
        expect(memberShape(pified.createStream,),).toBe('raw',);
      },
    },),

    it({
      name: 'include overrides the default Sync suffix exclusion',
      fn: async () => {
        const module = {
          callbackEndingInSync(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'sync',);
          },
        };
        const pified = pify({
          input: module,
          options: {
            include: ['callbackEndingInSync',],
          },
        },);
        expect(memberShape(pified.callbackEndingInSync,),).toBe('promisified',);
      },
    },),

    //endregion Pattern selection

    //region Descriptors and keys

    it({
      name: 'leaves non-writable non-configurable own members raw',
      fn: async () => {
        /**
         Module whose own member is frozen against proxy invariants.
         */
        const module = {};
        Object.defineProperty(module, 'prop', {
          value(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'x',);
          },
          writable: false,
          configurable: false,
        },);
        const pified = pify({
          input: module,
        },);
        expect(
          (pified as unknown as Record<string, unknown>).prop,
        ).toBe((module as Record<string, unknown>).prop,);
      },
    },),

    it({
      name: 'promisifies symbol-keyed members',
      fn: async () => {
        /**
         Description carrying enough words for the symbol lint rule.
         */
        const symbolKey = Symbol('fixture member key for symbol test',);
        const module = {
          [symbolKey](callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'symbol',);
          },
        };
        const pified = pify({
          input: module,
        },);
        /**
         Promisified symbol-keyed member read through the proxy view.
         */
        const symbolMember = (pified as unknown as Record<symbol, unknown>)[symbolKey] as (
          call: { readonly args: readonly unknown[]; },
        ) => Promise<unknown>;
        expect(await symbolMember({
          args: [],
        },),).toBe('symbol',);
      },
    },),

    //endregion Descriptors and keys

    it({
      name: 'never matches symbol keys against patterns, even regexes',
      fn: async () => {
        /**
         Description carrying enough words for the symbol lint rule.
         */
        const symbolKey = Symbol('fixture member key for pattern test',);
        const module = {
          [symbolKey](callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'symbol',);
          },
        };
        const pified = pify({
          input: module,
          options: {
            // oxlint-disable-next-line no-restricted-syntax/no-regex -- pattern under test must match the symbol's string form so only the symbol-key short-circuit keeps the member selected
            exclude: [/^Symbol/u,] as unknown as readonly (keyof typeof module)[],
          },
        },);
        /**
         Symbol-keyed member through the proxy view: the exclude pattern
         matches its string form, but symbol keys compare by identity only,
         so the member stays selected.
         */
        const symbolMember = (pified as unknown as Record<symbol, unknown>)[symbolKey] as (
          call: { readonly args: readonly unknown[]; },
        ) => Promise<unknown>;
        expect(memberShape(symbolMember,),).toBe('promisified',);
      },
    },),

    it({
      name: 'promisifies writable but non-configurable own members',
      fn: async () => {
        /**
         Module whose own member is writable yet frozen against
         reconfiguration: the writable flag alone keeps it selectable.
         */
        const module: Record<string, unknown> = {};
        Object.defineProperty(module, 'prop', {
          value(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'x',);
          },
          writable: true,
          configurable: false,
        },);
        const pified = pify({
          input: module,
        },);
        expect(
          memberShape((pified as unknown as Record<string, unknown>).prop,),
        ).toBe('promisified',);
      },
    },),

    it({
      name: 'exclude drops a member when any one of several patterns matches',
      fn: async () => {
        const module = {
          read(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'p',);
          },
          other(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'o',);
          },
        };
        const pified = pify({
          input: module,
          options: {
            exclude: [
              'read',
              'never-matches-this',
            ] as unknown as readonly (keyof typeof module)[],
          },
        },);
        expect(memberShape(pified.read,),).toBe('throws TypeError',);
        expect(memberShape(pified.other,),).toBe('promisified',);
      },
    },),

    //region Upstream quirks

    it({
      name: 'quirk: Object.prototype keys bypass include and exclude entirely',
      fn: async () => {
        const module = {
          toString(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 't',);
          },
          constructor: (callback: (error: unknown, value: unknown) => void): void => {
            callback(null, 'c',);
          },
        };
        const pified = pify({
          input: module,
          options: {
            include: [],
          },
        },);
        expect(memberShape((pified as unknown as { toString: unknown; }).toString,),).toBe('promisified',);
        expect(
          memberShape((pified as unknown as { constructor: unknown; }).constructor,),
        ).toBe('promisified',);
      },
    },),

    it({
      name: 'quirk: the first pify call freezes each member decision for later calls',
      fn: async () => {
        const shared = {
          a(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'a',);
          },
          b(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'b',);
          },
        };
        /**
         First pify view: selects only `a`, touching both members.
         */
        const first = pify({
          input: shared,
          options: {
            include: ['a',],
          },
        },);
        expect(memberShape(first.a,),).toBe('promisified',);
        expect(memberShape(first.b,),).toBe('throws TypeError',);
        /**
         Second pify view over the same target: its own `include` says `b`,
         but the shared cache froze both decisions at first touch.
         */
        const second = pify({
          input: shared,
          options: {
            include: ['b',],
          },
        },);
        expect(memberShape(second.a,),).toBe('promisified',);
        expect(memberShape(second.b,),).toBe('throws TypeError',);
      },
    },),

    //endregion Upstream quirks
  ],
},);
