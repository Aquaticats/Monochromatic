/**
 Push this checkout into the guest and build the crate's test target there, so the arms compile against the tree this run
 has rather than against a copy somebody remembered to add.

 The tree is unpacked over whatever the guest already holds, so a second run reuses the build's target folder instead of
 compiling the whole crate again: a renamed source file would linger, and these arms only add files. One `rustc` for a
 crate this size wants more than five gigabytes and the guest has eight, so one job and a swap file are what make the
 build fit; the other guest drivers measure the same bound.

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */

import { createHash, } from 'node:crypto';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  SWAP_GIBIBYTES,
} from '../../bin/mvm-guest-bounds.ts';
import {
  awaitDetachedGuestScript,
} from '../../bin/mvm-guest-detached-script.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  host,
  pushWorktreeSnapshot,
} from '../../bin/mvm-guest-host-command.ts';
import { startTheScript, } from './start-the-script.ts';
import {
  buildLog,
  crateBuildTimeoutMilliseconds,
  crateDirectory,
  finishMarginMilliseconds,
  stampName,
} from './step-21-fixture.ts';

/**
 The SHA-256 of one file, in hexadecimal.

 @param path - the file to read

 @returns its digest

 @example
 ```ts
 await digestOf('/logs/subject.tar.gz');
 ```
 */
async function digestOf(path: string,): Promise<string> {
  return createHash('sha256',)
    .update(await readFile(path,),)
    .digest('hex',);
}

/**
 Write the digest beside the run's logs and push it into the guest, after the archive it names.

 @param chosen - the run's guest and log folder

 @param digest - the archive's SHA-256

 @throws GuestDriverError naming the push that refused

 @example
 ```ts
 await stampTheArchive({ chosen, digest: 'ab12' });
 ```
 */
async function stampTheArchive({
  chosen,
  digest,
}: {
  readonly chosen: GuestFacts;
  readonly digest: string;
},): Promise<void> {
  /**
   The stamp on this host, which names the archive it belongs to.
   */
  const stamp = join(
    chosen.logDir,
    stampName,
  );
  await writeFile(
    stamp,
    `${digest}  subject.tar.gz\n`,
    { mode: 0o600, },
  );
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'push',
      chosen.name,
      stamp,
      `/mnt/shared/${stampName}`,
    ],
    log: 'guest-push-step-21-archive-stamp.log',
  },);
}

/**
 Push this checkout into the guest and build the crate's test target there.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */
export async function buildInGuest({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  /**
   The digest of the archive this run pushed, which the guest waits to see before it unpacks anything: `mvm push` returns
   before the guest sees the file, and a guest that copied the previous run's archive built the previous run's code.
   */
  const digest = await digestOf(join(
    chosen.logDir,
    'subject.tar.gz',
  ),);
  /**
   The stamp that carries the digest, which is pushed after the archive so its arrival dates the archive's.
   */
  await stampTheArchive({
    chosen,
    digest,
  },);
  /**
   The guest's whole build, which unpacks the pushed archive and compiles the test target the arms run.
   */
  const script = [
    'set -eu',
    // `mvm push` lands every file in the guest's shared folder, so the copy the build unpacks is taken from there and a
    // missing one refuses.
    'test -f /mnt/shared/subject.tar.gz',
    `tries=0; until grep --quiet '${digest}' /mnt/shared/step-21-archive.sha256 2>/dev/null; do tries=$((tries+1)); test "$tries" -lt 600 || { echo 'the pushed archive never reached the guest'; exit 3; }; sleep 1; done`,
    'cp /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    // The digest is checked rather than trusted: a stale copy is what a build of the wrong tree produces, and its own log
    // then says so.
    `printf '%s  %s\n' '${digest}' /root/subject.tar.gz | sha256sum --check --strict -`,
    // The tree is unpacked over whatever the guest already holds, so a second run reuses the build's target folder.
    `mkdir --parents ${crateDirectory}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${crateDirectory}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    'free',
    // One rustc for a crate this size wants more than five gigabytes and the guest has eight, so one job and a swap file
    // are what make the build fit.
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIBIBYTES} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'CARGO_BUILD_JOBS=1 cargo build --release --tests --locked > /root/step21-build.log 2>&1',
    'tail --lines=3 /root/step21-build.log',
  ].join('\n',);
  await startTheScript({
    chosen,
    script,
    log: buildLog,
    timeoutMilliseconds: crateBuildTimeoutMilliseconds,
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: buildLog,
    finishTimeoutMilliseconds: crateBuildTimeoutMilliseconds + finishMarginMilliseconds,
  },);
}
