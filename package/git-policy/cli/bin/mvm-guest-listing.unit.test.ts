import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { holdsGuestName, } from './mvm-guest-listing.ts';

await describe({
  name: holdsGuestName.name,
  children: [
    it({
      name: 'reads a name the listing pads and follows with its state',
      fn: async () => {
        expect(holdsGuestName({
          listing: 'git-policy-lock            running\n',
          name: 'git-policy-lock',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'reads a name that is one of several guests',
      fn: async () => {
        expect(holdsGuestName({
          listing: 'another-guest   running\ngit-policy-lock running\n',
          name: 'git-policy-lock',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'reads a name of an empty listing as absent',
      fn: async () => {
        expect(holdsGuestName({
          listing: 'no VMs found\n',
          name: 'git-policy-lock',
        },),).toBe(false,);
        expect(holdsGuestName({
          listing: '',
          name: 'git-policy-lock',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'reads a name only at the start of a line, not anywhere in it',
      fn: async () => {
        expect(holdsGuestName({
          listing: 'another-guest   running\n',
          name: 'git-policy-lock',
        },),).toBe(false,);
        expect(holdsGuestName({
          listing: 'stopped git-policy-lock\n',
          name: 'git-policy-lock',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'reads the prefix of another guest as that name too, which refuses rather than risks a guest',
      fn: async () => {
        expect(holdsGuestName({
          listing: 'git-policy-lock-2   running\n',
          name: 'git-policy-lock',
        },),).toBe(true,);
      },
    },),
  ],
},);
