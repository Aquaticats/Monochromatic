import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseLauncherOptions } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  defaultGateImageTag,
  gateImageTagOf,
  gateImageTagVariable,
  launcherImageArguments,
  nativeTestImageRepository,
} from './native-gate-image-tag.ts';

/**
 Parse a launch of the git-policy product the way its mise task starts it.

 @param argv - arguments after the ones the task and the wrapper give
 @returns the options
 */
function parse(argv: readonly string[]) {
  return parseLauncherOptions({
    argv: [
      '--task',
      '//package/git-policy/cli:native:mutation:scoped',
      '--tag-env',
      gateImageTagVariable,
      ...argv,
    ],
    environment: {},
    home: '/h',
    packageDirectory: '/repo/package/mutation-test/cli-cargo-mutants',
  });
}

await describe({
  name: 'the gate image of the native wrapper',
  children: [
    describe({
      name: gateImageTagOf.name,
      children: [
        it({
          name: 'is the development tag when the variable is not set, as the gate and the controls-only task name it',
          fn: async () => {
            expect(gateImageTagOf({})).toBe(defaultGateImageTag);
            expect(defaultGateImageTag).toBe('development');
          },
        }),
        it({
          name: 'is the variable\'s value when it is set, because the gate builds the tag it names',
          fn: async () => {
            expect(gateImageTagOf({ [gateImageTagVariable]: 'coordinator-tokio-062' })).toBe('coordinator-tokio-062');
          },
        }),
      ],
    }),
    describe({
      name: launcherImageArguments.name,
      children: [
        it({
          name: 'names the gate image\'s repository and tag, which the launcher reads as the image to tag from',
          fn: async () => {
            const options = parse([
              ...launcherImageArguments({}),
              'w5',
              'a.rs',
            ]);
            expect(options.imageRepository).toBe(nativeTestImageRepository);
            expect(options.fromTag).toBe('development');
          },
        }),
        it({
          name: 'follows the variable the person set for the gate',
          fn: async () => {
            const options = parse([
              ...launcherImageArguments({ [gateImageTagVariable]: 'mine' }),
              'w5',
              'a.rs',
            ]);
            expect(options.fromTag).toBe('mine');
          },
        }),
        it({
          name: 'gives way to a --from-tag the person passes, which comes later on the command line',
          fn: async () => {
            const options = parse([
              ...launcherImageArguments({}),
              '--from-tag',
              'older-gate',
              'w5',
              'a.rs',
            ]);
            expect(options.fromTag).toBe('older-gate');
          },
        }),
      ],
    }),
  ],
});
