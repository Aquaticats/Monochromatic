import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  formatMark,
  RECOMMENDED_EXEMPT_MARK,
} from '../dist/final/node/exempt-mark-reservation.mjs';
import type { Clearnet, } from '../dist/final/node/tunnel-clearnet.mjs';
import {
  BLOCK_EXCEPTIONS,
  blockTraffic,
  installTrafficBlock,
  removeTrafficBlock,
  trafficBlockExists,
  trafficBlockScript,
  trafficBlockState,
} from '../dist/final/node/tunnel-traffic-block.mjs';

/**
 Runner of commands that may fail,
 as the removal and the state check take it.
 */
type AllowFailureRunner = NonNullable<Parameters<typeof removeTrafficBlock>[0]['runCommand']>;

/**
 Runner of commands that must succeed,
 as the install takes it.
 */
type StrictRunner = NonNullable<Parameters<typeof installTrafficBlock>[0]['runCommand']>;

/**
 What one scripted `nft` call answers.
 */
type Answer = {
  /**
   Exit code of the call.
   */
  readonly exitCode: number;

  /**
   Standard error of the call.
   */
  readonly stderr: string;

  /**
   Standard output of the call.
   */
  readonly stdout?: string;
};

/**
 Builds a runner that answers from a script and records the calls it received.

 @param answers - Answers in call order.

 @returns The runner and the recorded invocations,
 each as the arguments after `nft`.

 @example
 ```ts
 scripted({ answers: [{ exitCode: 1, stderr: 'Error: No such file or directory' }] });
 ```
 */
function scripted({ answers, }: { readonly answers: readonly Answer[]; },): { readonly runner: AllowFailureRunner; readonly calls: string[][]; } {
  /**
   Invocations in order.
   */
  const calls: string[][] = [];
  /**
   Position in the script.
   */
  const cursor = { next: 0, };
  /**
   The runner under test.

   @param call - The invocation.

   @returns The scripted answer.
   */
  function runner(call: { readonly command: string; readonly args: readonly string[]; },): ReturnType<AllowFailureRunner> {
    calls.push([...call.args,],);
    /**
     Answer for this call.
     */
    const answer = answers[cursor.next] ?? { exitCode: 0, stderr: '', };
    cursor.next += 1;
    return Promise.resolve({ stdout: '', ...answer, },);
  }
  return { runner, calls, };
}

/**
 Answer of `nft --json list tables` with the given tables in it,
 each as a family and a name.

 @param tables - Tables of the ruleset.

 @returns The answer.

 @example
 ```ts
 listing({ tables: [{ family: 'inet', name: 'x' }] });
 ```
 */
function listing({ tables, }: { readonly tables: readonly { readonly family: string; readonly name: string; }[]; },): Answer {
  return {
    exitCode: 0,
    stderr: '',
    stdout: JSON.stringify({
      nftables: [
        { metainfo: { version: '1.1.6', json_schema_version: 1, }, },
        ...tables.map((table,) => ({ table: { ...table, handle: 1, }, }),),
      ],
    },),
  };
}

/**
 Answer of `nft --json list tables` for a ruleset with the block in it.
 */
const LISTED: Answer = listing({ tables: [{ family: 'inet', name: 'wg-quicker-wgtest-blocked', },], },);

/**
 Answer of `nft --json list tables` for a ruleset without the block,
 as an exit status of zero and a listing show it.
 */
const NO_SUCH_TABLE: Answer = listing({ tables: [{ family: 'inet', name: 'other', },], },);

/**
 Answer of `nft` when the caller may not read or change the rules.
 */
const NOT_PERMITTED: Answer = { exitCode: 1, stderr: 'Error: Operation not permitted\n', };

/**
 Makes a scratch directory in the test scratch root,
 removed when the scope exits,
 where the real one may belong to another user.

 @returns A disposable directory.

 @example
 ```ts
 await using dir = await scratchDirectory();
 ```
 */
