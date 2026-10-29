import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  p,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region String shorthand

    describe({
      name: 'p (string shorthand)',
      children: [
        it({
          name:
            'sets bin, check, effname, absent available, and empty overrides from a single name',
          fn: async () => {
            const entry = p('curl',);
            expect(entry.bin,).toBe('curl',);
            expect(entry.check,).toBe('--version',);
            expect(entry.effname,).toBe('curl',);
            expect(entry.available,).toBeUndefined();
            expect(entry.overrides,).toEqual({},);
          },
        },),
        it({
          name: 'produces a frozen overrides object',
          fn: async () => {
            const entry = p('tmux',);
            expect(Object.isFrozen(entry.overrides,),).toBe(true,);
          },
        },),
      ],
    },),

    //endregion String shorthand

    //region Object spec without yes

    describe({
      name: 'p (object spec without yes)',
      children: [
        it({
          name: 'uses effname as bin when bin is omitted',
          fn: async () => {
            const entry = p({ effname: 'wget', },);
            expect(entry.bin,).toBe('wget',);
            expect(entry.effname,).toBe('wget',);
          },
        },),
        it({
          name: 'uses explicit bin when provided',
          fn: async () => {
            const entry = p({ bin: 'rg', effname: 'ripgrep', },);
            expect(entry.bin,).toBe('rg',);
            expect(entry.effname,).toBe('ripgrep',);
          },
        },),
        it({
          name: 'leaves available absent when yes is omitted',
          fn: async () => {
            const entry = p({ effname: 'wget', },);
            expect(entry.available,).toBeUndefined();
            expect(entry.overrides,).toEqual({},);
          },
        },),
        it({
          name: 'handles spec with no overrides',
          fn: async () => {
            const entry = p({ bin: 'rg', effname: 'ripgrep', },);
            expect(entry.overrides,).toEqual({},);
          },
        },),
        it({
          name: 'uses custom check flag when provided',
          fn: async () => {
            const entry = p({ bin: 'openssl', check: 'version', effname: 'openssl', },);
            expect(entry.check,).toBe('version',);
          },
        },),
        it({
          name: 'defaults check to --version when omitted',
          fn: async () => {
            const entry = p({ bin: 'rg', effname: 'ripgrep', },);
            expect(entry.check,).toBe('--version',);
          },
        },),
        it({
          name: 'produces a frozen overrides object',
          fn: async () => {
            const entry = p({ effname: 'wget', },);
            expect(Object.isFrozen(entry.overrides,),).toBe(true,);
          },
        },),
      ],
    },),

    //endregion Object spec without yes

    //region Object spec with yes

    describe({
      name: 'p (object spec with yes)',
      children: [
        it({
          name: 'builds available set from bare manager names',
          fn: async () => {
            const entry = p({ effname: 'curl', yes: ['apt', 'dnf', 'brew',], },);
            expect(entry.available,).toBeDefined();
            expect(entry.available?.has('apt',),).toBe(true,);
            expect(entry.available?.has('dnf',),).toBe(true,);
            expect(entry.available?.has('brew',),).toBe(true,);
            expect(entry.available?.has('pacman',),).toBe(false,);
          },
        },),
        it({
          name: 'extracts overrides from tuples in yes',
          fn: async () => {
            const entry = p({
              effname: 'acpica',
              yes: [
                'apt',
                ['dnf', 'acpica-tools',],
                ['pacman', 'acpica-utils',],
              ],
            },);
            expect(entry.available?.has('apt',),).toBe(true,);
            expect(entry.available?.has('dnf',),).toBe(true,);
            expect(entry.available?.has('pacman',),).toBe(true,);
            expect(entry.overrides,).toEqual({
              dnf: 'acpica-tools',
              pacman: 'acpica-utils',
            },);
          },
        },),
        it({
          name: 'bare manager names produce no overrides',
          fn: async () => {
            const entry = p({ effname: 'tmux', yes: ['apt', 'dnf',], },);
            expect(entry.overrides,).toEqual({},);
          },
        },),
        it({
          name: 'freezes available set',
          fn: async () => {
            const entry = p({ effname: 'curl', yes: ['apt',], },);
            expect(Object.isFrozen(entry.available,),).toBe(true,);
          },
        },),
        it({
          name: 'freezes overrides from yes tuples',
          fn: async () => {
            const entry = p({ effname: 'acpica', yes: [['dnf', 'acpica-tools',],], },);
            expect(Object.isFrozen(entry.overrides,),).toBe(true,);
          },
        },),
        it({
          name: 'empty yes array produces empty available set',
          fn: async () => {
            const entry = p({ effname: 'niche-tool', yes: [], },);
            expect(entry.available,).toBeDefined();
            expect(entry.available?.size,).toBe(0,);
          },
        },),
        it({
          name: 'combines bin, check, and yes',
          fn: async () => {
            const entry = p({
              bin: 'rg',
              check: '-V',
              effname: 'ripgrep',
              yes: ['apt', ['brew', 'ripgrep',],],
            },);
            expect(entry.bin,).toBe('rg',);
            expect(entry.check,).toBe('-V',);
            expect(entry.effname,).toBe('ripgrep',);
            expect(entry.available?.has('apt',),).toBe(true,);
            expect(entry.available?.has('brew',),).toBe(true,);
            expect(entry.overrides,).toEqual({ brew: 'ripgrep', },);
          },
        },),
      ],
    },),
    //endregion Object spec with yes
  ],
},);
