/**
 Launching the commit service program of the image.

 @module
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { realpath, } from 'node:fs/promises';

//region Constants

/**
 The service program, installed beside the wrapper by the image build.
 */
const SERVICE_PROGRAM = '/opt/native/cli-git-service';

/**
 The Git the service runs: the image's installed Git.
 */
const SERVICE_GIT = '/usr/bin/git';

/**
 Environment variable that arms one fault point of the service
 (`src/native/service/service_fault.rs`, compiled into the release executable).
 */
const FAULT_VARIABLE = 'CLI_GIT_TEST_ONLY_SERVICE_FAULT';

/**
 The 128-bit FNV-1a offset basis and prime of the instance name (`src/native/service/service_instance.rs`).
 */
const FNV_OFFSET = 0x6C_62_27_2E_07_BB_01_42_62_B8_21_75_62_95_C5_8Dn;

/**
 The prime of the hash, which is 2 to the 88th plus 315.
 */
const FNV_PRIME = 0x00_00_00_00_01_00_00_00_00_00_00_00_00_00_01_3Bn;

/**
 Bits of the hash.
 */
const FNV_BITS = 128n;

/**
 The 128-bit mask of the hash arithmetic.
 */
const FNV_MASK = (1n << FNV_BITS) - 1n;

/**
 Hexadecimal digits of an instance name: one per four bits of the hash.
 */
const INSTANCE_NAME_DIGITS = 32;

//endregion Constants

//region Instance

/**
 Resolve a path to its real path, or keep it as given when it cannot be resolved.

 @param path - the path

 @returns the real path, or the path itself when it does not exist

 @example
 ```ts
 await resolvedOrGiven('/work/repo/.git');
 ```
 */
async function resolvedOrGiven(path: string,): Promise<string> {
  try {
    return await realpath(path,);
  }
  catch (error) {
    // A directory that does not exist is hashed as given; any other failure is worth seeing.
    if (!(Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT')))
      console.error(`Cannot resolve ${path}: ${String(error,)}`,);
    return path;
  }
}

/**
 The name of the service instance of a worktree: 32 hexadecimal digits of the FNV-1a hash of the real path of its own Git
 directory, the name `src/native/service/service_instance.rs` gives it.
 The wrapper derives the socket from it, so the suite starts each worktree's service under the same name.

 @param gitDir - the worktree's own Git directory (`.git` of the main worktree, `.git/worktrees/<id>` of a linked one)

 @returns the instance name

 @example
 ```ts
 await instanceOf('/work/repo/.git'); // => '3f9a...'
 ```
 */
export async function instanceOf(gitDir: string,): Promise<string> {
  /**
   Real path, so that two spellings of one directory are one instance; a directory that cannot be resolved is hashed as given.
   */
  const real = await resolvedOrGiven(gitDir,);
  return Buffer.from(real,)
    .reduce(
    function step(
      hash: bigint,
      byte: number,
    ) {
      return ((hash ^ BigInt(byte,)) * FNV_PRIME) & FNV_MASK;
    },
    FNV_OFFSET,
    )
    .toString(16,)
    .padStart(
    INSTANCE_NAME_DIGITS,
    '0',
    );
}

//endregion Instance

//region Launch

/**
 Spawns the service program.

 @param socket - path of the unix socket it listens on

 @param journal - directory of its journal

 @param instance - name of the worktree instance it serves, which it refuses every other worktree's messages for

 @param fault - fault point that makes it end itself, none for a clean service

 @returns the process, whose standard error is a pipe

 @example
 ```ts
 const child = spawnService({ socket: '/work/s1/service/runtime/cli-git/service.sock', journal: '/work/s1/service/journal' });
 ```
 */
export function spawnService({
  socket,
  journal,
  instance,
  fault,
}: Readonly<{
  socket: string;
  journal: string;
  instance: string;
  fault?: string;
}>,): ChildProcess {
  return spawn(
    SERVICE_PROGRAM,
    [
      '--journal',
      journal,
      '--git',
      SERVICE_GIT,
      '--path',
      socket,
      '--instance',
      instance,
    ],
    {
      stdio: [
        'ignore',
        'ignore',
        'pipe',
      ],
      env: fault === undefined ? process.env : {
        ...process.env,
        [FAULT_VARIABLE]: fault,
      },
    },
  );
}

//endregion Launch
