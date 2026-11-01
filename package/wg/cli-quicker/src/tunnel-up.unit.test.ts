import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  startApplicationExemptions,
  startExemptionsOrAbort,
} from '../dist/final/node/application-exemption.mjs';
import { bringUp, } from '../dist/final/node/tunnel-up.mjs';

/**
 What the steps of one up did,
 and what the rollback did.
 */
type Trace = {
  /**
   Events in the order they happened.
   */
  readonly events: string[];

  /**
   Commands the exemption start ran.
   */
  readonly ran: string[];
};

/**
 How the exemption program is found in a test.
 */
type Watcher = 'missing' | 'found';

/**
 Runs `bringUp` with recording steps,
 where the exemption step is the real `startExemptionsOrAbort` over a resolver that finds or misses the program.

 @param exemptMark - `ExemptMark` of the config,
 absent for none.

 @param watcher - Whether the exemption program is found.

 @param failAt - Name of a step that throws,
 absent for none.

 @returns The trace and what the up threw,
 absent when it did not throw.

 @example
 ```ts
 await runUp({ exemptMark: 100, watcher: 'missing' });
 ```
 */
async function runUp(
  {
    exemptMark,
    watcher,
    failAt,
  }: {
    readonly exemptMark?: number;
    readonly watcher: Watcher;
    readonly failAt?: string;
  },
): Promise<{ readonly trace: Trace; readonly thrown?: unknown; }> {
  /**
   Recording of the run.
   */
  const trace: Trace = { events: [], ran: [], };
  /**
   Builds a step that records its name and throws when asked to.
   */
  const step = (name: string,) => (): Promise<void> => {
    trace.events.push(name,);
    return failAt === name ? Promise.reject(new Error(`${name} failed`,),) : Promise.resolve();
  };
  try {
    await bringUp({
      ...(exemptMark === undefined ? {} : { exemptMark, }),
      steps: {
        addLink: step('addLink',),
        runPreUp: step('runPreUp',),
        applyPeerConfig: step('applyPeerConfig',),
        allowEndpoints: step('allowEndpoints',),
        addAddressesAndBringUp: step('addAddressesAndBringUp',),
        setDns: step('setDns',),
        setupRoutes: step('setupRoutes',),
        startExemptions: ({ mark, },) => {
          trace.events.push(`startExemptions ${String(mark,)}`,);
          return startExemptionsOrAbort({
            interfaceName: 'wgtest',
            mark,
            start: (call,) => startApplicationExemptions({
              ...call,
              resolveCommand: () => Promise.resolve(
                watcher === 'found'
                  ? { found: true, path: '/usr/bin/wg-quicker-exempt', } as const
                  : { found: false, searched: ['/repo/target/release/wg-quicker-exempt', '/usr/bin/wg-quicker-exempt',], } as const,
              ),
              runCommand: (command,) => {
                trace.ran.push(command.command,);
                return Promise.resolve({ stdout: '', stderr: '', },);
              },
            },),
          },);
        },
        runPostUp: step('runPostUp',),
      },
      rollback: () => {
        trace.events.push('rollback',);
        return Promise.resolve();
      },
    },);
  }
  catch (thrown) {
    return { trace, thrown, };
  }
  return { trace, };
}

await describe({
  name: bringUp.name,
  children: [
    it({
      name: 'stops the up and rolls back once when the config sets ExemptMark and the watcher program is missing, naming the paths searched and how to install',
      fn: async () => {
        const { trace, thrown, } = await runUp({ exemptMark: 100, watcher: 'missing', },);
        expect(trace.events,).toEqual([
          'addLink',
          'runPreUp',
          'applyPeerConfig',
          'allowEndpoints',
          'addAddressesAndBringUp',
          'setDns',
          'setupRoutes',
          'startExemptions 100',
          'rollback',
        ],);
        expect(trace.ran,).toEqual([],);
        expect(Error.isError(thrown,),).toBe(true,);
        expect((thrown as Error).name,).toBe('WatcherMissingError',);
        expect((thrown as Error).message,).toContain('/repo/target/release/wg-quicker-exempt, /usr/bin/wg-quicker-exempt',);
        expect((thrown as Error).message,).toContain('mise run //package/wg/cli-quicker-exempt:build',);
      },
    },),
    it({
      name: 'goes on without ExemptMark: the exemption step is skipped, the post hooks run, and nothing is rolled back',
      fn: async () => {
        const { trace, thrown, } = await runUp({ watcher: 'missing', },);
        expect(thrown,).toBeUndefined();
        expect(trace.events.at(-1),).toBe('runPostUp',);
        expect(trace.events.some((event,) => event.startsWith('startExemptions',),),).toBe(false,);
        expect(trace.events.includes('rollback',),).toBe(false,);
        expect(trace.ran,).toEqual([],);
      },
    },),
    it({
      name: 'goes on with ExemptMark when the watcher program is found, and runs it once',
      fn: async () => {
        const { trace, thrown, } = await runUp({ exemptMark: 100, watcher: 'found', },);
        expect(thrown,).toBeUndefined();
        expect(trace.events.slice(-3,),).toEqual(['setupRoutes', 'startExemptions 100', 'runPostUp',],);
        expect(trace.events.includes('rollback',),).toBe(false,);
        expect(trace.ran,).toEqual(['/usr/bin/wg-quicker-exempt',],);
      },
    },),
    it({
      name: 'rolls back once and rethrows the failure of any other step, without running the steps after it',
      fn: async () => {
        const { trace, thrown, } = await runUp({ exemptMark: 100, watcher: 'found', failAt: 'setDns', },);
        expect(trace.events,).toEqual([
          'addLink',
          'runPreUp',
          'applyPeerConfig',
          'allowEndpoints',
          'addAddressesAndBringUp',
          'setDns',
          'rollback',
        ],);
        expect((thrown as Error).message,).toBe('setDns failed',);
      },
    },),
  ],
},);
