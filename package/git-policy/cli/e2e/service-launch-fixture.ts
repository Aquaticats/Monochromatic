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
 The Git the service runs, which the image keeps off the default path.
 */
const SERVICE_GIT = '/opt/git/2.55.0/bin/git';

/**
 Environment variable that arms one fault point of the service
 (`src/native/service_fault.rs`, compiled into the release executable).
 */
const FAULT_VARIABLE = 'CLI_GIT_TEST_ONLY_SERVICE_FAULT';

/**
 The 128-bit FNV-1a offset basis and prime of the instance name (`src/native/service_instance.rs`).
 */
const FNV_OFFSET = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58dn;

const FNV_PRIME = 0x0000_0000_0100_0000_0000_0000_0000_013bn;

/**
 The 128-bit mask of the hash arithmetic.
 */
const FNV_MASK = (1n << 128n) - 1n;

//endregion Constants

//region Instance

/**
 The name of the service instance of a worktree: 32 hexadecimal digits of the FNV-1a hash of the real path of its own Git
 directory, the name `src/native/service_instance.rs` gives it.
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
  let real = gitDir;
  try {
    real = await realpath(gitDir,);
  }
  catch {
    real = gitDir;
  }
  /**
   Running hash.
   */
  let hash = FNV_OFFSET;
  for (const byte of Buffer.from(real,)) {
    hash ^= BigInt(byte,);
    hash = (hash * FNV_PRIME) & FNV_MASK;
  }
  return hash.toString(16,)
    .padStart(
      32,
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
