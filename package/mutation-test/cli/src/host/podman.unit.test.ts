import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildShardArgs,
  diskWeightArguments,
  LOWEST_DISK_WEIGHT,
} from '../../dist/final/node/index.mjs';

/**
 Image reference the fixture argv runs; the last podman option precedes it.
 */
const fixtureImage = 'localhost/mutation-test-runtime:fixture';

/**
 The argv of a fixture shard container.

 @returns Podman argv built from fixed fixture resources.

 @example
 ```ts
 const args = fixtureArgs();
 ```
 */
function fixtureArgs(): readonly string[] {
  return buildShardArgs({
    repoRoot: '/repo',
    image: fixtureImage,
    manifestDir: '/manifest',
    reportDir: '/report',
    resources: {
      memory: '1g',
      cpus: '1',
      pidsLimit: 1,
      sessionTimeoutSeconds: 1,
      workTmpfsSize: '1g',
    },
    selinuxRelabel: false,
  },);
}

await describe({
  name: diskWeightArguments.name,
  children: [
    it({
      name: 'sets both block-layer weight files from the one named weight',
      fn: async () => {
        expect(diskWeightArguments(),).toEqual([
          `--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,
          `--cgroup-conf=io.weight=${String(LOWEST_DISK_WEIGHT,)}`,
        ],);
      },
    },),
    describe({
      name: buildShardArgs.name,
      children: [
        it({
          name: 'gives the shard container both disk-weight options before its image',
          fn: async () => {
            /**
             Argv of the fixture container.
             */
            const args = fixtureArgs();
            /**
             Where the image sits; options after it would belong to the container command.
             */
            const imageIndex = args.indexOf(fixtureImage,);
            expect(imageIndex,).toBeGreaterThan(0,);
            for (const option of diskWeightArguments()) {
              /**
               Where the option sits; -1 means the argv lacks it.
               */
              const optionIndex = args.indexOf(option,);
              expect(optionIndex,).toBeGreaterThanOrEqual(0,);
              expect(optionIndex,).toBeLessThan(imageIndex,);
            }
          },
        },),
        it({
          name: 'passes each disk-weight option exactly once',
          fn: async () => {
            /**
             Argv of the fixture container.
             */
            const args = fixtureArgs();
            for (const option of diskWeightArguments())
              expect(args.filter((argument,) => argument === option,),).toHaveLength(1,);
          },
        },),
      ],
    },),
  ],
},);