async function scratchDirectory(): ReturnType<NonNullable<Parameters<typeof installTrafficBlock>[0]['makeDirectory']>> {
  /**
   Path of the new directory.
   */
  const path = await mkdtemp(join(scratchRoot(), 'wg-quicker-block-',),);
  return {
    path,
    [Symbol.asyncDispose]: async (): Promise<void> => {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: installTrafficBlock.name,
  children: [
    it({
      name: 'installs and replaces the block in one nft call that creates, empties and fills the table, with no delete',
      fn: async () => {
        /**
         Invocations and the script each one read.
         */
        const seen: { readonly args: readonly string[]; readonly script: string; }[] = [];
        /**
         Runner that reads the script file while the call runs,
         because the file is removed when the install ends.

         @param call - The invocation.

         @returns An empty result.
         */
        const runner: StrictRunner = async (call,) => {
          seen.push({ args: [...call.args,], script: await readFile(call.args[1] ?? '', 'utf8',), },);
          return { stdout: '', stderr: '', };
        };
        await installTrafficBlock({ interfaceName: 'wgtest', runCommand: runner, makeDirectory: scratchDirectory, },);
        expect(seen.length,).toBe(1,);
        expect(seen[0]?.args[0],).toBe('-f',);
        /**
         Lines of the script.
         */
        const lines = (seen[0]?.script ?? '').split('\n',);
        expect(lines[0],).toBe('add table inet wg-quicker-wgtest-blocked',);
        expect(lines[1],).toBe('flush table inet wg-quicker-wgtest-blocked',);
        expect(lines.some((line,) => line.startsWith('delete',) || line.includes('delete table',),),).toBe(false,);
        expect(seen[0]?.script,).toBe(trafficBlockScript({ interfaceName: 'wgtest', },),);
        /**
         The same install with exceptions.
         */
        seen.length = 0;
        await installTrafficBlock({ interfaceName: 'wgtest', clearnet: { mark: RECOMMENDED_EXEMPT_MARK, hosts: [], interfaces: [], }, runCommand: runner, makeDirectory: scratchDirectory, },);
        expect(seen[0]?.script,).toBe(trafficBlockScript({ interfaceName: 'wgtest', clearnet: { mark: RECOMMENDED_EXEMPT_MARK, hosts: [], interfaces: [], }, },),);
        expect(seen[0]?.script,).toContain('counter accept',);
      },
    },),
  ],
},);

await describe({
  name: trafficBlockState.name,
  children: [
    it({
      name: 'tells a table that is listed, one that is not, and a listing that cannot be read apart, and only a table known to be there counts as existing',
      fn: async () => {
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [LISTED,], },).runner, },),).toBe('present',);
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [NO_SUCH_TABLE,], },).runner, },),).toBe('absent',);
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [NOT_PERMITTED,], },).runner, },),).toBe('unknown',);
        expect(await trafficBlockExists({ interfaceName: 'wgtest', runCommand: scripted({ answers: [LISTED,], },).runner, },),).toBe(true,);
        expect(await trafficBlockExists({ interfaceName: 'wgtest', runCommand: scripted({ answers: [NO_SUCH_TABLE,], },).runner, },),).toBe(false,);
        expect(await trafficBlockExists({ interfaceName: 'wgtest', runCommand: scripted({ answers: [NOT_PERMITTED,], },).runner, },),).toBe(false,);
      },
    },),
    it({
      name: 'asks for the structured listing of the tables, and matches the family as well as the name',
      fn: async () => {
        /**
         The calls made.
         */
        const { runner, calls, } = scripted({ answers: [listing({ tables: [{ family: 'ip', name: 'wg-quicker-wgtest-blocked', },], },),], },);
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: runner, },),).toBe('absent',);
        expect(calls,).toEqual([['--json', 'list', 'tables',],],);
      },
    },),
    it({
      name: 'does not take the words of an error for the answer: a failure that says no such file is unknown, and so is output that is not the listing',
      fn: async () => {
        /**
         The English words that `nft` printed for an absent table before the listing was read instead.
         */
        const english: Answer = { exitCode: 1, stderr: 'Error: No such file or directory\nlist table inet x\n', };
        /**
         The same failure in another language.
         */
        const german: Answer = { exitCode: 1, stderr: 'Fehler: Datei oder Verzeichnis nicht gefunden\n', };
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [english,], },).runner, },),).toBe('unknown',);
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [german,], },).runner, },),).toBe('unknown',);
        // A listing without the table, printed by a command that failed, does not show the table gone either.
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [{ ...NO_SUCH_TABLE, exitCode: 1, },], },).runner, },),).toBe('unknown',);
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [{ ...listing({ tables: [], },), exitCode: 1, },], },).runner, },),).toBe('unknown',);
        for (const stdout of ['', 'not json', '[]', '{}', '{"nftables":"x"}', 'null',]) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own runner.
          expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [{ exitCode: 0, stderr: '', stdout, },], },).runner, },),).toBe('unknown',);
        }
      },
    },),
  ],
},);

