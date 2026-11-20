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
      name: 'names the fixture root variable CLI_GIT_TEST_ROOT',
      fn: async () => {
        expect(gateFixtureRootVariable,).toBe('CLI_GIT_TEST_ROOT',);
      },
    }),
  ],
},);
