/**
 Tests that every container the matrix runner starts and every image it builds carries limits.
 A recording engine stands in for `podman` when images are built, and the run arguments are read without starting a
 container.

 @module
 */

import { diskWeightArguments, } from '@monochromatic-dev/container-limit-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildContainerImage,
  containerRunArguments,
  type ContainerEngine,
  type ContainerImageInputs,
} from '../dist/final/node/index.mjs';

/**
 Inputs of one fixture build with no bounding flags of its own, so the default bound shows.
 */
const UNBOUNDED_INPUTS: ContainerImageInputs = {
  image: 'localhost/fixture/image',
  containerfile: 'Containerfile',
  contextDirectory: '.',
  buildArguments: ['--build-arg', 'ZIG_ARCHIVE_SHA256=first',],
  boundingFlags: [],
  baseImages: [],
};

/**
 Runs one fixture build against a recording engine.

 @param inputs - what the image is built from

 @returns arguments of the one build the engine ran

 @example
 ```ts
 const args = await buildArgumentsOf(UNBOUNDED_INPUTS);
 ```
 */
async function buildArgumentsOf(inputs: ContainerImageInputs,): Promise<readonly string[]> {
  /**
   Arguments of every build the engine ran.
   */
  const builds: (readonly string[])[] = [];
  /**
   Engine that records a build and has no local images.
   */
  const engine: ContainerEngine = {
    hasImage: async () => false,
    idOf: async () => '',
    labelOf: async () => '',
    build: async (buildCommandArguments,) => {
      builds.push(buildCommandArguments,);
    },
  };
  await buildContainerImage({
    inputs,
    engine,
  },);
  expect(builds.length,).toBe(1,);
  return builds[0] ?? [];
}

await describe({
  name: 'container limits of the matrix runner',
  children: [
    describe({
      name: containerRunArguments.name,
      children: [
        it({
          name: 'puts every cell at the lowest disk weight in both forms',
          fn: async () => {
            const args = containerRunArguments({
              image: 'docker.io/library/ubuntu:latest',
              monorepoRoot: '/repo',
              command: 'true',
            },);
            for (const option of diskWeightArguments()) {
              expect(args,).toContain(option,);
            }
          },
        },),
        it({
          name: 'keeps the 2 GiB and 2 CPU bound and adds a process limit',
          fn: async () => {
            const args = containerRunArguments({
              image: 'docker.io/library/ubuntu:latest',
              monorepoRoot: '/repo',
              command: 'true',
            },);
            expect(args,).toContain('--memory=2g',);
            expect(args,).toContain('--cpus=2',);
            expect(args.some((argument,) => argument.startsWith('--pids-limit=',),),).toBe(true,);
          },
        },),
        it({
          name: 'mounts the shared pnpm store read-only at the path the checkout links name',
          fn: async () => {
            const args = containerRunArguments({
              image: 'docker.io/library/ubuntu:latest',
              monorepoRoot: '/repo',
              command: 'true',
            },);
            /**
             Mount argument that carries the store, found by its read-only suffix.
             */
            const store = args.find((argument,) => argument.endsWith(':ro',) && argument.includes('/pnpm/store:',),);
            expect(store,).toBeDefined();
            /**
             Where the mount reads from, and where the cell sees it.
             */
            const [source, destination,] = (store ?? '').split(':',);
            expect(source,).toBe(destination,);
          },
        },),
        it({
          name: 'keeps SELinux labels off the cell, so neither the shared store nor the checkout is relabelled',
          fn: async () => {
            const args = containerRunArguments({
              image: 'docker.io/library/ubuntu:latest',
              monorepoRoot: '/repo',
              command: 'true',
            },);
            /**
             Position of the label option.
             */
            const label = args.indexOf('--security-opt',);
            expect(label,).toBeGreaterThan(-1,);
            expect(args[label + 1],).toBe('label=disable',);
            expect(args.some((argument,) => argument.endsWith(',z',) || argument.endsWith(',Z',),),).toBe(false,);
          },
        },),
        it({
          name: 'puts the limits before the image and ends with the shell command',
          fn: async () => {
            const args = containerRunArguments({
              image: 'docker.io/library/ubuntu:latest',
              monorepoRoot: '/repo',
              command: 'true',
            },);
            /**
             Where the image is.
             */
            const image = args.indexOf('docker.io/library/ubuntu:latest',);
            expect(image,).toBeGreaterThan(-1,);
            for (const flag of [
              '--memory=',
              '--cpus=',
              '--pids-limit=',
              '--blkio-weight=',
              '--cgroup-conf=',
            ]) {
              expect(args.findIndex((argument,) => argument.startsWith(flag,),),).toBeLessThan(image,);
            }
            expect(args.slice(-3,),).toEqual(['sh', '-c', 'true',],);
          },
        },),
      ],
    },),
    describe({
      name: buildContainerImage.name,
      children: [
        it({
          name: 'bounds the memory and CPUs of a build whose caller gave no bounding flags',
          fn: async () => {
            const args = await buildArgumentsOf(UNBOUNDED_INPUTS,);
            expect(args.some((argument,) => argument.startsWith('--memory=',),),).toBe(true,);
            expect(args.some((argument,) => argument.startsWith('--cpu-quota=',),),).toBe(true,);
          },
        },),
        it({
          name: 'leaves a caller that bounds the build itself alone',
          fn: async () => {
            const args = await buildArgumentsOf({
              ...UNBOUNDED_INPUTS,
              boundingFlags: ['--memory=1g',],
            },);
            expect(args.filter((argument,) => argument.startsWith('--memory=',),),).toEqual(['--memory=1g',],);
            expect(args.some((argument,) => argument.startsWith('--cpu-quota=',),),).toBe(false,);
          },
        },),
        it({
          name: 'carries no option podman build lacks',
          fn: async () => {
            const args = await buildArgumentsOf(UNBOUNDED_INPUTS,);
            expect(args.some((argument,) => argument.startsWith('--blkio-weight',),),).toBe(false,);
            expect(args.some((argument,) => argument.startsWith('--pids-limit',),),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
