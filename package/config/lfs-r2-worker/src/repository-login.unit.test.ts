/**
 Tests for the repository login: a throwaway repository per case, with a real Git
 reading and writing the configuration the helper lives in.

 @module
 */

import { execFileSync, } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  installRepositoryLogin,
  RepositoryLoginError,
  type RepositoryLogin,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 LFS host every case configures.
 */
const HOST = 'https://lfs.example.test';

/**
 Credential file every case points the helper at, inside the fixture.
 */
const CREDENTIAL_FILE_NAME = 'credentials';

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
 Create an empty directory under the test scratch root, never the system temporary
 directory, whose backing device is unknowable.

 @param prefix - prefix of the directory name

 @returns absolute path of the new directory
 */
function temporaryDirectory(prefix: string,): string {
  return mkdtempSync(join(scratchRoot(), prefix,),);
}

/**
 Run one Git command in a fixture and answer its standard output.

 @param cwd - directory the command runs in

 @param args - arguments after the program name

 @returns standard output, trimmed
 */
function git(cwd: string, args: readonly string[],): string {
  return execFileSync(
    'git',
    [...args,],
    {
      cwd,
      encoding: 'utf8',
      env: GIT_ENVIRONMENT,
    },
  ).trim();
}

/**
 Create a repository whose `.lfsconfig` names `url`, committed so a linked
 worktree checks it out too.

 @param url - `lfs.url` the repository declares; absent for a repository without `.lfsconfig`

 @returns absolute path of the repository
 */
function repositoryDeclaring(url?: string,): string {
  /**
   The new repository.
   */
  const repository = temporaryDirectory('lfs-login-repository-',);
  git(
    repository,
    ['init', '--quiet',],
  );
  if (url !== undefined) {
    writeFileSync(
      join(repository, '.lfsconfig',),
      `[lfs]\n\turl = ${url}\n`,
    );
  }
  git(
    repository,
    [
      '-c',
      'user.name=probe',
      '-c',
      'user.email=probe@example.test',
      'commit',
      '--quiet',
      '--allow-empty',
      '--message=root',
      ...url === undefined ? [] : ['.lfsconfig',],
    ],
  );
  return repository;
}

/**
 All values a repository's own configuration holds for one key.

 @param repository - repository to read

 @param key - config key to read

 @returns values in order, empty when the key is absent
 */
function localValues(repository: string, key: string,): readonly string[] {
  try {
    return git(
      repository,
      [
        'config',
        '--local',
        '--get-all',
        key,
      ],
    ).split('\n',)
      .filter(function keep(line: string,): boolean {
        return line !== '';
      },);
  }
  catch (error) {
    if (((typeof error) === 'object') && (error !== null) && ('status' in error) && (error.status === 1)) {
      return [];
    }
    throw error;
  }
}

/**
 Install the login for a fixture repository.

 @param repository - repository to configure

 @param credentialFile - private file the helper should read

 @returns what the install reported
 */
