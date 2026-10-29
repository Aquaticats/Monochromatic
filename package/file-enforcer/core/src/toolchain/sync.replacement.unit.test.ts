/**
 Unit tests for the warning sync logs before it replaces a different installed version.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { NOT_INSTALLED, } from './status.observe.ts';
import { replacementWarning, } from './sync.ts';

await describe({
  name: replacementWarning.name,
  children: [
    it({
      name: 'names the entry, the installed version and the locked version when they differ',
      fn: async () => {
        /**
         Warning lines for lint installed at 0.4.0 under a lock that pins 0.3.0.
         */
        const warnings = replacementWarning({
          id: 'cargo:monochromatic-lint',
          observed: '0.4.0',
          locked: '0.3.0',
        },);
        expect(warnings.length,)
          .toBe(1,);
        expect(warnings[0],)
          .toContain('cargo:monochromatic-lint',);
        expect(warnings[0],)
          .toContain('installed at 0.4.0',);
        expect(warnings[0],)
          .toContain('lock pins 0.3.0',);
      },
    },),
    it({
      name: 'stays silent when the tool is absent',
      fn: async () => {
        expect(replacementWarning({
          id: 'cargo:monochromatic-lint',
          observed: NOT_INSTALLED,
          locked: '0.3.0',
        },),)
          .toEqual([],);
      },
    },),
    it({
      name: 'stays silent when the installed version matches the lock, including a v prefix',
      fn: async () => {
        expect(replacementWarning({
          id: 'cargo:monochromatic-lint',
          observed: '0.4.0',
          locked: '0.4.0',
        },),)
          .toEqual([],);
        expect(replacementWarning({
          id: 'cargo:monochromatic-lint',
          observed: 'v0.4.0',
          locked: '0.4.0',
        },),)
          .toEqual([],);
      },
    },),
  ],
},);
