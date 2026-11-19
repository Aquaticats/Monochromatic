import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fixtureRootArguments,
  gateFixtureRoot,
  gateFixtureRootMode,
  gateFixtureRootMountOptions,
  gateFixtureRootSize,
  gateFixtureRootVariable,
} from './native-gate-fixture-root.ts';

await describe({
  name: 'the fixture root the gate\'s test container makes',
  children: [
    it({
      name: 'mounts one tmpfs at the root and points the variable both faces of the suite read at it',
      fn: async () => {
        expect(fixtureRootArguments(),).toEqual([
          '--tmpfs',
          `${gateFixtureRoot}:${gateFixtureRootMountOptions}`,
          '--env',
          `${gateFixtureRootVariable}=${gateFixtureRoot}`,
        ],);
      },
    }),
    it({
      name: 'mounts the root writable and open to every test process of the container',
      fn: async () => {
        expect(gateFixtureRootMountOptions,).toBe(`rw,size=${gateFixtureRootSize},mode=${gateFixtureRootMode}`,);
        expect(gateFixtureRootMode,).toBe('1777',);
      },
    }),
    it({
      name: 'takes the variable name the Rust fixtures read, so the mount and the tests cannot name it differently',
      fn: async () => {
        expect(gateFixtureRootVariable,).toBe('CLI_GIT_TEST_ROOT',);
      },
    }),
  ],
},);
