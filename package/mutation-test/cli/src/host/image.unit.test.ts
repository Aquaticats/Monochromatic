import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  RUNTIME_IMAGE_BUILD_LIMITS,
  runtimeImageBuildArguments,
} from '../../dist/final/node/index.mjs';

/**
 The argument vector of a fixture image build.

 @returns the arguments after `podman`

 @example
 ```ts
 const args = fixtureArguments();
 ```
 */
function fixtureArguments(): readonly string[] {
  return runtimeImageBuildArguments({
    repoRoot: '/repo',
    baseImage: 'docker.io/cachyos/cachyos-v4:latest',
    reference: 'localhost/mutation-test-runtime:fixture',
  },);
}

await describe({
  name: runtimeImageBuildArguments.name,
  children: [
    it({
      name: 'bounds the memory and the CFS quota of the runtime image build',
      fn: async () => {
        const args = fixtureArguments();
        expect(args,).toContain(`--memory=${RUNTIME_IMAGE_BUILD_LIMITS.memory}`,);
        expect(args.some((argument,) => argument.startsWith('--cpu-period=',),),).toBe(true,);
        expect(args.some((argument,) => argument.startsWith('--cpu-quota=',),),).toBe(true,);
      },
    },),
    it({
      name: 'carries no option `podman build` lacks',
      fn: async () => {
        const args = fixtureArguments();
        expect(args.some((argument,) => argument.startsWith('--blkio-weight',),),).toBe(false,);
        expect(args.some((argument,) => argument.startsWith('--pids-limit',),),).toBe(false,);
      },
    },),
    it({
      name: 'keeps the build command, the base image argument and the context',
      fn: async () => {
        const args = fixtureArguments();
        expect(args[0],).toBe('build',);
        expect(args,).toContain('--tag',);
        expect(args.at(-1,),).toBe('/repo',);
      },
    },),
    it({
      name: 'puts every bound before the context',
      fn: async () => {
        const args = fixtureArguments();
        expect(args.findIndex((argument,) => argument.startsWith('--memory=',),),).toBeLessThan(args.length - 1,);
      },
    },),
  ],
},);
