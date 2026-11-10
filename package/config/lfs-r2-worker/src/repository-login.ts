/**
 Point one repository's own Git settings at the private LFS credential file.

 Only that repository is touched: the key is written to its local scope, which
 every linked worktree shares, and no global or system setting is read for its
 value or written at all. The key names one host, so any other repository, and any
 other host, keeps asking nobody.

 The repository has to name the host in its own `.lfsconfig` first, which is what
 makes this a deliberate act on a clone the person chose rather than a rule that
 follows every checkout on the machine.

 @module
 */

import {
  access,
  mkdir,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

import {
  type CredentialHelper,
  parseLfsHost,
  storeCredentialHelper,
} from './credential-helper.ts';
import {
  GitCommandError,
  runGit,
} from './git-command.ts';

/**
 Permission bits of the directory that holds the private credential file.
 */
const CREDENTIAL_DIRECTORY_MODE = 0o700;

/**
 A repository that cannot be given the login, and why.
 */
export class RepositoryLoginError extends Error {
  /**
   @param message - what is missing and what to do about it
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'RepositoryLoginError';
  }
}

/**
 Where the helper went and what it names.
 */
export type RepositoryLogin = {
  /**
   Whether this call changed anything; `false` when the helper already stood as asked.
   */
  readonly changed: boolean;
  /**
   Helper command the key holds.
   */
  readonly command: string;
  /**
   Absolute path of the Git config file that received the key.
   */
  readonly configFile: string;
  /**
   Absolute path of the private credential file the helper reads.
   */
  readonly file: string;
  /**
   Config key that was written.
   */
  readonly key: string;
  /**
   Directory the login was installed for.
   */
  readonly repository: string;
};

/**
 Parameters for {@link installRepositoryLogin}.
 */
export type InstallRepositoryLoginParams = {
  /**
   Repository or linked worktree to configure.
   */
  readonly repository: string;
  /**
   LFS host the helper serves, as an absolute URL.
   */
  readonly host: string;
  /**
   Absolute path of the private credential file the helper reads.
   */
  readonly credentialFile: string;
};

/**
 Whether a path exists, whatever kind of entry it is.

 @param path - path to look for

 @returns `true` when the path exists

 @throws when the path cannot be looked up for a reason other than being absent
 */
async function exists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (((typeof error) === 'object') && (error !== null)
      && ('code' in error)
      && (error.code === 'ENOENT')) {
      return false;
    }
    throw error;
  }
}

/**
 Read every `lfs.url` a `.lfsconfig` declares.

 @param lfsConfig - absolute path of the repository's `.lfsconfig`

 @returns the declared URLs, in file order

 @throws {@link RepositoryLoginError} when the path does not exist or Git cannot read it
 */
async function readLfsUrls(lfsConfig: string,): Promise<readonly string[]> {
  if (!(await exists(lfsConfig,))) {
    throw new RepositoryLoginError(`no .lfsconfig at ${lfsConfig}; this repository declares no LFS server`,);
  }
  try {
    return (await runGit({
      args: [
        'config',
        '--file',
        lfsConfig,
        '--get-all',
        'lfs.url',
      ],
      cwd: dirname(lfsConfig,),
    },))
      .split('\n',)
      .map(function trim(line: string,): string {
        return line.trim();
      },)
      .filter(function keep(line: string,): boolean {
        return line !== '';
      },);
  }
  catch (error) {
    throw new RepositoryLoginError(`cannot read lfs.url from ${lfsConfig}: ${String(error,)}`,);
  }
}

/**
 Resolve the repository's shared Git directory, and so the config file every
 worktree reads.

 @param repository - repository or linked worktree

 @returns absolute path of the shared Git directory
 */
async function commonGitDirectory(repository: string,): Promise<string> {
  return (await runGit({
    args: [
      'rev-parse',
      '--path-format=absolute',
      '--git-common-dir',
    ],
    cwd: repository,
  },)).trim();
}

/**
 Whether the repository's `.lfsconfig` names the host the helper is for.

 @param urls - URLs the repository declares

 @param origin - scheme and authority of the host

 @returns `true` when one declared URL has that origin
 */
