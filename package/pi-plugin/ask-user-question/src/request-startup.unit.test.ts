import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runStartupFixture, } from './request-startup-fixture.ts';

await describe({
  name: 'detached helper startup',
  children: ['normal', 'quoted-launch-paths', 'detached-start', 'helper-removed-after-launch', 'runtime-removed', 'helper-missing-before-launch',].map(function startupScenario(scenario,) {
    return it({
      name: scenario,
      skip: ((scenario === 'runtime-removed') && (process.platform !== 'linux'))
        || ((scenario === 'quoted-launch-paths') && (process.platform === 'win32')),
      fn: async () => {
        expect(await runStartupFixture(scenario,),).toContain(`verified ${scenario}`,);
      },
    },);
  },),
},);
