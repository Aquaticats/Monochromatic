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
 5. `POST .../instance` with `monthlyBilling: false`, then `GET .../instance/<id>`
    every three seconds until the instance reports `ACTIVE`.
 6. `ssh` probes until the login account the image names answers, then one
    `uname -a`, then one `scp` in and one `scp` out of `/tmp/mvm-live.txt`.
 7. `DELETE .../instance/<id>`, which is what stops the meter.
 8. `GET .../instance`, and the test fails if any instance still carries mvm's
    name prefix.

 A disposable guard runs `destroyAll` on scope exit, so a failed assertion still
 deletes what the run made.

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

        // Safety net: tear down every mvm-owned instance on scope exit, even on a
        // thrown assertion, so a failed run does not leave a billed instance.
        await using _teardown = {
          async [Symbol.asyncDispose](): Promise<void> {
            await backend.destroyAll();
          },
        };

        // Create makes the smallest flavor the first Canadian region has in stock.
        await backend.create({ name, },);

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
         Instances still carrying the tool's prefix, which must be none.
         */
        const remaining = (await backend.list()).map(function vmName(vm,) {
          return vm.name;
        },);
        expect(remaining,).toEqual([],);
      },
      timeout: 900_000,
    },),
  ],
},);
