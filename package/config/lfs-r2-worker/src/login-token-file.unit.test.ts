/**
 Tests for storing one machine's token: a throwaway directory per case, with real
 Git writing and reading the private credential file.

 @module
 */

import { execFileSync, } from 'node:child_process';
import {
  chmodSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  statSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  CREDENTIAL_FILE_MODE,
  LoginTokenError,
  storeLoginToken,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 LFS host every case stores a credential for.
 */
const HOST = 'https://lfs.example.test';

/**
 The machine's client id.
 */
const CLIENT_ID = 'machine-one';

/**
 The machine's token; made up, and shaped like the hex token the runbook generates.
 */
const TOKEN = 'a'.repeat(64,);

/**
 Environment every Git command in these cases runs with, so nothing can open a
 password dialog on the machine running the suite.
 */
const GIT_ENVIRONMENT = {
  ...process.env,
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};

/**
 Private file path inside a fresh temporary directory.

 @returns absolute path of a credential file that does not exist yet
 */
function credentialFileInFreshDirectory(): string {
  return join(mkdtempSync(join(scratchRoot(), 'lfs-token-',),), 'config', 'credentials',);
}

/**
 Ask the stored file what it answers for a host, through the same helper a
 `git-lfs` push uses.

 @param credentialFile - private credential file

 @param host - LFS host to ask about

 @param configArguments - Git options that select the helper

 @returns the fields Git answered with

 @throws when Git has no credential to answer, which is what a missing entry looks like
 */
function fill({
  credentialFile,
  host,
  configArguments,
}: {
  readonly credentialFile: string;
  readonly host: string;
  readonly configArguments: readonly string[];
},): Readonly<Record<string, string>> {
  /**
   Parsed host the request is built from.
   */
  const url = new URL(host,);
  /**
   What Git answered.
   */
  const output = execFileSync(
    'git',
    [
      ...configArguments,
      'credential',
      'fill',
    ],
    {
      cwd: join(credentialFile, '..'),
      encoding: 'utf8',
      env: GIT_ENVIRONMENT,
      input: `protocol=${url.protocol.slice(0, -1,)}\nhost=${url.host}\n\n`,
    },
  );
  return Object.fromEntries(output.split('\n',)
    .filter(function hasField(line: string,): boolean {
      return line.includes('=',);
    },)
    .map(function toEntry(line: string,): readonly [string, string] {
      /**
       Index of the `=` that separates a field name from its value.
       */
      const separator = line.indexOf('=',);
      return [
        line.slice(
          0,
          separator,
        ),
        line.slice(separator + 1,),
      ];
    },),);
}

/**
 Git options that run the helper this package installs, and no other.

 @param credentialFile - private credential file

 @param host - LFS host the helper serves

 @returns arguments placed before the `credential` subcommand
 */
function helperArguments({
  credentialFile,
  host,
}: { readonly credentialFile: string; readonly host: string; },): readonly string[] {
  return [
    '-c',
    'credential.helper=',
    '-c',
    `credential.${new URL(host,).origin}.helper=store --file '${credentialFile}'`,
  ];
}

/**
 Run a case and capture the error it raised.

 @param fn - case body

 @returns the error, or `undefined` when nothing was raised
 */
async function captureError(fn: () => Promise<unknown>,): Promise<unknown> {
  try {
    await fn();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: storeLoginToken.name,
      children: [
        it({
          name: 'creates the file at mode 0600 and lets Git answer with the pair',
          fn: async () => {
            /**
             Private file the token goes into.
             */
            const credentialFile = credentialFileInFreshDirectory();
            /**
             What the store reported.
             */
            const result = await storeLoginToken({
              host: HOST,
              clientId: CLIENT_ID,
              credentialFile,
              token: TOKEN,
            },);
            expect(result.clientId,).toBe(CLIENT_ID,);
            expect(result.credentialFile,).toBe(credentialFile,);
            expect(result.key,).toBe(`credential.${HOST}.helper`,);
            expect(statSync(credentialFile,).mode & 0o777,).toBe(CREDENTIAL_FILE_MODE,);
            expect(fill({
              credentialFile,
              host: HOST,
              configArguments: helperArguments({
                credentialFile,
                host: HOST,
              },),
            },),).toMatchObject({
              username: CLIENT_ID,
              password: TOKEN,
            },);
          },
        },),
        it({
          name: 'strips one trailing line ending from a piped token',
          fn: async () => {
            /**
             Private file the token goes into.
             */
            const credentialFile = credentialFileInFreshDirectory();
            await storeLoginToken({
              host: HOST,
              clientId: CLIENT_ID,
              credentialFile,
              token: `${TOKEN}\n`,
            },);
            expect(fill({
              credentialFile,
              host: HOST,
              configArguments: helperArguments({
                credentialFile,
                host: HOST,
              },),
            },).password,).toBe(TOKEN,);
            expect(readFileSync(credentialFile, 'utf8',),).not.toContain('\n\n',);
          },
        },),
        it({
          name: 'tightens the mode of a file that already existed',
          fn: async () => {
            /**
             Private file that already exists, readable by everyone.
             */
            const credentialFile = credentialFileInFreshDirectory();
            await storeLoginToken({
              host: HOST,
              clientId: CLIENT_ID,
              credentialFile,
              token: TOKEN,
            },);
            chmodSync(
              credentialFile,
              0o644,
            );
            await storeLoginToken({
              host: HOST,
              clientId: CLIENT_ID,
              credentialFile,
              token: TOKEN,
            },);
            expect(statSync(credentialFile,).mode & 0o777,).toBe(CREDENTIAL_FILE_MODE,);
          },
        },),
        it({
          name: 'keeps the entry of another host in the same file',
          fn: async () => {
            /**
             Private file both credentials go into.
             */
            const credentialFile = credentialFileInFreshDirectory();
            await storeLoginToken({
              host: HOST,
              clientId: CLIENT_ID,
              credentialFile,
              token: TOKEN,
            },);
            await storeLoginToken({
              host: 'https://other.example.test',
              clientId: 'machine-two',
              credentialFile,
              token: 'b'.repeat(64,),
            },);
            expect(fill({
              credentialFile,
              host: HOST,
              configArguments: helperArguments({
                credentialFile,
                host: HOST,
              },),
            },),).toMatchObject({
              username: CLIENT_ID,
              password: TOKEN,
            },);
            expect(readFileSync(credentialFile, 'utf8',),).toContain('other.example.test',);
          },
        },),
        it({
          name: 'answers nothing for a host it holds no credential for',
          fn: async () => {
            /**
             Private file holding one host's credential.
             */
            const credentialFile = credentialFileInFreshDirectory();
            await storeLoginToken({
              host: HOST,
              clientId: CLIENT_ID,
              credentialFile,
              token: TOKEN,
            },);
            /**
             Error Git raised when asked about another host.
             */
            const error = await captureError(async () => fill({
              credentialFile,
              host: 'https://other.example.test',
              configArguments: helperArguments({
                credentialFile,
                host: 'https://other.example.test',
              },),
            },),);
            expect(error,).toBeDefined();
          },
        },),
        it({
          name: 'writes through its own helper alone, so a competing helper receives nothing',
          fn: async () => {
            /**
             Directory that is also a repository, so its own helper applies to any Git
             command run there.
             */
            const repository = mkdtempSync(join(scratchRoot(), 'lfs-token-repository-',),);
            execFileSync(
              'git',
              ['init', '--quiet',],
              {
                cwd: repository,
                env: GIT_ENVIRONMENT,
              },
            );
            /**
             File the competing helper would write.
             */
            const competing = join(repository, 'competing-credentials',);
            execFileSync(
              'git',
              [
                'config',
                '--local',
                'credential.helper',
                `store --file ${competing}`,
              ],
              {
                cwd: repository,
                env: GIT_ENVIRONMENT,
              },
            );
            /**
             Private file the token goes into, inside that repository so the competing
             helper is in scope.
             */
            const credentialFile = join(repository, 'credentials',);
            await storeLoginToken({
              host: HOST,
              clientId: CLIENT_ID,
              credentialFile,
              token: TOKEN,
            },);
            expect(existsSync(competing,),).toBe(false,);
          },
        },),
        ...([
          ['an empty token', '', CLIENT_ID, 'empty',],
          ['a token holding a line break', 'first\nsecond', CLIENT_ID, 'whitespace',],
          ['an empty client id', TOKEN, '', 'empty',],
          ['a client id holding a space', TOKEN, 'two words', 'whitespace',],
        ] as const).map(function mapRefused([description, token, clientId, expected,]: readonly [string, string, string, string,],) {
          return it({
            name: `refuses ${description}`,
            fn: async () => {
              /**
               Private file the refused value would have gone into.
               */
              const credentialFile = credentialFileInFreshDirectory();
              /**
               Error the store raised.
               */
              const error = await captureError(async () => await storeLoginToken({
                host: HOST,
                clientId,
                credentialFile,
                token,
              },),);
              expect(error,).toBeInstanceOf(LoginTokenError,);
              expect((error as Error).message,).toContain(expected,);
            },
          },);
        },),
      ],
    },),
  ],
},);
