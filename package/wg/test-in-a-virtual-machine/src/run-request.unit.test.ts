/**
 Tests of the words a guest is asked to run.

 Every command reaches the guest through a shell, and `mvm exec` joins its own
 arguments with spaces before that shell parses them, so one misquoted word is
 a command the guest runs differently from the one intended. These cases name
 the words that used to need quoting and the shape each phase must have.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseArguments,
  quoteForPosixShell,
  renderCommandLine,
  renderPhaseCommand,
  type TestPhase,
} from '../dist/final/node/index.mjs';

/**
 A phase as one test's definition writes it, with every field under control.

 @param overrides - fields this case changes

 @returns a phase with the rest filled in
 */
function phase(overrides: Partial<TestPhase>,): TestPhase {
  return {
    id: 'probe',
    caller: 'root',
    workingDirectory: '/work',
    environment: {},
    words: ['true',],
    ...overrides,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: quoteForPosixShell.name,
      children: [
        //region Words a shell reads back unchanged

        it({
          name: 'leaves a plain word alone',
          fn: async () => {
            expect(quoteForPosixShell('apt-get',),).toBe('apt-get',);
          },
        },),

        it({
          name: 'keeps a path with slashes, dots and colons readable',
          fn: async () => {
            expect(quoteForPosixShell('/work/package/wg/cli-quicker/src/a.b.ts:1',),).toBe(
              '/work/package/wg/cli-quicker/src/a.b.ts:1',
            );
          },
        },),

        //endregion Words a shell reads back unchanged

        //region Words a shell would otherwise split

        it({
          name: 'quotes a word with a space',
          fn: async () => {
            expect(quoteForPosixShell('ubuntu ALL=(ALL) NOPASSWD: ALL',),).toBe(
              "'ubuntu ALL=(ALL) NOPASSWD: ALL'",
            );
          },
        },),

        it({
          name: 'escapes an embedded single quote',
          fn: async () => {
            expect(quoteForPosixShell("it's",),).toBe(String.raw`'it'\''s'`,);
          },
        },),

        it({
          name: 'quotes a word with a semicolon rather than letting it end the command',
          fn: async () => {
            expect(quoteForPosixShell('a;b',),).toBe("'a;b'",);
          },
        },),

        it({
          name: 'quotes an empty word so it survives as an argument',
          fn: async () => {
            expect(quoteForPosixShell('',),).toBe("''",);
          },
        },),

        //endregion Words a shell would otherwise split
      ],
    },),

    describe({
      name: renderCommandLine.name,
      children: [
        it({
          name: 'joins the words with one space each',
          fn: async () => {
            expect(renderCommandLine({ words: [
              'apt-get',
              'install',
              '--yes',
            ], },),).toBe('apt-get install --yes',);
          },
        },),
      ],
    },),

    describe({
      name: renderPhaseCommand.name,
      children: [
        //region The caller decides how the command is started

        it({
          name: 'starts an account phase through runuser in a login shell',
          fn: async () => {
            /**
             The rendered command line.
             */
            const command = renderPhaseCommand({
              logPath: '/work/logs/account.log',
              phase: phase({
                caller: 'account',
                workingDirectory: '/work/package/wg/cli-quicker',
                words: [
                  'node',
                  'src/a.test.ts',
                ],
              },),
            },);
            expect(command,).toBe(
              "runuser --user ubuntu -- sh -lc 'cd /work/package/wg/cli-quicker && node src/a.test.ts' >> /work/logs/account.log 2>&1",
            );
          },
        },),

        it({
          name: 'starts a root phase in the machine root shell without runuser',
          fn: async () => {
            /**
             The rendered command line.
             */
            const command = renderPhaseCommand({
              logPath: '/work/logs/root.log',
              phase: phase({ caller: 'root', },),
            },);
            expect(command,).toBe("sh -lc 'cd /work && true' >> /work/logs/root.log 2>&1",);
          },
        },),

        //endregion The caller decides how the command is started

        //region Variables and arguments keep their places

        it({
          name: 'sets the phase variables before the program and keeps their values quoted',
          fn: async () => {
            /**
             The rendered command line.
             */
            const command = renderPhaseCommand({
              logPath: '/work/logs/manager.log',
              phase: phase({
                environment: {
                  DBUS_SESSION_BUS_ADDRESS: 'unix:path=/run/user/1000/bus',
                  XDG_RUNTIME_DIR: '/run/user/1000',
                },
                words: ['probe',],
              },),
            },);
            expect(command,).toBe(
              "sh -lc 'cd /work && DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus XDG_RUNTIME_DIR=/run/user/1000 probe' >> /work/logs/manager.log 2>&1",
            );
          },
        },),

        it({
          name: 'quotes an argument with a space so the program sees one argument',
          fn: async () => {
            /**
             The rendered command line.
             */
            const command = renderPhaseCommand({
              logPath: '/work/logs/space.log',
              phase: phase({ words: [
                'find',
                '-name',
                'two words',
              ], },),
            },);
            expect(command,).toContain("'two words'",);
          },
        },),

        //endregion Variables and arguments keep their places
      ],
    },),

    describe({
      name: parseArguments.name,
      children: [
        //region A usable request

        it({
          name: 'reads the test to run and the file to keep its output in',
          fn: async () => {
            expect(parseArguments([
              '--test',
              'bypass',
              '--log',
              '/var/log/out.txt',
            ],),).toEqual({
              test: 'bypass',
              logPath: '/var/log/out.txt',
            },);
          },
        },),

        it({
          name: 'reads a run that keeps no file',
          fn: async () => {
            expect(parseArguments(['--test', 'route',],),).toEqual({
              test: 'route',
              logPath: undefined,
            },);
          },
        },),

        //endregion A usable request

        //region A request this program refuses

        it({
          name: 'refuses a run that names no test',
          fn: async () => {
            expect(() => {
              parseArguments([],);
            },).toThrow(/--test names the test to run/u,);
          },
        },),

        it({
          name: 'refuses an argument it does not know',
          fn: async () => {
            expect(() => {
              parseArguments([
                '--test',
                'bypass',
                '--keep',
              ],);
            },).toThrow(/unknown argument --keep/u,);
          },
        },),

        //endregion A request this program refuses
      ],
    },),
  ],
},);
