import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { LOWEST_DISK_WEIGHT, } from '@monochromatic-dev/container-limit-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bootcConverterArguments,
  IMAGE_BUILD_LIMITS,
  imageBuildArguments,
} from '../dist/final/node/container-arguments.mjs';

/**
 Reads this package's `mise.toml`, whose `image:build` and `image:check` commands are shell strings and so cannot import
 the constant.

 @returns File text.

 @example
 ```ts
 const text = await miseToml();
 ```
 */
async function miseToml(): Promise<string> {
  return await readFile(
    join(
      import.meta.dirname,
      '..',
      'mise.toml',
    ),
    'utf8',
  );
}

/**
 Inputs of the converter container, spelled as fixture paths.
 */
const fixtureConverter = {
  outputDirectory: '/out',
  diskConfigDirectory: '/config-dir',
  imageTag: 'localhost/fixture:latest',
};

await describe({
  name: 'the rootful containers of the build pipeline',
  children: [
    describe({
      name: bootcConverterArguments.name,
      children: [
        it({
          name: 'puts the privileged converter at the lowest disk weight',
          fn: async () => {
            const args = bootcConverterArguments(fixtureConverter,);
            expect(args,).toContain(`--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,);
            expect(args,).toContain(`--cgroup-conf=io.weight=${String(LOWEST_DISK_WEIGHT,)}`,);
          },
        },),
        it({
          name: 'gives the privileged converter CPU, memory and process limits',
          fn: async () => {
            const args = bootcConverterArguments(fixtureConverter,);
            expect(args.some((argument,) => argument.startsWith('--cpus=',),),).toBe(true,);
            expect(args.some((argument,) => argument.startsWith('--memory=',),),).toBe(true,);
            expect(args.some((argument,) => argument.startsWith('--pids-limit=',),),).toBe(true,);
          },
        },),
        it({
          name: 'keeps every limit before the image the converter runs',
          fn: async () => {
            const args = bootcConverterArguments(fixtureConverter,);
            const converterImage = args.findIndex((argument,) => argument.includes('bootc-image-builder',),);
            expect(converterImage,).toBeGreaterThan(0,);
            for (const flag of [
              '--cpus=',
              '--memory=',
              '--pids-limit=',
              '--blkio-weight=',
              '--cgroup-conf=',
            ]) {
              const position = args.findIndex((argument,) => argument.startsWith(flag,),);
              expect(position,).toBeGreaterThan(-1,);
              expect(position,).toBeLessThan(converterImage,);
            }
          },
        },),
        it({
          name: 'still runs privileged with the bootc-image-builder image and the disk config',
          fn: async () => {
            const args = bootcConverterArguments(fixtureConverter,);
            expect(args,).toContain('--privileged',);
            expect(args,).toContain('/config/disk.toml',);
            expect(args.at(-1,),).toBe(fixtureConverter.imageTag,);
          },
        },),
      ],
    },),
    describe({
      name: imageBuildArguments.name,
      children: [
        it({
          name: 'bounds the rootful image build with memory and the CFS quota',
          fn: async () => {
            const args = imageBuildArguments({
              imageTag: fixtureConverter.imageTag,
              containerfile: '/p/Containerfile',
              ignorefile: '/p/.containerignore',
              contextDirectory: '/repo',
            },);
            expect(args.slice(0, 2,),).toEqual(['podman', 'build',],);
            expect(args,).toContain(`--memory=${IMAGE_BUILD_LIMITS.memory}`,);
            expect(args.some((argument,) => argument.startsWith('--cpu-quota=',),),).toBe(true,);
            expect(args.at(-1,),).toBe('/repo',);
          },
        },),
        it({
          name: 'carries no option `podman build` lacks',
          fn: async () => {
            const args = imageBuildArguments({
              imageTag: fixtureConverter.imageTag,
              containerfile: '/p/Containerfile',
              ignorefile: '/p/.containerignore',
              contextDirectory: '/repo',
            },);
            expect(args.some((argument,) => argument.startsWith('--blkio-weight',),),).toBe(false,);
            expect(args.some((argument,) => argument.startsWith('--pids-limit',),),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'mise.toml',
      children: [
        it({
          name: 'matches the lowest_disk_weight variable to the shared constant',
          fn: async () => {
            /**
             Text before the value in the variable's line.
             */
            const prefix = 'lowest_disk_weight = "';
            /**
             The variable's line as written in the file.
             */
            const line = (await miseToml())
              .split('\n',)
              .find((candidate,) => candidate.startsWith(prefix,),);
            expect(line,).toBe(`${prefix}${String(LOWEST_DISK_WEIGHT,)}"`,);
          },
        },),
        it({
          name: 'gives every podman run of mise.toml both options from that variable',
          fn: async () => {
            /**
             Lines of mise.toml that start a container.
             */
            const runs = (await miseToml())
              .split('\n',)
              .filter((line,) => line.includes('podman run',),);
            expect(runs.length,).toBeGreaterThan(0,);
            for (const line of runs) {
              expect(line,).toContain('--blkio-weight {{vars.lowest_disk_weight}}',);
              expect(line,).toContain('--cgroup-conf io.weight={{vars.lowest_disk_weight}}',);
            }
          },
        },),
        it({
          name: 'gives the image build the same memory and CPU bound as the build helper',
          fn: async () => {
            const build = (await miseToml())
              .split('\n',)
              .find((line,) => line.includes('podman build',),);
            expect(build,).toContain(`--memory ${IMAGE_BUILD_LIMITS.memory}`,);
          },
        },),
      ],
    },),
  ],
},);
