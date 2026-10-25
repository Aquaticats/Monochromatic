/**
 Tests for the flag that can be raised and never lowered, which the Claude client keeps for a billing refusal and for
 the first rate-limit line of a run. It sat inside the client until that file passed the package's line limit, and
 these cases pin what it did there.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { oneWayFlag, } from '../dist/final/node/index.mjs';

await describe({
  name: oneWayFlag.name,
  children: [
    it({
      name: 'STARTS LOWERED and reads raised once it is raised',
      fn: async () => {
        const flag = oneWayFlag();

        expect(flag.isRaised(),).toBe(false,);
        flag.raise();
        expect(flag.isRaised(),).toBe(true,);
      },
    },),
    it({
      name: 'STAYS RAISED when it is raised again, because nothing lowers it',
      fn: async () => {
        const flag = oneWayFlag();

        flag.raise();
        flag.raise();
        expect(flag.isRaised(),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS EACH FLAG ITS OWN: raising one leaves another lowered, so one client\'s billing refusal never reaches '
        + 'another client',
      fn: async () => {
        const raised = oneWayFlag();
        const untouched = oneWayFlag();

        raised.raise();
        expect(untouched.isRaised(),).toBe(false,);
      },
    },),
  ],
},);
