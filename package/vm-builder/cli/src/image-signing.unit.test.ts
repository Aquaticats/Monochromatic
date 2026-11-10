/**
 The push and cosign steps of the image release, checked against a runner that records the commands instead of running
 them: signing needs a registry login and a key this host keeps outside the tests.
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { DEV_VM_IMAGE_TAG, DEV_VM_REGISTRY_IMAGE_TAG, } from '../dist/final/node/dev-vm.mjs';
import { type StreamingCommandRunner, } from '../dist/final/node/exec.mjs';
import {
  cosignKeyPath,
  pushAndSignImage,
} from '../dist/final/node/image-signing.mjs';

//region Fixtures

/**
 One command a step ran, as the recording runner saw it.
 */
type RecordedCommand = {
  readonly cmd: string;
  readonly args: readonly string[];
};

/**
 Signing key the push cases hand the step, spelled as the entry scripts resolve it.
 */
const FIXTURE_KEY_PATH = '/repo/package/config/cosign/cosign.key';

/**
 Builds a runner that records every command and can refuse one, so the failure path runs without a registry.

 @param failOn - Predicate naming the command that must fail; none means every command succeeds

 @returns The recorded calls, filled as the step runs, and the runner to pass it
 */
function recordCommands(
  {
    failOn,
  }: {
    readonly failOn?: (command: RecordedCommand,) => boolean;
  } = {},
): {
  readonly commands: RecordedCommand[];
  readonly runner: StreamingCommandRunner;
} {
  const commands: RecordedCommand[] = [];
  const runner: StreamingCommandRunner = async ({ cmd, args = [], },) => {
    const command = {
      cmd,
      args: [...args,],
    };
    commands.push(command,);
    if (failOn?.(command,) === true) {
      throw new Error(`${cmd} refused by the recording runner`,);
    }
  };
  return {
    commands,
    runner,
  };
}

/**
 Captures the rejection reason of an operation, so a case reads the error the step let through.

 @param operation - Call expected to reject

 @returns The rejection reason, or `undefined` when the operation fulfilled
 */
async function rejectionOf(operation: Promise<unknown>,): Promise<Error | undefined> {
  try {
    await operation;
  }
  catch (error) {
    return error as Error;
  }
  return undefined;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: cosignKeyPath.name,
      children: [
        it({
          name: 'places the signing key in the config package that ships it',
          fn: async () => {
            expect(cosignKeyPath({ monorepoRoot: '/repo', },),).toBe(join(
              '/repo',
              'package',
              'config',
              'cosign',
              'cosign.key',
            ),);
          },
        },),
      ],
    },),
    describe({
      name: pushAndSignImage.name,
      children: [
        it({
          name: 'tags the local build under the registry name and pushes it',
          fn: async () => {
            const { commands, runner, } = recordCommands();
            await pushAndSignImage({
              keyPath: FIXTURE_KEY_PATH,
              runner,
            },);
            expect(commands.slice(
              0,
              2,
            ),).toStrictEqual([
              {
                cmd: 'sudo',
                args: [
                  'podman',
                  'tag',
                  DEV_VM_IMAGE_TAG,
                  DEV_VM_REGISTRY_IMAGE_TAG,
                ],
              },
              {
                cmd: 'sudo',
                args: [
                  'podman',
                  'push',
                  DEV_VM_REGISTRY_IMAGE_TAG,
                ],
              },
            ],);
          },
        },),
        it({
          name: 'signs the pushed tag with the given key and no transparency-log upload',
          fn: async () => {
            const { commands, runner, } = recordCommands();
            await pushAndSignImage({
              keyPath: FIXTURE_KEY_PATH,
              runner,
            },);
            expect(commands,).toHaveLength(3,);
            expect(commands.at(-1,),).toStrictEqual({
              cmd: 'sudo',
              args: [
                'env',
                'COSIGN_PASSWORD=',
                'cosign',
                'sign',
                '--key',
                FIXTURE_KEY_PATH,
                '--tlog-upload=false',
                DEV_VM_REGISTRY_IMAGE_TAG,
              ],
            },);
          },
        },),
        it({
          name: 'stops before signing when the push fails',
          fn: async () => {
            const { commands, runner, } = recordCommands({
              failOn: (command,) => command.args.includes('push',),
            },);
            const error = await rejectionOf(pushAndSignImage({
              keyPath: FIXTURE_KEY_PATH,
              runner,
            },),);
            expect(error?.message,).toBe('sudo refused by the recording runner',);
            expect(commands.some((command,) => command.args.includes('sign',),),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
