import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { RECOMMENDED_EXEMPT_MARK, } from '../dist/final/node/exempt-mark-reservation.mjs';
import { readClearnet, } from '../dist/final/node/tunnel-clearnet.mjs';
import {
  normalizeAddress,
  type BypassState,
  type HostRuleTools,
} from '../dist/final/node/tunnel-bypass-hosts.mjs';

/**
 Protocol number the rules of this lifecycle carry.
 */
const OWNED_PROTOCOL = '201';

/**
 State claiming a table and a preference for the interface.
 */
const STATE: BypassState = {
  version: 2,
  interfaceName: 'wgtest',
  mark: RECOMMENDED_EXEMPT_MARK,
  table: 52_000,
  preference: 50,
  ownerId: 'owner',
  routes: [],
};

/**
 Listing of one family:
 the rule of the mark,
 two owned destinations,
 and look-alikes that do not belong to this lifecycle.

 @param owned - Destinations of the owned rules.

 @param foreign - Destination of a rule in another table.

 @returns `ip -json rule show` text.

 @example
 ```ts
 listing({ owned: ['203.0.113.7'], foreign: '203.0.113.9' });
 ```
 */
function listing({ owned, foreign, }: { readonly owned: readonly string[]; readonly foreign: string; },): string {
  return JSON.stringify([
    {
      priority: STATE.preference,
      src: 'all',
      fwmark: `0x${STATE.mark.toString(16,)}`,
      table: String(STATE.table,),
      protocol: OWNED_PROTOCOL,
    },
    ...owned.map((dst,) => ({
      priority: STATE.preference,
      src: 'all',
      dst,
      table: String(STATE.table,),
      protocol: OWNED_PROTOCOL,
    }),),
    {
      priority: STATE.preference,
      src: 'all',
      dst: foreign,
      table: String(STATE.table + 1,),
      protocol: OWNED_PROTOCOL,
    },
  ],);
}

/**
 Rule listing that answers per family and records what was asked.

 @param asked - Families asked for,
 in order.

 @returns The tool.

 @example
 ```ts
 listingTools({ asked: [] });
 ```
 */
function listingTools({ asked, }: { readonly asked: string[]; },): HostRuleTools {
  return {
    list: ({ family, },) => {
      asked.push(family,);
      return Promise.resolve(family === '-4'
        ? listing({ owned: ['203.0.113.7', '203.0.113.8',], foreign: '198.51.100.1', },)
        : listing({ owned: ['2001:db8::7',], foreign: '2001:db8::9', },),);
    },
    add: () => Promise.resolve(),
    remove: () => Promise.resolve(),
  };
}

/**
 A runtime directory of the ownership state that is empty,
 where the real reader finds no state.
 The absence is a symbol that each separately built bundle creates for itself,
 so a test cannot hand one to the bundle under test.
 */
