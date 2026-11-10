/**
 Tests for the install steps: the autostart entry and the KWin script link.

 The autostart `Exec` value crosses two grammars, so the cases read it back with
 a desktop-entry parser, check the shell script with `sh -n`, and then run the
 parsed command against a stand-in daemon whose path holds a space and a single
 quote, with a stand-in `systemd-cat` recording the journal. Every folder is
 disposable and every home directory is one the test created.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  readlink,
  rm,
  writeFile,
} from 'node:fs/promises';
import { delimiter, join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  buildAutostartDesktopEntry,
  flushLogs,
  installKeyHelper,
  InstallError,
  reconcileKwinScriptLink,
  runCommand,
  writeAutostartEntry,
} from '../dist/final/node/testing.mjs';

/**
 Prefix every disposable folder starts with.
 */
const FOLDER_PREFIX = 'key-helper-install-';

/**
 Directory name of a checkout whose path holds a space and a single quote.
 */
const AWKWARD_CHECKOUT_DIRECTORY = "c heck'out";

/**
 Filename of the daemon binary inside the checkout.
 */
const DAEMON_BINARY_NAME = 'key-helper-service';

/**
 Directory name of the KWin script package inside the checkout.
 */
const KWIN_SCRIPT_DIRECTORY = 'kwin-script';

/**
 Filename of the autostart entry the install writes.
 */
const AUTOSTART_FILE_NAME = 'key-helper-service.desktop';

/**
 Path of the autostart directory, relative to the home directory.
 */
const AUTOSTART_RELATIVE_DIRECTORY = '.config/autostart';

/**
 Path of the KWin scripts directory, relative to the home directory.
 */
const KWIN_SCRIPTS_RELATIVE_DIRECTORY = '.local/share/kwin/scripts';

/**
 Name of the symlink the KWin script is installed as.
 */
const KWIN_SCRIPT_LINK_NAME = 'key-helper-kwin';

/**
 Prefix of the entry's `Exec` key.
 */
const EXEC_PREFIX = 'Exec=';

/**
 Shell used by the entry's `Exec` line.
 */
const SHELL_PATH = '/bin/sh';

/**
 `-c` option of the entry's shell.
 */
const SHELL_COMMAND_OPTION = '-c';

/**
 Environment variable naming the file the stand-in daemon writes its marker to.
 */
const STAND_IN_MARKER_VARIABLE = 'KEY_HELPER_TEST_DAEMON_MARKER';

/**
 Environment variable naming the file the stand-in `systemd-cat` records into.
 */
const JOURNAL_VARIABLE = 'KEY_HELPER_TEST_JOURNAL';

/**
 Execution mode of a stand-in program.
 */
const EXECUTABLE_FILE_MODE = 0o755;

/**
 Marker the stand-in daemon writes when it runs.
 */
const DAEMON_MARKER_TEXT = 'daemon ran';

/**
 Exit status the stand-in `systemd-cat` reports.
 */
const SYSTEMD_CAT_EXIT_CODE = 0;

/**
 A disposable folder that removes itself when disposed.
 
 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
type ScratchFolder = {
  /**
   Absolute path of the folder.
   */
  readonly path: string;
  /**
   Remove the folder and everything inside it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Create an empty disposable folder.
 
 @returns The folder and its own removal
 
 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<ScratchFolder> {
  /**
   Absolute path of the new folder.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    FOLDER_PREFIX,
  ));
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 Split an `Exec` value into arguments the way the Desktop Entry Specification
 says a launcher does.

 Arguments are separated by whitespace; a double quote starts a quoted argument
 in which a backslash escapes the next character, and quoting may start inside
 an argument.
 
 @param entryText - Whole desktop file's text
 
 @returns The entry's argument vector
 
 @example
 ```ts
 execArguments({ entryText }); // ['/bin/sh', '-c', 'exec ...']
 ```
 */
