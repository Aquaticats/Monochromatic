/**
 Live, billed Hetzner provisioning check.
 
 Marked `.expensive.` so `test:unit` skips it unless run with `--all` or by
 explicit path. It provisions real, billed servers, so it only runs when
 `HCLOUD_TOKEN` is set; otherwise it reports a skip. Run it deliberately, for
 example:
 
 ```sh
 HCLOUD_TOKEN=... node package/mvm/cli/src/backend/hetzner/provision.expensive.unit.test.ts
 ```
 
 It provisions a throwaway server, execs a command, round-trips a file,
 clones, lists, then destroys every mvm-labelled server and confirms the
 account is clean (a safety-net disposable also tears down on failure).
 
 @module
 */

import { randomBytes, } from 'node:crypto';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { selectBackend, } from '@monochromatic-dev/mvm-cli/ts/backend/registry.ts';

/**
 Whether a token is present to run the live, billed flow.
 */
const HAS_TOKEN = ((typeof process.env.HCLOUD_TOKEN) === 'string')
  && (process.env.HCLOUD_TOKEN !== '');

await describe({
  name: 'hetzner live provisioning (expensive, billed)',
  concurrency: 1,
  children: [
    it({
      name: 'provisions, execs, transfers, clones, lists, and tears down (or skips without a token)',
      fn: async () => {
        if (!HAS_TOKEN) {
          // No token: report a skip so `--all` runs without one still pass.
          expect(HAS_TOKEN,).toBe(false,);
          return;
        }

        const backend = await selectBackend('hetzner',);
        const suffix = randomBytes(3,)
          .toString('hex',);
        const name = `live-${suffix}`;
        const cloneName = `live-${suffix}-c`;

        // Safety net: tear down every mvm-labelled server on scope exit, even on
        // a thrown assertion, so a failed run does not leak billed servers.
        await using _teardown = {
          async [Symbol.asyncDispose](): Promise<void> {
            await backend.destroyAll();
          },
        };

        await backend.create({ name, },);

        const execResult = await backend.exec({
          command: 'uname -a',
          name,
        },);
        expect(execResult.exitCode,).toBe(0,);
        expect(execResult.stdout,).toContain('Linux',);

        const dir = await mkdtemp(join(
          scratchRoot(),
          'mvm-live-',
        ),);
        const localPath = join(
          dir,
          'in.txt',
        );
        const payload = `payload-${suffix}`;
        await writeFile(
          localPath,
          payload,
        );
        await backend.pushFile({
          name,
          hostPath: localPath,
          guestPath: '/root/mvm-live.txt',
        },);
        const pulled = await backend.pullFile({
          name,
          guestPath: '/root/mvm-live.txt',
        },);
        expect(pulled.toString('utf8',),).toContain(payload,);
        await rm(
          dir,
          {
            force: true,
            recursive: true,
          },
        );

        await backend.clone({
          destination: cloneName,
          source: name,
        },);
        const names = (await backend.list()).map(function vmName(vm,) {
          return vm.name;
        },);
        expect(names,).toContain(name,);
        expect(names,).toContain(cloneName,);

        await backend.destroyAll();
        expect((await backend.list()).length,).toBe(0,);
      },
    },),
  ],
},);
