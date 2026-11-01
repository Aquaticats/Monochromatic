/**
 Launching the commit service program of the image.

 @module
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';

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

//endregion Constants

//region Launch

/**
 Spawns the service program.

 @param socket - path of the unix socket it listens on

 @param journal - directory of its journal

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
  fault,
}: Readonly<{
  socket: string;
  journal: string;
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
