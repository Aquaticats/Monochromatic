/**
 Stand-in for `ssh`, used by the ovh backend's unit tests.

 Run as `node fixture.fake-ovh-ssh.ts <the arguments ssh would have received>`.
 It appends its arguments to the file `MVM_FAKE_OVH_SSH_CALLS` names, one JSON
 array per line, then exits with the status `MVM_FAKE_OVH_SSH_EXIT` names, `0`
 when it names none. Nothing here talks to a host or a network: it exists so a
 test can see the exact argv an operation built, and can make a readiness probe
 fail.

 @module
 */

import { appendFile, } from 'node:fs/promises';

/**
 Exit status meaning the remote side answered.
 */
const EXIT_OK = 0;

/**
 File the calls are appended to, `''` when the test wants no log.
 */
const callsFile = process.env
  .MVM_FAKE_OVH_SSH_CALLS
  ?? '';

if (callsFile !== '') {
  await appendFile(
    callsFile,
    `${JSON.stringify(process.argv
      .slice(2,),)}\n`,
  );
}

/**
 Status this invocation exits with.
 */
const requested = Number(process.env
  .MVM_FAKE_OVH_SSH_EXIT
  ?? '0',);

process.exitCode = Number.isFinite(requested,) ? requested : EXIT_OK;