await describe({
  name: removeTrafficBlock.name,
  children: [
    it({
      name: 'resolves when the table was never there or the delete worked and the table is gone',
      fn: async () => {
        for (const answers of [[NO_SUCH_TABLE, NO_SUCH_TABLE,], [LISTED, NO_SUCH_TABLE,],]) {
          const { runner, calls, } = scripted({ answers, },);
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own recording.
          await removeTrafficBlock({ interfaceName: 'wgtest', runCommand: runner, },);
          expect(calls[0],).toEqual(['delete', 'table', 'inet', 'wg-quicker-wgtest-blocked',],);
          expect(calls[1],).toEqual(['--json', 'list', 'tables',],);
        }
      },
    },),
    it({
      name: 'throws a typed error naming the table and the command that removes it when the delete fails and the table is still there',
      fn: async () => {
        /**
         What the removal threw.
         */
        const thrown = await thrownBy({
          run: () => removeTrafficBlock({
            interfaceName: 'wgtest',
            runCommand: scripted({ answers: [NOT_PERMITTED, LISTED,], },).runner,
          },),
        },);
        expect((thrown as Error).name,).toBe('TrafficBlockError',);
        /**
         Its message.
         */
        const message = String(thrown,);
        expect(message,).toContain('the traffic block (nft table inet wg-quicker-wgtest-blocked) is still in place',);
        expect(message,).toContain('nft delete table exited 1: Error: Operation not permitted',);
        expect(message,).toContain('sudo nft delete table inet wg-quicker-wgtest-blocked',);
      },
    },),
    it({
      name: 'throws a typed error, and does not claim success, when it cannot tell whether the table is gone',
      fn: async () => {
        /**
         What the removal threw.
         */
        const thrown = await thrownBy({
          run: () => removeTrafficBlock({
            interfaceName: 'wgtest',
            runCommand: scripted({ answers: [NOT_PERMITTED, NOT_PERMITTED,], },).runner,
          },),
        },);
        expect((thrown as Error).name,).toBe('TrafficBlockError',);
        expect(String(thrown,),).toContain('could not tell whether the traffic block (nft table inet wg-quicker-wgtest-blocked) was removed',);
        expect(String(thrown,),).toContain('sudo nft list table inet wg-quicker-wgtest-blocked',);
        expect(String(thrown,),).toContain('sudo nft delete table inet wg-quicker-wgtest-blocked',);
      },
    },),
  ],
},);

/**
 The block of an interface that has an application bypass:
 the exempt mark and destinations of both families.
 */
