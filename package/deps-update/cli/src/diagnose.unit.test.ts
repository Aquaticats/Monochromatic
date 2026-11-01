/**
 Unit tests for the diagnosis, with a fake pnpm runner that simulates the
 loose resolution's exclude-list write, and a stubbed registry.

 @module
 */

import {
  appendFile,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  diagnoseImmaturePicks,
  firstDiagnosticLine,
  LOOSE_RESOLUTION_ARGS,
} from './diagnose.ts';
import {
  PnpmCommandError,
  type RunPnpm,
} from './pnpm.ts';

/**
 Workspace manifest before resolution.
 */
const MANIFEST = "minimumReleaseAge: 1440\nminimumReleaseAgeExclude:\n  - '@earendil-works/pi-*'\n";

/**
 Throwaway workspace root that removes itself when its `await using` scope ends.
 */
type TempRoot = AsyncDisposable & {
  /**
   Absolute root path.
   */
  readonly root: string;
};

/**
 Creates a workspace root holding only root manifests.

 @returns disposable root

 @example
 ```ts
 await using ws = await makeRoot();
 ```
 */
async function makeRoot(): Promise<TempRoot> {
  /**
   Fresh root.
   */
  const root = await mkdtemp(join(scratchRoot(), 'deps-update-diagnose-',),);
  await Promise.all([
    writeFile(join(root, 'package.json',), '{}',),
    writeFile(join(root, 'pnpm-workspace.yaml',), MANIFEST,),
  ],);
  return {
    root,
    [Symbol.asyncDispose]: async function removeRoot(): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Builds a fake pnpm that answers the diagnosis' commands.

 @param root - workspace root reported by `pnpm list`

 @param appended - exclude lines the loose resolution appends

 @param failLoose - whether the loose resolution exits nonzero after appending

 @returns runner plus the argument lists it received

 @example
 ```ts
 const { runPnpm } = fakePnpm({ root, appended: [], failLoose: false });
 ```
 */
function fakePnpm({
  root,
  appended,
  failLoose,
}: {
  readonly root: string;
  readonly appended: readonly string[];
  readonly failLoose: boolean;
},): {
  readonly runPnpm: RunPnpm;
  readonly calls: readonly (readonly string[])[];
} {
  /**
   Received argument lists.
   */
  const calls: (readonly string[])[] = [];
  return {
    calls,
    runPnpm: async function fake({
      args,
      cwd,
    },): Promise<string> {
      calls.push(args,);
      if (args[0] === 'list')
        return JSON.stringify([{ path: root, },],);
      if (args[0] === 'config')
        return args[2] === 'registry' ? 'https://registry.example/\n' : 'undefined\n';
      if (args[0] === 'why') {
        return JSON.stringify([{
          name: args[1],
          version: '0.87.1',
          dependents: [
            { name: '@earendil-works/pi-coding-agent', version: '0.87.1', },
            { name: '@earendil-works/pi-coding-agent', version: '0.87.1', },
            { name: 'workspace-app', },
          ],
        },],);
      }
      if (args.join(' ',) === LOOSE_RESOLUTION_ARGS.join(' ',)) {
        await appendFile(
          join(cwd, 'pnpm-workspace.yaml',),
          appended.map(function toLine(entry,): string {
            return `  - '${entry}'\n`;
          },).join('',),
        );
        if (failLoose) {
          throw new PnpmCommandError({
            args,
            cwd,
            output: 'progress\n[ERR_PNPM_PEER_DEP_ISSUES] Unmet peer dependencies\n',
            cause: new Error('exit 1',),
          },);
        }
        return '';
      }
      throw new Error(`unexpected pnpm ${args.join(' ',)}`,);
    },
  };
}

/**
 Registry stub serving chord's publish time.
 */
async function fetchImpl(): Promise<Response> {
  return Response.json({ time: { '0.87.1': '2026-09-22T19:38:00.606Z', }, },);
}

await describe({
  name: '',
  children: [
    describe({
      name: diagnoseImmaturePicks.name,
      children: [
        it({
          name: 'reports appended picks with time and deduplicated dependents',
          fn: async () => {
            await using ws = await makeRoot();
            /**
             Fake pnpm appending one pick.
             */
            const pnpm = fakePnpm({ root: ws.root, appended: ['@earendil-works/chord@0.87.1',], failLoose: false, },);
            /**
             Diagnosis result.
             */
            const diagnosis = await diagnoseImmaturePicks({ root: ws.root, runPnpm: pnpm.runPnpm, fetchImpl, },);
            expect(diagnosis.minutes,).toBe(1_440,);
            expect(diagnosis.picks,).toEqual([{
              name: '@earendil-works/chord',
              version: '0.87.1',
              publishedAt: new Date('2026-09-22T19:38:00.606Z',),
              dependents: ['@earendil-works/pi-coding-agent@0.87.1', 'workspace-app',],
            },],);
            expect('followUp' in diagnosis,).toBe(false,);
            expect(pnpm.calls,).toContainEqual(['config', 'get', '@earendil-works:registry',],);
          },
        },),
        it({
          name: 'keeps picks and records a later loose-resolution failure',
          fn: async () => {
            await using ws = await makeRoot();
            /**
             Diagnosis result.
             */
            const diagnosis = await diagnoseImmaturePicks({
              root: ws.root,
              runPnpm: fakePnpm({ root: ws.root, appended: ['@earendil-works/chord@0.87.1',], failLoose: true, },).runPnpm,
              fetchImpl,
            },);
            expect(diagnosis.picks,).toHaveLength(1,);
            expect(diagnosis.followUp,).toBe('[ERR_PNPM_PEER_DEP_ISSUES] Unmet peer dependencies',);
          },
        },),
        it({
          name: 'rethrows a loose-resolution failure that recorded no picks',
          fn: async () => {
            await using ws = await makeRoot();
            /**
             Rejection from the diagnosis.
             */
            const error = await (async function attempt(): Promise<unknown> {
              try {
                await diagnoseImmaturePicks({
                  root: ws.root,
                  runPnpm: fakePnpm({ root: ws.root, appended: [], failLoose: true, },).runPnpm,
                  fetchImpl,
                },);
                return 'resolved';
              }
              catch (caught) {
                return caught;
              }
            })();
            expect(error,).toBeInstanceOf(PnpmCommandError,);
          },
        },),
        it({
          name: 'returns no picks when nothing was appended',
          fn: async () => {
            await using ws = await makeRoot();
            expect((await diagnoseImmaturePicks({
              root: ws.root,
              runPnpm: fakePnpm({ root: ws.root, appended: [], failLoose: false, },).runPnpm,
              fetchImpl,
            },)).picks,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: firstDiagnosticLine.name,
      children: [
        it({
          name: 'prefers the first ERR_PNPM line',
          fn: async () => {
            expect(firstDiagnosticLine('a\n  [ERR_PNPM_X] one\n[ERR_PNPM_Y] two\n',),).toBe('[ERR_PNPM_X] one',);
          },
        },),
        it({
          name: 'falls back to the last nonblank line',
          fn: async () => {
            expect(firstDiagnosticLine('a\nlast\n\n',),).toBe('last',);
          },
        },),
        it({
          name: 'describes empty output',
          fn: async () => {
            expect(firstDiagnosticLine('',),).toBe('pnpm printed no output',);
          },
        },),
      ],
    },),
  ],
},);