function namesOrigin({
  urls,
  origin,
}: {
  readonly urls: readonly string[];
  readonly origin: string
},): boolean {
  return urls.some(function hasOrigin(url: string,): boolean {
    return URL.canParse(url,) && (new URL(url,).origin === origin);
  },);
}

/**
 Read every value of one local config key.

 Git answers exit status 1 when a key is absent, which is a normal answer here and
 not a failure, so it becomes an empty list.

 @param repository - repository or linked worktree

 @param key - config key to read

 @returns the values the repository's own configuration holds, in order; empty when it holds none
 */
async function localValues({
  repository,
  key,
}: {
  readonly repository: string;
  readonly key: string
},): Promise<readonly string[]> {
  try {
    return (await runGit({
      args: [
        'config',
        '--local',
        '--get-all',
        key,
      ],
      cwd: repository,
    },))
      .split('\n',)
      .map(function trim(line: string,): string {
        return line.trim();
      },)
      .filter(function keep(line: string,): boolean {
        return line !== '';
      },);
  }
  catch (error) {
    if ((error instanceof GitCommandError) && (error.status === 1)) {
      return [];
    }
    throw error;
  }
}

/**
 Whether Git sees a work tree at this directory.

 A directory that is not a repository at all is the same answer as one outside
 every work tree, so both become `false` here and one message covers them.

 @param repository - directory to ask about

 @returns `true` when Git answers that the directory is inside a work tree
 */
async function isInsideWorkTree(repository: string,): Promise<boolean> {
  try {
    return (await runGit({
      args: [
        'rev-parse',
        '--is-inside-work-tree',
      ],
      cwd: repository,
    },)).trim() === 'true';
  }
  catch (error) {
    if (error instanceof GitCommandError) {
      return false;
    }
    throw error;
  }
}

/**
 Give one repository a host-scoped credential helper that reads the private file.

 @param repository - repository or linked worktree to configure

 @param host - LFS host the helper serves, as an absolute URL

 @param credentialFile - absolute path of the private credential file

 @returns where the helper went and what it names

 @throws {@link RepositoryLoginError} when the directory is not inside a work tree, or its
   `.lfsconfig` does not name the host

 @example
 ```ts
 await installRepositoryLogin({ repository: process.cwd(), host, credentialFile });
 ```
 */
export async function installRepositoryLogin({
  repository,
  host,
  credentialFile,
}: InstallRepositoryLoginParams,): Promise<RepositoryLogin> {
  /**
   Host split into the parts a credential request and config key need.
   */
  const lfsHost = parseLfsHost({ host, },);
  if (!(await isInsideWorkTree(repository,))) {
    throw new RepositoryLoginError(`${repository} is not a Git work tree, so it has no local configuration to write`,);
  }
  /**
   URLs the repository's own LFS configuration declares.
   */
  const urls = await readLfsUrls(`${repository}/.lfsconfig`,);
  if (!namesOrigin({
    urls,
    origin: lfsHost.origin,
  },)) {
    throw new RepositoryLoginError(`${repository}/.lfsconfig does not name ${lfsHost.origin}; run this inside a clone of the repository whose LFS server is ${lfsHost.origin}`,);
  }
  /**
   Config key and helper command for that host and file.
   */
  const helper: CredentialHelper = storeCredentialHelper({
    host: lfsHost,
    credentialFile,
  },);
  /**
   Values the key held before this call.
   */
  const existing = await localValues({
    repository,
    key: helper.key,
  },);
  /**
   Whether one helper and no other already stood there, so this call changes nothing.
   */
  const alreadySet = (existing.length === 1) && (existing[0] === helper.command);
  await mkdir(
    dirname(credentialFile,),
    {
      recursive: true,
      mode: CREDENTIAL_DIRECTORY_MODE,
    },
  );
  if (!alreadySet) {
    await runGit({
      args: [
        'config',
        '--local',
        '--replace-all',
        helper.key,
        helper.command,
      ],
      cwd: repository,
    },);
  }
  return {
    changed: !alreadySet,
    command: helper.command,
    configFile: `${await commonGitDirectory(repository,)}/config`,
    file: credentialFile,
    key: helper.key,
    repository,
  };
}