type EmptyRuntimeDirectory = {
  /**
   Path of the empty directory.
   */
  readonly path: string;

  /**
   Puts the environment back and removes the directory.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Points the runtime directory of the ownership state at a new empty directory until the scope ends.

 @returns The directory,
 disposed by `await using`.

 @example
 ```ts
 await using runtime = await emptyRuntimeDirectory();
 ```
 */
async function emptyRuntimeDirectory(): Promise<EmptyRuntimeDirectory> {
  /**
   Empty directory standing for the runtime directory.
   */
  const path = await mkdtemp(join(scratchRoot(), 'wg-quicker-clearnet-',),);
  /**
   Value to put back.
   */
  const previous = process.env.WG_QUICKER_RUNTIME_DIRECTORY;
  process.env.WG_QUICKER_RUNTIME_DIRECTORY = path;
  return {
    path,
    [Symbol.asyncDispose]: async function restore(): Promise<void> {
      if (previous === undefined)
        delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
      else
        process.env.WG_QUICKER_RUNTIME_DIRECTORY = previous;
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Reads what a call throws.

 @param run - Call expected to throw.

 @returns The thrown value,
 or nothing when it did not throw.

 @example
 ```ts
 await thrownBy({ run: () => Promise.reject(new Error('x')) });
 ```
 */
async function thrownBy({ run, }: { readonly run: () => unknown; },): Promise<unknown> {
  try {
    await run();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: readClearnet.name,
  children: [
    it({
      name: 'returns nothing, and lists no rules, when there is no ownership state',
      fn: async () => {
        /**
         Families asked for.
         */
        const asked: string[] = [];
        /**
         What was read.
         */
        await using runtime = await emptyRuntimeDirectory();
        expect(process.env.WG_QUICKER_RUNTIME_DIRECTORY,).toBe(runtime.path,);
        const read = await readClearnet({
          interfaceName: STATE.interfaceName,
          tools: listingTools({ asked, },),
        },);
        expect(read,).toEqual({ bypass: false, },);
        expect(asked,).toEqual([],);
      },
    },),
    it({
      name: 'returns the mark of the state and the owned destinations of both families, and none of another table',
      fn: async () => {
        /**
         Families asked for.
         */
        const asked: string[] = [];
        /**
         What was read.
         */
        const read = await readClearnet({
          interfaceName: STATE.interfaceName,
          readState: () => Promise.resolve(STATE,),
          tools: listingTools({ asked, },),
        },);
        expect(read.bypass,).toBe(true,);
        expect(read.bypass ? read.clearnet.mark : undefined,).toBe(STATE.mark,);
        expect(read.bypass ? read.clearnet.hosts : [],).toEqual([
          { family: '-4', address: '203.0.113.7', },
          { family: '-4', address: '203.0.113.8', },
          { family: '-6', address: normalizeAddress({ family: '-6', text: '2001:db8::7', },), },
        ],);
        expect(asked.toSorted(),).toEqual(['-4', '-6',],);
      },
    },),
    it({
      name: 'returns the physical default interfaces of the routes the state owns, once per family, and none from a route without a valid dev',
      fn: async () => {
        /**
         Owned routes of the state, listed the way the kernel renders them.
         */
        const state: BypassState = {
          ...STATE,
          routes: [
            { proto: '-4', tokens: ['default', 'via', '192.0.2.1', 'dev', 'eth0', 'table', '52000', 'proto', '201',], },
            { proto: '-4', tokens: ['default', 'via', '192.0.2.1', 'dev', 'eth0', 'metric', '700', 'table', '52000', 'proto', '201',], },
            { proto: '-4', tokens: ['default', 'via', '198.51.100.1', 'dev', 'wlp9s0', 'table', '52000', 'proto', '201',], },
            { proto: '-6', tokens: ['default', 'via', 'fe80::1', 'dev', 'eth0', 'table', '52000', 'proto', '201',], },
            { proto: '-6', tokens: ['unreachable', 'default', 'metric', '42760', 'table', '52000', 'proto', '201',], },
            { proto: '-6', tokens: ['default', 'dev', 'bad;name', 'table', '52000', 'proto', '201',], },
            { proto: '-6', tokens: ['default', 'table', '52000', 'proto', '201', 'dev',], },
          ],
        };
        /**
         What was read.
         */
        const read = await readClearnet({
          interfaceName: STATE.interfaceName,
          readState: () => Promise.resolve(state,),
          tools: listingTools({ asked: [], },),
        },);
        expect(read.bypass ? read.clearnet.interfaces : [],).toEqual([
          { family: '-4', name: 'eth0', },
          { family: '-4', name: 'wlp9s0', },
          { family: '-6', name: 'eth0', },
        ],);
      },
    },),
    it({
      name: 'rejects, and does not answer with nothing, when the state or the rules cannot be read',
      fn: async () => {
        /**
         Failure of the state reader.
         */
        const stateFailure = await thrownBy({
          run: () => readClearnet({
            interfaceName: STATE.interfaceName,
            readState: () => Promise.reject(new Error('state unreadable',),),
            tools: listingTools({ asked: [], },),
          },),
        },);
        expect(String(stateFailure,),).toContain('state unreadable',);
        /**
         Failure of the rule listing.
         */
        const listFailure = await thrownBy({
          run: () => readClearnet({
            interfaceName: STATE.interfaceName,
            readState: () => Promise.resolve(STATE,),
            tools: {
              ...listingTools({ asked: [], },),
              list: () => Promise.reject(new Error('ip rule show failed',),),
            },
          },),
        },);
        expect(String(listFailure,),).toContain('ip rule show failed',);
      },
    },),
  ],
},);
