/**
 `repository-link-run [--wait-seconds=<n>] [--identity=<file>] [--for=git] [--] <command...>`: checks the repository
 link and then replaces itself with the command. With `--wait-seconds` it asks again every second until the deadline,
 then fails with the last answer (for launchers of long-lived programs such as MCP servers that start in the login race).

 @module
 */

import { homedir, } from 'node:os';
import {
  argv,
  env,
  execve,
  exit,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { checkRepositoryLink, } from './check-link.ts';
import {
  MILLISECONDS_PER_SECOND,
  WAIT_STEP_SECONDS,
} from './constants.ts';
import { readIdentity, } from './identity-file.ts';
import { realProbe, } from './real-probe.ts';
import { resolveCommand, } from './resolve-command.ts';
import { parseBinOptions, } from './run-options.ts';

/**
 The options of this call.
 */
const options = parseBinOptions({
  argv: argv.slice(2,),
  home: homedir(),
},);

/**
 The moment after which the wait gives up, in milliseconds since the epoch.
 */
const giveUpAt = Date.now() + (options.waitSeconds
  * MILLISECONDS_PER_SECOND);

/**
 Asks once.

 @returns the answer
 */
async function ask(): ReturnType<typeof checkRepositoryLink> {
  return await checkRepositoryLink({
    read: await readIdentity(options.identityFile,),
    probe: realProbe,
    deadlineSeconds: options.deadlineSeconds,
  },);
}

/**
 Asks until the repository is ready or the wait is over.

 @returns the last answer
 */
async function askUntilReadyOrGiveUp(): ReturnType<typeof ask> {
  /**
   The answer of this step.
   */
  const answer = await ask();
  if (answer.ready || (Date.now() >= giveUpAt)) {
    return answer;
  }
  await wait(WAIT_STEP_SECONDS * MILLISECONDS_PER_SECOND,);
  return await askUntilReadyOrGiveUp();
}

/**
 The parts of the value this function uses, taken out of it.
 */
const [command = '', ...commandArguments] = options.rest;
if (command === '') {
  console.error('usage: repository-link-run [--wait-seconds=<n>] [--] <command...>',);
  exit(options.failureStatus,);
}
/**
 The answer of this step.
 */
const answer = await askUntilReadyOrGiveUp();
if (!answer.ready) {
  console.error(answer.message,);
  exit(options.failureStatus,);
}
if ((typeof execve) !== 'function') {
  console.error('this node has no process.execve, so repository-link-run cannot replace itself with the command',);
  exit(options.failureStatus,);
}
execve(
  await resolveCommand({
    command,
    pathVariable: env.PATH ?? '',
  },),
  [
    command,
    ...commandArguments,
  ],
  env,
);