async function install(repository: string, credentialFile: string,): Promise<RepositoryLogin> {
  return await installRepositoryLogin({
    repository,
    host: HOST,
    credentialFile,
  },);
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
      name: installRepositoryLogin.name,
      children: [
        it({
          name: 'writes the helper into the repository its own configuration, and nowhere else',
          fn: async () => {
            /**
             Repository being configured.
             */
            const repository = repositoryDeclaring(HOST,);
            /**
             Private file the helper reads, outside the repository's `.git`.
             */
            const credentialFile = join(temporaryDirectory('lfs-login-file-',), CREDENTIAL_FILE_NAME,);
            /**
             What the install reported.
             */
            const result = await install(repository, credentialFile,);
            expect(result.changed,).toBe(true,);
            expect(result.key,).toBe(`credential.${HOST}.helper`,);
            expect(result.command,).toBe(`store --file '${credentialFile}'`,);
            expect(result.file,).toBe(credentialFile,);
            expect(result.repository,).toBe(repository,);
            expect(localValues(repository, result.key,),).toEqual([result.command,],);
            expect(result.configFile,).toBe(join(repository, '.git', 'config',),);
            expect(readFileSync(result.configFile, 'utf8',),).toContain(`[credential "${HOST}"]`,);
            expect(readFileSync(result.configFile, 'utf8',),).toContain(`helper = ${result.command}`,);
            expect(existsSync(credentialFile,),).toBe(false,);
          },
        },),
        it({
          name: 'creates the credential directory so the helper can write the file later',
          fn: async () => {
            /**
             Repository being configured.
             */
            const repository = repositoryDeclaring(HOST,);
            /**
             Directory that does not exist yet, with the file inside it.
             */
            const directory = join(temporaryDirectory('lfs-login-file-',), 'nested', 'deeper',);
            await install(repository, join(directory, CREDENTIAL_FILE_NAME,),);
            expect(existsSync(directory,),).toBe(true,);
          },
        },),
        it({
          name: 'changes nothing on a second run and keeps exactly one helper',
          fn: async () => {
            /**
             Repository being configured.
             */
            const repository = repositoryDeclaring(HOST,);
            /**
             Private file the helper reads.
             */
            const credentialFile = join(temporaryDirectory('lfs-login-file-',), CREDENTIAL_FILE_NAME,);
            /**
             What the first run reported.
             */
            const first = await install(repository, credentialFile,);
            /**
             What the second run reported.
             */
            const second = await install(repository, credentialFile,);
            expect(first.changed,).toBe(true,);
            expect(second.changed,).toBe(false,);
            expect(localValues(repository, second.key,),).toEqual([second.command,],);
          },
        },),
        it({
          name: 'replaces a different helper for the same host instead of adding a second one',
          fn: async () => {
            /**
             Repository being configured.
             */
            const repository = repositoryDeclaring(HOST,);
            /**
             Key the helper goes under.
             */
            const key = `credential.${HOST}.helper`;
            git(
              repository,
              [
                'config',
                '--local',
                '--replace-all',
                key,
                'cache --timeout=3600',
              ],
            );
            /**
             Private file the new helper reads.
             */
            const credentialFile = join(temporaryDirectory('lfs-login-file-',), CREDENTIAL_FILE_NAME,);
            /**
             What the install reported.
             */
            const result = await install(repository, credentialFile,);
            expect(result.changed,).toBe(true,);
            expect(localValues(repository, key,),).toEqual([result.command,],);
          },
        },),
        it({
          name: 'writes the configuration every linked worktree of the repository reads',
          fn: async () => {
            /**
             Repository being configured.
             */
            const repository = repositoryDeclaring(HOST,);
            /**
             Linked worktree the login is installed from.
             */
            const worktree = join(temporaryDirectory('lfs-login-worktree-',), 'checkout',);
            git(
              repository,
              [
                'worktree',
                'add',
                '--quiet',
                worktree,
              ],
            );
            /**
             Private file the helper reads.
             */
            const credentialFile = join(temporaryDirectory('lfs-login-file-',), CREDENTIAL_FILE_NAME,);
            /**
             What the install reported when run from the linked worktree.
             */
            const result = await install(worktree, credentialFile,);
            expect(result.repository,).toBe(worktree,);
            expect(result.configFile,).toBe(join(repository, '.git', 'config',),);
            expect(localValues(repository, result.key,),).toEqual([result.command,],);
            expect(localValues(worktree, result.key,),).toEqual([result.command,],);
            expect(
              existsSync(join(worktree, '.git', 'config',),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'accepts an .lfsconfig that names the host with a path after it',
          fn: async () => {
            /**
             Repository whose url carries a path, which the origin still matches.
             */
            const repository = repositoryDeclaring(`${HOST}/objects`,);
            /**
             What the install reported.
             */
            const result = await install(repository, join(temporaryDirectory('lfs-login-file-',), CREDENTIAL_FILE_NAME,),);
            expect(result.changed,).toBe(true,);
            expect(result.key,).toBe(`credential.${HOST}.helper`,);
          },
        },),
        it({
          name: 'refuses a repository with no .lfsconfig',
          fn: async () => {
            /**
             Repository that declares no LFS server.
             */
            const repository = repositoryDeclaring();
            /**
             Error the install raised.
             */
            const error = await captureError(async () => await install(repository, join(temporaryDirectory('lfs-login-file-',), CREDENTIAL_FILE_NAME,),),);
            expect(error,).toBeInstanceOf(RepositoryLoginError,);
            expect((error as Error).message,).toContain('.lfsconfig',);
          },
        },),
        it({
          name: 'refuses an .lfsconfig that names another host',
          fn: async () => {
            /**
             Repository whose LFS server is not the one the login is for.
             */
            const repository = repositoryDeclaring('https://elsewhere.example.test',);
            /**
             Error the install raised.
             */
            const error = await captureError(async () => await install(repository, join(temporaryDirectory('lfs-login-file-',), CREDENTIAL_FILE_NAME,),),);
            expect(error,).toBeInstanceOf(RepositoryLoginError,);
            expect((error as Error).message,).toContain(HOST,);
          },
        },),
        it({
          name: 'refuses a directory that is not a Git work tree',
          fn: async () => {
            /**
             Directory that is not a repository at all.
             */
            const plain = temporaryDirectory('lfs-login-plain-',);
            mkdirSync(join(plain, '.lfsconfig-dir',), { recursive: true, },);
            writeFileSync(
              join(plain, '.lfsconfig',),
              `[lfs]\n\turl = ${HOST}\n`,
            );
            /**
             Error the install raised.
             */
            const error = await captureError(async () => await install(plain, join(temporaryDirectory('lfs-login-file-',), CREDENTIAL_FILE_NAME,),),);
            expect(error,).toBeInstanceOf(RepositoryLoginError,);
            expect((error as Error).message,).toContain('not a Git work tree',);
          },
        },),
      ],
    },),
  ],
},);
