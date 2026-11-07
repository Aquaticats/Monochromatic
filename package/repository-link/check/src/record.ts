/**
 `repository-link-record --phase=before|after [--identity=<file>] [--link-path=<path>] [--mount-point=<path>]`: records
 the identity the check enforces, then prints the check's answer for it (exit 0 when ready). Phase `before` is for the
 install while the repository is still a real folder on the NVMe; phase `after` is the cut-over step that follows the swap.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import {
  argv,
  exit,
} from 'node:process';
import { EXIT_TEMPFAIL, } from './constants.ts';
import { recordIdentity, RecordIdentityError, } from './record-identity.ts';
import { realProbe, } from './real-probe.ts';
import { parseBinOptions, } from './run-options.ts';

/**
 The default mount point of the 860, the plan's `/var/mnt/encrypted`.
 */
const DEFAULT_MOUNT_POINT = '/var/mnt/encrypted';

/**
 The leading options of this call.
 */
const options = parseBinOptions({ argv: argv.slice(2,), home: homedir(), },);

/**
 Reads one `--name=value` among the arguments.

 @param name - the option name without dashes
 @returns the value, or an empty string
 */
function option(name: string,): string {
  const lead = `--${name}=`;
  const found = argv.slice(2,).find(function startsWithLead(argument,) {
    return argument.startsWith(lead,);
  },);
  return found === undefined ? '' : found.slice(lead.length,);
}

/**
 The phase to record.
 */
const phase = option('phase',);

if (phase !== 'before' && phase !== 'after') {
  console.error('usage: repository-link-record --phase=before|after [--identity=<file>] [--link-path=<path>] [--mount-point=<path>]',);
  exit(EXIT_TEMPFAIL,);
}
try {
  const { identity, answer, } = await recordIdentity({
    phase,
    identityFile: options.identityFile,
    linkPath: option('link-path',) || join(homedir(), 'Monochromatic',),
    mountPoint: option('mount-point',) || DEFAULT_MOUNT_POINT,
    probe: realProbe,
  },);
  console.log(`recorded phase ${identity.phase} in ${options.identityFile}`,);
  if (answer.ready) {
    console.log(answer.message,);
  }
  else {
    console.error(answer.message,);
    exit(EXIT_TEMPFAIL,);
  }
}
catch (error) {
  if (error instanceof RecordIdentityError) {
    console.error(error.message,);
    exit(EXIT_TEMPFAIL,);
  }
  throw error;
}
