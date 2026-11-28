/**
 Live, billed OVHcloud provisioning check.

 Marked `.expensive.` so `test:unit` skips it unless run with `--all` or by
 explicit path. It provisions a real, billed instance, so it only runs when the
 five names are set; otherwise it reports a skip. Run it deliberately, for
 example:

 ```sh
 mise run //package/mvm/cli:test:ovh
 ```

 What it calls, in order, so a reader can see the whole bill before running it:

 1. `GET /1.0/auth/time`, once, for the clock correction every request signs with.
 2. `GET /v1/cloud/project/<project>/region`, then one `GET .../region/<name>`
    per region, to find the project's Canadian regions.
 3. `GET .../region/<name>/flavor` and `GET .../image?region=<name>` in the first
    region that has a flavor in stock, to pick the smallest one and the image.
 4. `GET .../sshkey`, and `POST .../sshkey` when this key is not there yet.
 5. `POST .../instance` with `monthlyBilling: false`, after the backend chose the
    smallest general-purpose size with at least two vCPUs and 4 GiB of memory,
    then `GET .../instance/<id>` every three seconds until the instance reports
    `ACTIVE`.
 6. `ssh` probes until the login account the image names answers, then one
    `uname -a`, then one `scp` in and one `scp` out of `/tmp/mvm-live.txt`.
 7. `DELETE .../instance/<id>`, which is what stops the meter.
 8. `GET .../instance`, and the test fails if the listing still names the
    machine this run made.

 A disposable guard destroys that one machine on scope exit, so a failed
 assertion still deletes what the run made. Nothing here runs `destroyAll` or
 asks for an empty listing: several runs test against the one project at once,
 and the other machines in it are theirs.

 @module
 */

import { randomBytes, } from 'node:crypto';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { selectBackend, } from '../../../dist/final/node/index.mjs';

//region Names

/**
 The five names this test needs, in the order the backend requires them.
 */
const REQUIRED_NAMES = [
  'OVH_ENDPOINT',
  'OVH_APPLICATION_KEY',
  'OVH_APPLICATION_SECRET',
  'OVH_CONSUMER_KEY',
  'MVM_OVH_PROJECT_ID',
];

/**
 Whether every name the backend needs is set, so the live flow may run.
 */
const HAS_NAMES = REQUIRED_NAMES.every(function isSet(name,) {
  return ((process.env[name] ?? '') !== '');
},);

/**
 Path the file round trip uses inside the instance. Any login account can write
 it, so the test does not assume a home directory.
 */
const GUEST_PATH = '/tmp/mvm-live.txt';

/**
 Fewest virtual CPUs this run asks for, which is what a guest building this
 repository's Rust code needs.
 */
const ASKED_VCPUS = 2;

/**
 Least memory this run asks for, in gibibytes.
 */
const ASKED_MEMORY_GIB = 4;

//endregion Names

await describe({
  name: 'ovh live provisioning (expensive, billed)',
  concurrency: 1,
  children: [
    it({
      name: 'provisions, execs, transfers, destroys, and leaves no machine behind (or skips without the five names)',
      fn: async () => {
        if (!HAS_NAMES) {
          // No keys: report a skip so `--all` runs without them still pass.
          expect(HAS_NAMES,).toBe(false,);
          return;
        }

        const backend = await selectBackend('ovh',);
        const suffix = randomBytes(3,)
          .toString('hex',);
        const name = `live-${suffix}`;

        // Safety net: destroy this run's own machine on scope exit, even on a thrown
        // assertion, so a failed run does not leave a billed instance. Machines of
        // other runs in the same project are left alone.
        await using _teardown = {
          async [Symbol.asyncDispose](): Promise<void> {
            /**
             Whether the project still lists the machine this run made.
             */
            const listed = (await backend.list()).some(function ownMachine(vm,) {
              return vm.name === name;
            },);
            if (listed) {
              await backend.destroy({ name, },);
            }
          },
        };

        // Create asks for a floor of two vCPUs and 4 GiB, so the cheapest size
        // that meets both wins rather than the smallest in stock. The backend
        // prints the size it picked and its hourly price before it creates.
        await backend.create({
          memoryGiB: ASKED_MEMORY_GIB,
          name,
          vcpus: ASKED_VCPUS,
        },);

        /**
         The machine as the backend lists it, which is where the size it was
         actually given can be read back.
         */
        const sized = (await backend.list())
          .find(function ours(vm,) {
            return vm.name === name;
          },);
        // Exact terminal output this run reports; no logger is configured here.
        console.log(
          `live: asked at least ${String(ASKED_VCPUS,)} vCPUs and ${String(ASKED_MEMORY_GIB,)} GiB; the backend lists ${String(sized?.vcpus,)} vCPUs and ${String(sized?.memoryGiB,)} GiB`,
        );
        expect(sized?.vcpus,).toBeGreaterThanOrEqual(ASKED_VCPUS,);
        expect(sized?.memoryGiB,).toBeGreaterThanOrEqual(ASKED_MEMORY_GIB,);

        /**
         Result of one command run inside the instance.
         */
        const execResult = await backend.exec({
          command: 'uname -a',
          name,
        },);
        expect(execResult.exitCode,).toBe(0,);
        expect(execResult.stdout,).toContain('Linux',);

        /**
         Throwaway host directory holding the file that is pushed.
         */
        const dir = await mkdtemp(join(
          scratchRoot(),
          'mvm-live-',
        ),);
        /**
         Host path of the file that is pushed and pulled back.
         */
        const localPath = join(
          dir,
          'in.txt',
        );
        /**
         Payload the round trip must return unchanged.
         */
        const payload = `payload-${suffix}`;
        await writeFile(
          localPath,
          payload,
        );
        await backend.pushFile({
          name,
          guestPath: GUEST_PATH,
          hostPath: localPath,
        },);
        /**
         Bytes read back out of the instance.
         */
        const pulled = await backend.pullFile({
          guestPath: GUEST_PATH,
          name,
        },);
        expect(pulled.toString('utf8',),).toContain(payload,);
        await rm(
          dir,
          {
            force: true,
            recursive: true,
          },
        );

        await backend.destroy({ name, },);
        /**
         What the listing still holds of this run's machine, which must be nothing.
         Machines of other runs in the project are not looked at.
         */
        const remaining = (await backend.list())
          .map(function vmName(vm,) {
            return vm.name;
          },)
          .filter(function ownMachine(listedName,) {
            return listedName === name;
          },);
        expect(remaining,).toEqual([],);
      },
      timeout: 900_000,
    },),
  ],
},);
