/**
 `repository-link-check`: asks whether the repository link is usable, prints the answer's plain message, and exits 0
 when ready, 75 (`EX_TEMPFAIL`) when not, or 128 with `--for=git` (Git's own fatal status, for the Git wrapper).
 Options: `--identity=<file>`, `--for=git`, `--deadline-seconds=<n>`. It creates nothing.

 @module
 */

import { homedir, } from 'node:os';
import {
  argv,
  exit,
} from 'node:process';
import { checkRepositoryLink, } from './check-link.ts';
import { readIdentity, } from './identity-file.ts';
import { realProbe, } from './real-probe.ts';
import { parseBinOptions, } from './run-options.ts';

/**
 The options of this call.
 */
const options = parseBinOptions({
  argv: argv.slice(2,),
  home: homedir(),
},);

/**
 The check's answer.
 */
const answer = await checkRepositoryLink({
  read: await readIdentity(options.identityFile,),
  probe: realProbe,
  deadlineSeconds: options.deadlineSeconds,
},);

if (answer.ready) {
  console.log(answer.message,);
}
else {
  console.error(answer.message,);
  exit(options.failureStatus,);
}
