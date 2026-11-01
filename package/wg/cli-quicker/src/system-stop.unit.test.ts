import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  shutdownTargetMeansStopping,
  systemIsStopping,
} from '../dist/final/node/system-stop.mjs';

await describe({
  name: shutdownTargetMeansStopping.name,
  children: [
    it({
      name: 'means stopping only for an active or activating shutdown target',
      fn: async () => {
        expect(shutdownTargetMeansStopping({ printed: 'active\n', },),).toBe(true,);
        expect(shutdownTargetMeansStopping({ printed: 'activating\n', },),).toBe(true,);
        expect(shutdownTargetMeansStopping({ printed: 'inactive\n', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: 'deactivating\n', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: 'failed\n', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: '', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: 'inactive active', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: 'reactivating', },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: systemIsStopping.name,
  children: [
    it({
      name: 'follows the shutdown target alone, and judges a probe that cannot tell as a system that keeps running',
      fn: async () => {
        expect(await systemIsStopping({ readState: () => Promise.resolve('active\n',), },),).toBe(true,);
        expect(await systemIsStopping({ readState: () => Promise.resolve('activating\n',), },),).toBe(true,);
        expect(await systemIsStopping({ readState: () => Promise.resolve('inactive\n',), },),).toBe(false,);
        expect(await systemIsStopping({ readState: () => Promise.reject(new Error('systemctl exited 1',),), },),).toBe(false,);
      },
    },),
  ],
},);
