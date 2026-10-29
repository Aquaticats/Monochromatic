/**
 Unit tests for the default pnpm runner against the `pnpm` on `PATH`.

 @module
 */

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  PnpmCommandError,
  runPnpm,
} from './pnpm.ts';

await describe({
  name: runPnpm.name,
  children: [
    it({
      name: 'returns stdout on success',
      fn: async () => {
        /**
         Dotted version parts pnpm printed.
         */
        const parts = (await runPnpm({ args: ['--version',], cwd: scratchRoot(), },)).trim().split('.',);
        expect(parts,).toHaveLength(3,);
        expect(parts.every(function isInteger(part,): boolean {
          return Number.isInteger(Number(part,),);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'throws PnpmCommandError with output on failure',
      fn: async () => {
        /**
         Rejection from an unknown command.
         */
        const error = await (async function attempt(): Promise<unknown> {
          try {
            await runPnpm({ args: ['deps-update-no-such-command',], cwd: scratchRoot(), },);
            return 'resolved';
          }
          catch (caught) {
            return caught;
          }
        })();
        expect(error,).toBeInstanceOf(PnpmCommandError,);
        expect((error as PnpmCommandError).message,).toContain('deps-update-no-such-command',);
      },
    },),
  ],
},);