function execArguments({ entryText }: { readonly entryText: string; }): readonly string[] {
  /**
   The entry's `Exec` line.
   */
  const execLine = entryText
    .split('\n')
    .find(function isExec(line: string): boolean {
      return line.startsWith(EXEC_PREFIX);
    });
  if (execLine === undefined) {
    throw new Error(`the entry has no ${EXEC_PREFIX} line`,);
  }
  /**
   The `Exec` value without its key.
   */
  const value = execLine.slice(EXEC_PREFIX.length);
  /**
   Arguments built so far.
   */
  const args: string[] = [];
  /**
   Argument under construction, or `undefined` between arguments.
   */
  let current = '';
  /**
   Whether the next character is escaped by a backslash.
   */
  let escaped = false;
  /**
   Whether the parser is inside a quoted argument.
   */
  let quoted = false;
  for (const character of value) {
    if (escaped) {
      current += character;
      escaped = false;
      continue;
    }
    if ((character === '\\') && quoted) {
      escaped = true;
      continue;
    }
    if (character === '"') {
      quoted = !quoted;
      continue;
    }
    if ((character === ' ') && (!quoted)) {
      if (current !== '') {
        args.push(current);
        current = '';
      }
      continue;
    }
    current += character;
  }
  if (current !== '') {
    args.push(current);
  }
  return args;
}

/**
 Environment variables the execution case needs, restored when it ends.
 
 @param path - `PATH` the case runs with
 @param daemonMarker - File the stand-in daemon writes its marker into
 @param journal - File the stand-in `systemd-cat` records into
 
 @returns A disposer that puts every variable back
 
 @example
 ```ts
 using _environment = overrideEnvironment({ path, daemonMarker, journal });
 ```
 */
function overrideEnvironment({
  path,
  daemonMarker,
  journal,
}: {
  readonly path: string;
  readonly daemonMarker: string;
  readonly journal: string;
}): { readonly [Symbol.dispose]: () => void; } {
  /**
   `PATH` as it was before this case.
   */
  const originalPath = process.env.PATH ?? '';
  process.env.PATH = path;
  process.env[STAND_IN_MARKER_VARIABLE] = daemonMarker;
  process.env[JOURNAL_VARIABLE] = journal;
  return {
    [Symbol.dispose](): void {
      process.env.PATH = originalPath;
      // Both markers are unset for every other case, so removing them is the
      // state they had.
      delete process.env.KEY_HELPER_TEST_DAEMON_MARKER;
      delete process.env.KEY_HELPER_TEST_JOURNAL;
    },
  };
}

/**
 Install a stand-in program that runs one line of shell.
 
 @param path - Absolute path the program must occupy
 @param body - Shell body the program runs
 
 @example
 ```ts
 await writeProgram({ path, body: 'echo ran' });
 ```
 */
