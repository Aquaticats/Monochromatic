/**
 Guards the path the guest drivers start the ZFS binding's own recipe from: a wrong one dies with node's `Cannot find
 module` at the driver's first step, and nothing else in the module's own tests would say so.
 */
import { existsSync, } from 'node:fs';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { guestRecipePath, } from './mvm-guest-recipe-step.ts';

/** Where the recipe sits below the package this driver belongs to, which the resolved path has to end with. */
const RECIPE_FROM_THE_PACKAGE = '/package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts';

await describe({
  name: '',
  children: [
    describe({
      name: guestRecipePath.name,
      children: [
        it({
          name: 'ends at the recipe inside the repository-shaped package folder',
          fn: async () => {
            expect(guestRecipePath(),).toContain(RECIPE_FROM_THE_PACKAGE,);
          },
        },),
        it({
          name: 'names a file that is there, which a path stopping one folder short does not',
          fn: async () => {
            expect(
              existsSync(guestRecipePath(),),
            ).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