const CLEARNET: Clearnet = {
  mark: RECOMMENDED_EXEMPT_MARK,
  hosts: [
    { family: '-4', address: '203.0.113.7', },
    { family: '-4', address: '203.0.113.8', },
    { family: '-6', address: '2001:db8::7', },
  ],
  interfaces: [
    { family: '-4', name: 'eth0', },
    { family: '-4', name: 'wlp9s0', },
    { family: '-6', name: 'eth0', },
  ],
};

/**
 Reads what the thrown value says.

 @param run - Call expected to throw.

 @returns The error,
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
  name: trafficBlockScript.name,
  children: [
    it({
      name: 'lets the exempt mark and the update hosts through ahead of the drop, and keeps loopback allowed and forwarding dropped',
      fn: async () => {
        /**
         Lines of the script.
         */
        const lines = trafficBlockScript({ interfaceName: 'wgtest', clearnet: CLEARNET, },).split('\n',);
        /**
         Table that holds the block.
         */
        const table = 'wg-quicker-wgtest-blocked';
        /**
         Where each rule stands.
         */
        const markAccept = lines.indexOf(`add rule inet ${table} output meta mark ${formatMark({ mark: RECOMMENDED_EXEMPT_MARK, },)} counter accept`,);
        const v4Accept = lines.indexOf(`add rule inet ${table} output ip daddr { 203.0.113.7, 203.0.113.8 } counter accept`,);
        const v6Accept = lines.indexOf(`add rule inet ${table} output ip6 daddr { 2001:db8::7 } counter accept`,);
        const outputDrop = lines.indexOf(`add rule inet ${table} output oifname != "lo" counter drop`,);
        const outputChain = lines.indexOf(`add chain inet ${table} output { type filter hook output priority -400; policy accept; }`,);
        expect(outputChain,).toBeGreaterThan(-1,);
        expect(markAccept,).toBeGreaterThan(outputChain,);
        expect(v4Accept,).toBeGreaterThan(markAccept,);
        expect(v6Accept,).toBeGreaterThan(v4Accept,);
        expect(outputDrop,).toBeGreaterThan(v6Accept,);
        /**
         Where the forward chain and its rules stand.
         */
        const forwardChain = lines.indexOf(`add chain inet ${table} forward { type filter hook forward priority -400; policy accept; }`,);
        const forwardV4 = lines.indexOf(`add rule inet ${table} forward ip daddr { 203.0.113.7, 203.0.113.8 } counter accept`,);
        const forwardV6 = lines.indexOf(`add rule inet ${table} forward ip6 daddr { 2001:db8::7 } counter accept`,);
        const replyV4 = lines.indexOf(`add rule inet ${table} forward iifname { "eth0", "wlp9s0" } ct direction reply ip saddr { 203.0.113.7, 203.0.113.8 } counter accept`,);
        const replyV6 = lines.indexOf(`add rule inet ${table} forward iifname { "eth0" } ct direction reply ip6 saddr { 2001:db8::7 } counter accept`,);
        const forwardDrop = lines.indexOf(`add rule inet ${table} forward counter drop`,);
        expect(forwardChain,).toBeGreaterThan(outputDrop,);
        expect(forwardV4,).toBeGreaterThan(forwardChain,);
        expect(forwardV6,).toBeGreaterThan(forwardV4,);
        // The replies of the update hosts to the guests come after the requests and ahead of the drop.
        expect(replyV4,).toBeGreaterThan(forwardV6,);
        expect(replyV6,).toBeGreaterThan(replyV4,);
        expect(forwardDrop,).toBeGreaterThan(replyV6,);
        // The forward chain accepts only the update hosts, to and from: the mark is a mark of this machine's sockets, so it has no rule there,
        // and no rule matches the state of a connection, which would pass the guests' established flows that were bound for the tunnel.
        expect(lines.filter((line,) => line.startsWith(`add rule inet ${table} forward `,) && line.endsWith('counter accept',),),).toEqual([
          `add rule inet ${table} forward ip daddr { 203.0.113.7, 203.0.113.8 } counter accept`,
          `add rule inet ${table} forward ip6 daddr { 2001:db8::7 } counter accept`,
          `add rule inet ${table} forward iifname { "eth0", "wlp9s0" } ct direction reply ip saddr { 203.0.113.7, 203.0.113.8 } counter accept`,
          `add rule inet ${table} forward iifname { "eth0" } ct direction reply ip6 saddr { 2001:db8::7 } counter accept`,
        ],);
        // Two forged packets can make a "reply" in the tracker, so every source rule also names the interfaces a real reply arrives on.
        expect(lines.filter((line,) => line.includes(' saddr ',) && (!line.includes(' iifname { "',)),),).toEqual([],);
        // A source address alone would pass a forged source: every source rule is restricted to the reply direction of a tracked flow,
        // and no rule matches the state of a connection.
        expect(lines.filter((line,) => line.includes(' saddr ',) && (!line.includes(' ct direction reply ',)),),).toEqual([],);
        expect(lines.filter((line,) => line.includes(' ct ',),).length,).toBe(2,);
        expect(lines.some((line,) => line.includes('ct state',) || line.includes('established',) || line.includes('related',),),).toBe(false,);
        // The output chain gets no source rule: the replies to this machine's own packets arrive through the input hook.
        expect(lines.filter((line,) => line.includes(' output ',) && line.includes(' saddr ',),),).toEqual([],);
        // Loopback is the one interface the drop leaves alone.
        expect(lines.some((line,) => line.includes('oifname != "lo"',) && line.endsWith('counter drop',),),).toBe(true,);
      },
    },),
    it({
      name: 'accepts nothing in the forward chain when there is a mark but no update host, and never puts the mark in it',
      fn: async () => {
        /**
         Lines of the script.
         */
        const lines = trafficBlockScript({ interfaceName: 'wgtest', clearnet: { mark: RECOMMENDED_EXEMPT_MARK, hosts: [], interfaces: [], }, },).split('\n',);
        expect(lines.filter((line,) => line.includes(' forward ',) && line.includes('accept',) && (!line.includes('policy accept',)),),).toEqual([],);
        expect(lines.filter((line,) => line.includes(' forward ',) && line.includes('meta mark',),),).toEqual([],);
        expect(lines.filter((line,) => line.includes(' saddr ',),),).toEqual([],);
        expect(lines.includes('add rule inet wg-quicker-wgtest-blocked forward counter drop',),).toBe(true,);
        expect(lines.some((line,) => line.includes(' output ',) && line.includes('meta mark',),),).toBe(true,);
      },
    },),
    it({
      name: 'has no accept rule at all when the interface has no application bypass',
      fn: async () => {
        /**
         Rules that accept,
         not counting the policy of the chains.
         */
        const accepts = trafficBlockScript({ interfaceName: 'wgtest', },).split('\n',).filter((line,) => line.startsWith('add rule',) && line.endsWith('accept',),);
        expect(accepts,).toEqual([],);
      },
    },),
    it({
      name: 'writes one address once, and nothing for a family without addresses',
      fn: async () => {
        /**
         Script for repeated and single-family destinations.
         */
        const script = trafficBlockScript({
          interfaceName: 'wgtest',
          clearnet: { hosts: [{ family: '-4', address: '203.0.113.7', }, { family: '-4', address: '203.0.113.7', },], interfaces: [], },
        },);
        expect(script,).toContain('ip daddr { 203.0.113.7 } counter accept',);
        expect(script,).not.toContain('ip6 daddr',);
      },
    },),
    it({
      name: 'gives each family the reply rule of its own physical default interfaces, and none to a family without one',
      fn: async () => {
        /**
         Source rules of a script for the given interfaces.
         */
        const sourceRules = (interfaces: Clearnet['interfaces'],): readonly string[] => trafficBlockScript({
          interfaceName: 'wgtest',
          clearnet: { ...CLEARNET, interfaces, },
        },).split('\n',).filter((line,) => line.includes(' saddr ',),);
        expect(sourceRules([{ family: '-4', name: 'eth0', },],),).toEqual([
          'add rule inet wg-quicker-wgtest-blocked forward iifname { "eth0" } ct direction reply ip saddr { 203.0.113.7, 203.0.113.8 } counter accept',
        ],);
        expect(sourceRules([{ family: '-6', name: 'wlan0', },],),).toEqual([
          'add rule inet wg-quicker-wgtest-blocked forward iifname { "wlan0" } ct direction reply ip6 saddr { 2001:db8::7 } counter accept',
        ],);
        // Without any physical default interface no forwarded packet counts as a reply, and the destination rules stay.
        expect(sourceRules([],),).toEqual([],);
        expect(trafficBlockScript({ interfaceName: 'wgtest', clearnet: { ...CLEARNET, interfaces: [], }, },),).toContain(' forward ip daddr { 203.0.113.7, 203.0.113.8 } counter accept',);
        // The same interface twice is written once.
        expect(sourceRules([{ family: '-4', name: 'eth0', }, { family: '-4', name: 'eth0', },],),).toEqual([
          'add rule inet wg-quicker-wgtest-blocked forward iifname { "eth0" } ct direction reply ip saddr { 203.0.113.7, 203.0.113.8 } counter accept',
        ],);
      },
    },),
    it({
      name: 'refuses an interface name that is not an interface name, so nothing but a name reaches the script',
      fn: async () => {
        for (const name of ['eth0" } ; flush ruleset ; { "', 'eth0\nflush ruleset', 'eth 0', 'eth0;id', '-x', '', 'a'.repeat(16,), '$(id)', 'eth0"', 'eth*',]) {
          for (const family of ['-4', '-6',] as const) {
            // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own error.
            const thrown = await thrownBy({ run: () => trafficBlockScript({ interfaceName: 'wgtest', clearnet: { ...CLEARNET, interfaces: [{ family, name, },], }, },), },);
            expect((thrown as Error).name,).toBe('TrafficBlockError',);
          }
        }
      },
    },),
    it({
      name: 'accepts the smallest and the largest packet mark and refuses what is not a 32-bit mark',
      fn: async () => {
        /**
         Largest 32-bit value.
         */
        const largest = (2 ** 32) - 1;
        expect(trafficBlockScript({ interfaceName: 'wgtest', clearnet: { mark: 0, hosts: [], interfaces: [], }, },),).toContain(`meta mark ${formatMark({ mark: 0, },)} counter accept`,);
        expect(trafficBlockScript({ interfaceName: 'wgtest', clearnet: { mark: largest, hosts: [], interfaces: [], }, },),).toContain(`meta mark ${formatMark({ mark: largest, },)} counter accept`,);
        for (const mark of [-1, largest + 1, 1.5, Number.NaN,]) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own error.
          const thrown = await thrownBy({ run: () => trafficBlockScript({ interfaceName: 'wgtest', clearnet: { mark, hosts: [], interfaces: [], }, },), },);
          expect((thrown as Error).name,).toBe('TrafficBlockError',);
        }
      },
    },),
    it({
      name: 'refuses an address that is not an address of its family, so nothing but an address reaches the script',
      fn: async () => {
        for (const address of ['203.0.113.7 } ; flush ruleset ; { ', '203.0.113.7\nflush ruleset', 'example.test', '2001:db8::7', '203.0.113.7/24', '',]) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own error.
          const thrown = await thrownBy({ run: () => trafficBlockScript({ interfaceName: 'wgtest', clearnet: { hosts: [{ family: '-4', address, },], interfaces: [], }, },), },);
          expect((thrown as Error).name,).toBe('TrafficBlockError',);
        }
        /**
         A v4 address under the v6 family.
         */
        const crossed = await thrownBy({ run: () => trafficBlockScript({ interfaceName: 'wgtest', clearnet: { hosts: [{ family: '-6', address: '203.0.113.7', },], interfaces: [], }, },), },);
        expect((crossed as Error).name,).toBe('TrafficBlockError',);
      },
    },),
  ],
},);

