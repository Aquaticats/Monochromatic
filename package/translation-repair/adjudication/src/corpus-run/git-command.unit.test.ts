/**
 Tests for which git the package spawns: the real binary where the system has
 it, and the one on the PATH where it does not.

 @module
 */

import fsPromises from 'node:fs/promises';
import { syncBuiltinESMExports, } from 'node:module';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { resolveGit, } from '@monochromatic-dev/translation-repair-glossary';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';

import { runBuiltInChild, } from './built-child.test-fixture.ts';

/**
 Where the package looks for the real binary.
 */
const SYSTEM_GIT = '/usr/bin/git';

await describe({
  name: resolveGit.name,
  concurrency: 1,
  children: [
    it({
      name: 'PREFERS the real binary where the system has it, the control that the path fallback is not every '
        + 'answer, asked in a process of its own since the first answer of a process is kept',
      fn: async () => {
        expect(await runBuiltInChild({
          packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-glossary',),
          body: 'process.stdout.write(await built.resolveGit());',
        },),)
          .toBe(SYSTEM_GIT,);
      },
    },),
    it({
      name: 'FALLS BACK TO THE GIT ON THE PATH where the real binary is not there, saying so once, and answers a '
        + 'second ask from the first probe',
      fn: async (ctx,) => {
        /**
         The real `access`, which every other path still reaches.
         */
        const realAccess = fsPromises.access;
        /**
         `access` as it answers on a machine without the real binary. The
         package's one probe of the path is the call this stands in for.
         */
        const withoutGit = ctx.sinon.stub(
          fsPromises,
          'access',
        ).callsFake(async function accessWithoutGit(path, mode,) {
          if (path === SYSTEM_GIT) {
            throw Object.assign(
              new Error('no such file',),
              { code: 'ENOENT', },
            );
          }
          return await realAccess(
            path,
            mode,
          );
        },);
        syncBuiltinESMExports();
        // THE ESM BINDING OF `access` FOLLOWS THE STUB ONLY WHILE IT IS SYNCED, so
        // putting it back is a step of its own that the case's end runs.
        using putBack = {
          [Symbol.dispose]: function restoreAccess(): void {
            withoutGit.restore();
            syncBuiltinESMExports();
          },
        };
        const {
          result,
          warned,
        } = await warnLinesDuring({
          run: async function twice(): Promise<readonly string[]> {
            return [
              await resolveGit(),
              await resolveGit(),
            ];
          },
        },);
        expect({
          answers: result,
          warned,
        },).toEqual({
          answers: [
            'git',
            'git',
          ],
          warned: ['[git-command] /usr/bin/git not present (Error: no such file); using git from PATH',],
        },);
      },
    },),
  ],
},);