async function writeProgram({ path, body }: {
  readonly path: string;
  readonly body: string;
}): Promise<void> {
  await writeFile(
    path,
    `#!/bin/sh\n${body}\n`,
    'utf8',
  );
  await chmod(
    path,
    EXECUTABLE_FILE_MODE,
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildAutostartDesktopEntry.name,
      children: [
        it({
          name: 'encodes a binary path with a space and a single quote for both grammars',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Checkout directory whose name holds the two characters that used to
             break the entry.
             */
            const checkout = join(
              folder.path,
              AWKWARD_CHECKOUT_DIRECTORY,
            );
            await mkdir(checkout);
            /**
             Path of the stand-in daemon, which is the path under test.
             */
            const binPath = join(
              checkout,
              DAEMON_BINARY_NAME,
            );
            /**
             File the stand-in daemon writes its marker into.
             */
            const daemonMarker = join(
              folder.path,
              'daemon-marker.txt',
            );
            /**
             File the stand-in `systemd-cat` records the journal into.
             */
            const journal = join(
              folder.path,
              'journal.txt',
            );
            /**
             Folder holding the stand-in `systemd-cat`.
             */
            const standInBin = join(
              folder.path,
              'bin',
            );
            await mkdir(standInBin);
            await writeProgram({
              path: binPath,
              body: `printf '%s\\n' "${DAEMON_MARKER_TEXT}" > "$${STAND_IN_MARKER_VARIABLE}"`,
            });
            await writeProgram({
              path: join(
                standInBin,
                'systemd-cat',
              ),
              body: `cat > "$${JOURNAL_VARIABLE}"\nexit ${String(SYSTEMD_CAT_EXIT_CODE)}`,
            });
            using _environment = overrideEnvironment({
              path: `${standInBin}${delimiter}${process.env.PATH ?? ''}`,
              daemonMarker,
              journal,
            });
            /**
             The entry the install would write.
             */
            const entryText = buildAutostartDesktopEntry({ binPath, });
            /**
             Argument vector a launcher reads out of that entry.
             */
            const argv = execArguments({ entryText, });
            /**
             Shell the entry runs, its option, and whatever it passes after it.
             */
            const [shellPath, commandOption, ...scriptArguments] = argv;
            expect([shellPath, commandOption,]).toEqual([SHELL_PATH, SHELL_COMMAND_OPTION,]);
            /**
             Script the entry hands to the shell.
             */
            const [script,] = scriptArguments;
            expect(script).toBeDefined();
            if (script === undefined) {
              throw new Error('the entry handed the shell no script',);
            }
            // The shell script itself must parse, whatever the path holds.
            await runCommand({
              command: SHELL_PATH,
              args: ['-n', '-c', script,],
            });
            // Running what the entry parsed must exec exactly that path and
            // pipe its output into systemd-cat.
            await runCommand({
              command: SHELL_PATH,
              args: [SHELL_COMMAND_OPTION, script,],
            });
            expect(await readFile(
              daemonMarker,
              'utf8',
            )).toBe(`${DAEMON_MARKER_TEXT}\n`);
            expect(await readFile(
              journal,
              'utf8',
            )).toBe('');
          },
        }),
        it({
          name: 'leaves an ordinary path readable in the entry',
          fn: async () => {
            /**
             Path with nothing the two grammars reserve.
             */
            const binPath = '/home/someone/checkout/key-helper-service';
            /**
             Commands the entry would run.
             */
            const argv = execArguments({ entryText: buildAutostartDesktopEntry({ binPath, }), });
            expect(argv).toEqual([
              SHELL_PATH,
              SHELL_COMMAND_OPTION,
              `exec '${binPath}' 2>&1 | systemd-cat -t key-helper -p debug`,
            ]);
          },
        }),
      ],
    }),
    describe({
      name: writeAutostartEntry.name,
      children: [
        it({
          name: 'writes the entry into the home directory it is given',
          fn: async () => {
            await using folder = await makeFolder();
            await writeAutostartEntry({
              homeDirectory: folder.path,
              binPath: '/opt/key-helper-service',
            });
            expect(await readFile(
              join(
                folder.path,
                AUTOSTART_RELATIVE_DIRECTORY,
                AUTOSTART_FILE_NAME,
              ),
              'utf8',
            )).toBe(buildAutostartDesktopEntry({ binPath: '/opt/key-helper-service', }));
          },
        }),
      ],
    }),
    describe({
      name: reconcileKwinScriptLink.name,
      children: [
        it({
          name: 'creates the link when nothing is there',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Target the link must point at.
             */
            const kwinTarget = join(
              folder.path,
              'checkout',
              KWIN_SCRIPT_DIRECTORY,
            );
            /**
             Path the link must occupy.
             */
            const linkPath = join(
              folder.path,
              KWIN_SCRIPT_LINK_NAME,
            );
            await reconcileKwinScriptLink({ linkPath, kwinTarget, });
            expect(await readlink(linkPath)).toBe(kwinTarget);
          },
        }),
        it({
          name: 'replaces a link that points somewhere else',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Target the link must end up pointing at.
             */
            const kwinTarget = join(
              folder.path,
              'checkout',
              KWIN_SCRIPT_DIRECTORY,
            );
            /**
             Path the link must occupy.
             */
            const linkPath = join(
              folder.path,
              KWIN_SCRIPT_LINK_NAME,
            );
            await mkdir(join(
              folder.path,
              'stale',
              KWIN_SCRIPT_DIRECTORY,
            ), { recursive: true, },);
            await reconcileKwinScriptLink({
              linkPath,
              kwinTarget: join(
                folder.path,
                'stale',
                KWIN_SCRIPT_DIRECTORY,
              ),
            });
            await reconcileKwinScriptLink({ linkPath, kwinTarget, });
            expect(await readlink(linkPath)).toBe(kwinTarget);
          },
        }),
        it({
          name: 'leaves an already correct link alone',
          fn: async ctx => {
            await using folder = await makeFolder();
            /**
             Target the link points at.
             */
            const kwinTarget = join(
              folder.path,
              'checkout',
              KWIN_SCRIPT_DIRECTORY,
            );
            /**
             Path the link occupies.
             */
            const linkPath = join(
              folder.path,
              KWIN_SCRIPT_LINK_NAME,
            );
            await reconcileKwinScriptLink({ linkPath, kwinTarget, });
            /** Recorder over the destination info records are written to. */
            const standardOutput = ctx.sinon.spy(process.stdout, 'write');
            /** Recorder over the destination debug and error records use. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await reconcileKwinScriptLink({ linkPath, kwinTarget, });
            await flushLogs();
            /**
             Log lines saying the link was already correct.
             */
            const unchanged = [
              ...standardOutput.args,
              ...standardError.args,
            ]
              .map(function lineText(call: readonly unknown[]): string {
                return String(call[0]);
              })
              .filter(function namesCorrectLink(line: string): boolean {
                return line.includes('link already correct');
              });
            expect(unchanged).toHaveLength(1);
            expect(await readlink(linkPath)).toBe(kwinTarget);
          },
        }),
        it({
          name: 'reports a path it cannot inspect instead of recreating it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Regular file standing where the link's parent directory should be,
             so the entry inside it cannot be inspected (ENOTDIR).
             */
            const blockedParent = join(
              folder.path,
              'blocked',
            );
            await writeFile(
              blockedParent,
              '',
              'utf8',
            );
            let caught: unknown;
            try {
              await reconcileKwinScriptLink({
                linkPath: join(
                  blockedParent,
                  KWIN_SCRIPT_LINK_NAME,
                ),
                kwinTarget: join(
                  folder.path,
                  KWIN_SCRIPT_DIRECTORY,
                ),
              });
            } catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(InstallError);
            expect((caught as Error).message).toContain('ENOTDIR');
            expect((caught as Error).message).toContain(blockedParent);
          },
        }),
      ],
    }),
    describe({
      name: installKeyHelper.name,
      children: [
        it({
          name: 'links the script, writes the entry and enables the script',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Stand-in checkout holding the script package and the binary.
             */
            const checkout = join(
              folder.path,
              AWKWARD_CHECKOUT_DIRECTORY,
            );
            await mkdir(
              join(
                checkout,
                KWIN_SCRIPT_DIRECTORY,
              ),
              { recursive: true, },
            );
            /**
             Home directory the install writes into.
             */
            const homeDirectory = join(
              folder.path,
              'home',
            );
            await mkdir(homeDirectory);
            /** Enablement calls the install made. */
            const enabled: number[] = [];
            await installKeyHelper({
              homeDirectory,
              packageDirectory: checkout,
              enableKwinScript: function enableKwinScript(): void {
                enabled.push(enabled.length);
              },
            });
            expect(
              await readlink(join(
                homeDirectory,
                KWIN_SCRIPTS_RELATIVE_DIRECTORY,
                KWIN_SCRIPT_LINK_NAME,
              )),
            ).toBe(join(
              checkout,
              KWIN_SCRIPT_DIRECTORY,
            ));
            expect(await readFile(
              join(
                homeDirectory,
                AUTOSTART_RELATIVE_DIRECTORY,
                AUTOSTART_FILE_NAME,
              ),
              'utf8',
            )).toBe(buildAutostartDesktopEntry({
              binPath: join(
                checkout,
                DAEMON_BINARY_NAME,
              ),
            },));
            expect(enabled).toHaveLength(1);
          },
        }),
      ],
    }),
  ],
});
