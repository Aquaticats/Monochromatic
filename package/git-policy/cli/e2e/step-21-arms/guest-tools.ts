/**
 Install the Node tooling the pnpm arm needs inside the guest: the distribution's `nodejs` and `npm`, and the `pnpm` the
 repair starts from the search path.

 The guest recipe installs OpenZFS and the Rust toolchain and nothing of Node, and the pnpm arm is about the shipped
 `pnpm install --offline --frozen-lockfile`, so a real pnpm is what the arm starts rather than a stand-in. The step is
 idempotent: a second run of the driver reuses what the first installed.

 @example
 ```ts
 await installTheTools({ chosen });
 ```
 */

import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from '../../bin/mvm-guest-detached-script.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  finishMarginMilliseconds,
  toolsLog,
  toolsTimeoutMilliseconds,
} from './step-21-fixture.ts';

/**
 Install `nodejs`, `npm` and `pnpm` in the guest unless they are there.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the status the script ended with

 @example
 ```ts
 await installTheTools({ chosen });
 ```
 */
export async function installTheTools({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The guest's own install, which is one `apt-get` and one global `npm install`.
   */
  const script = [
    'set -eu',
    'export DEBIAN_FRONTEND=noninteractive',
    // The image's own unattended upgrade holds the dpkg lock for minutes after the first boot.
    'timeout 600 sh -c \'while fuser /var/lib/dpkg/lock-frontend > /dev/null 2>&1; do sleep 2; done\'',
    'if test ! -x /usr/bin/node; then apt-get --option Acquire::Retries=2 update > /root/step21-tools.log 2>&1; apt-get --option Acquire::Retries=2 --yes install nodejs npm >> /root/step21-tools.log 2>&1; fi',
    'if test ! -x /usr/local/bin/pnpm; then npm install --global pnpm >> /root/step21-tools.log 2>&1; fi',
    '/usr/bin/node --version',
    '/usr/local/bin/pnpm --version',
    'tail --lines=5 /root/step21-tools.log',
  ].join('\n',);
  await startDetachedGuestScript({
    chosen,
    script,
    log: toolsLog,
    timeoutMilliseconds: toolsTimeoutMilliseconds,
    nameCheck: 'step',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: toolsLog,
    finishTimeoutMilliseconds: toolsTimeoutMilliseconds + finishMarginMilliseconds,
  },);
}