await describe({
  name: blockTraffic.name,
  children: [
    it({
      name: 'installs the block with what the routing sends around the tunnel when the ownership state is there',
      fn: async () => {
        /**
         What was installed.
         */
        const installed: { readonly interfaceName: string; readonly clearnet?: Clearnet; }[] = [];
        /**
         Whether the block was installed.
         */
        const done = await blockTraffic({
          interfaceName: 'wgtest',
          readExceptions: () => Promise.resolve({ bypass: true, clearnet: CLEARNET, },),
          blockExists: () => Promise.resolve(true,),
          install: (call,) => {
            installed.push(call,);
            return Promise.resolve();
          },
        },);
        expect(done,).toBe(true,);
        expect(installed,).toEqual([{ interfaceName: 'wgtest', clearnet: CLEARNET, },],);
      },
    },),
    it({
      name: 'installs the block with no exceptions when there is no ownership state and no block',
      fn: async () => {
        /**
         What was installed.
         */
        const installed: { readonly interfaceName: string; readonly clearnet?: Clearnet; }[] = [];
        expect(await blockTraffic({
          interfaceName: 'wgtest',
          readExceptions: () => Promise.resolve({ bypass: false, },),
          blockExists: () => Promise.resolve(false,),
          install: (call,) => {
            installed.push(call,);
            return Promise.resolve();
          },
        },),).toBe(true,);
        expect(installed,).toEqual([{ interfaceName: 'wgtest', },],);
      },
    },),
    it({
      name: 'keeps a block that stands, with its exceptions, when an earlier down already removed the ownership state',
      fn: async () => {
        /**
         Installs attempted.
         */
        const installed: unknown[] = [];
        expect(await blockTraffic({
          interfaceName: 'wgtest',
          readExceptions: () => Promise.resolve({ bypass: false, },),
          blockExists: () => Promise.resolve(true,),
          install: (call,) => {
            installed.push(call,);
            return Promise.resolve();
          },
        },),).toBe(false,);
        expect(installed,).toEqual([],);
      },
    },),
    it({
      name: 'fails, and installs nothing, when what to let through cannot be read',
      fn: async () => {
        /**
         Installs attempted.
         */
        const installed: unknown[] = [];
        /**
         What was thrown.
         */
        const thrown = await thrownBy({
          run: () => blockTraffic({
            interfaceName: 'wgtest',
            readExceptions: () => Promise.reject(new Error('state unreadable',),),
            blockExists: () => Promise.resolve(false,),
            install: (call,) => {
              installed.push(call,);
              return Promise.resolve();
            },
          },),
        },);
        expect(String(thrown,),).toContain('state unreadable',);
        // The error is the typed one that the teardown turns into by-hand steps, and it is recognized by its marker.
        expect((thrown as Error).name,).toBe('ClearnetUnreadableError',);
        expect((thrown as { readonly clearnetUnreadable?: boolean; }).clearnetUnreadable,).toBe(true,);
        expect(((thrown as Error).cause as Error).message,).toBe('state unreadable',);
        expect(installed,).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: 'BLOCK_EXCEPTIONS',
  children: [
    it({
      name: 'says what the block was installed to let through, and that this depends on the application bypass',
      fn: async () => {
        expect(BLOCK_EXCEPTIONS,).toContain('exempt mark',);
        expect(BLOCK_EXCEPTIONS,).toContain('update hosts',);
        expect(BLOCK_EXCEPTIONS,).toContain('forwarded traffic',);
        expect(BLOCK_EXCEPTIONS,).toContain('when the interface has an application bypass',);
      },
    },),
  ],
},);
