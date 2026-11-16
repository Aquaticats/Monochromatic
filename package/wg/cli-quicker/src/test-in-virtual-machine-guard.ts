/**
 The refusal an integration test makes when it is started outside a machine.

 The bypass and route tests create network namespaces with addresses in the
 caller's own namespace, start a route watcher as root, and change firewall and
 OpenSnitch state. On a host with a live tunnel those changes are the host's, so
 the tests refuse unless the machine runner prepared the guest: that runner
 creates the marker directory as root and nothing else does.

 @module
 */

import { statSync, } from 'node:fs';

/**
 The guest file whose presence marks a machine the runner prepared.
 */
export const GUEST_MARKER_PATH = '/run/wg-quicker-test-in-a-virtual-machine';

/**
 Reports whether an entry is at a path, without raising a failure for one that
 is not there.

 @param path - absolute path to look at

 @returns whether an entry is there
 */
function entryExists(path: string,): boolean {
  try {
    statSync(path,);
    return true;
  }
  catch {
    return false;
  }
}

/**
 Refuses unless this process runs inside a machine the runner prepared.

 @param options.name - the test's name, named in the refusal

 @param options.miseTask - the mise task that runs the test in a machine

 @throws {@link Error} naming the task that runs this test in a machine

 @example
 ```ts
 requireTestInVirtualMachine({
   miseTask: '//package/wg/cli-quicker:test:integration:bypass',
   name: 'bypass',
 });
 ```
 */
export function requireTestInVirtualMachine(
  {
    miseTask,
    name,
  }: {
    readonly miseTask: string;
    readonly name: string;
  },
): void {
  if (entryExists(GUEST_MARKER_PATH,))
    return;
  throw new Error(
    [
      `The ${name} integration test changes the network, the firewall and root-owned state of the machine it runs on, so it runs only inside a throwaway virtual machine.`,
      `Run \`mise run ${miseTask}\`, which creates that machine, copies the checkout into it, runs this test there and destroys it.`,
      `This process found no ${GUEST_MARKER_PATH}, so it is not in such a machine.`,
    ].join('\n',),
  );
}
