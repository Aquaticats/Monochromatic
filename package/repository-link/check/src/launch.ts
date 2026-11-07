/**
 `repository-link-launch`: started through a link named like the program it stands for (`spawn-claude`, `spawn-pi`), it
 checks the repository link and then replaces itself with that program, run by the same Node, with the arguments it was
 given. It exists so a link in `~/.local/bin` can lead to a program outside the repository: with the repository away
 the person reads the check's plain message, not the shell's "No such file or directory". It creates nothing.

 @module
 */

import { homedir, } from 'node:os';
import {
  basename,
  join,
} from 'node:path';
import {
  argv,
  env,
  execPath,
  execve,
  exit,
} from 'node:process';
import { checkRepositoryLink, } from './check-link.ts';
import {
  EXIT_TEMPFAIL,
  EXIT_UNKNOWN_PROGRAM,
  LAUNCHED_PROGRAMS,
} from './constants.ts';
import {
  identityPathFor,
  readIdentity,
} from './identity-file.ts';
import { realProbe, } from './real-probe.ts';

/**
 The name this bin was started through: the last segment of the path Node was given.
 */
const name = basename(argv[1] ?? '',);

/**
 The program's path below the repository link, when the name is known.
 */
const relative = Object.hasOwn(
  LAUNCHED_PROGRAMS,
  name,
)
  ? LAUNCHED_PROGRAMS[name]
  : undefined;
if (relative === undefined) {
  console.error(
    `repository-link-launch does not know a program named ${name}; it knows ${Object.keys(LAUNCHED_PROGRAMS,)
      .join(', ',)}. Make the link in ~/.local/bin with one of those names.`,
  );
  exit(EXIT_UNKNOWN_PROGRAM,);
}
/**
 What the identity file says.
 */
const read = await readIdentity(
  identityPathFor(homedir(),),
);
/**
 The check's answer.
 */
const answer = await checkRepositoryLink({
  read,
  probe: realProbe,
},);
if (!answer.ready) {
  console.error(answer.message,);
  exit(EXIT_TEMPFAIL,);
}
if ((typeof execve) !== 'function') {
  console.error('this node has no process.execve, so repository-link-launch cannot replace itself with the program',);
  exit(EXIT_TEMPFAIL,);
}
/**
 The program to run.
 */
const program = join(
  read.identity
    .linkPath,
  relative,
);
execve(
  execPath,
  [
    execPath,
    program,
    ...argv.slice(2,),
  ],
  env,
);
