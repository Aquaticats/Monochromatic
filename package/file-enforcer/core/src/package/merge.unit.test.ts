import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mergeOverrides,
  p,
} from '../../dist/final/node/index.mjs';

//region mergeOverrides

await describe({
  name: mergeOverrides.name,
  children: [
    it({
      name: 'returns entries unchanged when no overrides match',
      fn: async () => {
        const generated = [p('curl',), p('tmux',),];
        const result = mergeOverrides({
          generated,
          overrideEntries: [],
        },);
        expect(result,).toEqual(generated,);
      },
    },),
    it({
      name: 'applies bin from override to matching effname',
      fn: async () => {
        const generated = [p({ effname: 'ripgrep', },),];
        const result = mergeOverrides({
          generated,
          overrideEntries: [
            p({ bin: 'rg', effname: 'ripgrep', },),
          ],
        },);
        expect(result[0]?.bin,).toBe('rg',);
        expect(result[0]?.effname,).toBe('ripgrep',);
      },
    },),
    it({
      name: 'applies check from override to matching effname',
      fn: async () => {
        const generated = [p({ effname: 'openssl', },),];
        const result = mergeOverrides({
          generated,
          overrideEntries: [
            p({ check: 'version', effname: 'openssl', },),
          ],
        },);
        expect(result[0]?.check,).toBe('version',);
      },
    },),
    it({
      name: 'applies both bin and check overrides',
      fn: async () => {
        const generated = [
          p({ effname: 'imagemagick', yes: [['dnf', 'ImageMagick',],], },),
        ];
        const result = mergeOverrides({
          generated,
          overrideEntries: [
            p({ bin: 'convert', check: '-version', effname: 'imagemagick', },),
          ],
        },);
        expect(result[0]?.bin,).toBe('convert',);
        expect(result[0]?.check,).toBe('-version',);
        expect(result[0]?.effname,).toBe('imagemagick',);
      },
    },),
    it({
      name: 'preserves manager overrides from generated entry',
      fn: async () => {
        const generated = [
          p({ effname: 'imagemagick', yes: [['dnf', 'ImageMagick',],], },),
        ];
        const result = mergeOverrides({
          generated,
          overrideEntries: [
            p({ bin: 'convert', effname: 'imagemagick', },),
          ],
        },);
        expect(result[0]?.overrides,).toEqual({ dnf: 'ImageMagick', },);
      },
    },),
    it({
      name: 'preserves available set from generated entry',
      fn: async () => {
        const generated = [
          p({ effname: 'imagemagick', yes: ['apt', ['dnf', 'ImageMagick',],], },),
        ];
        const result = mergeOverrides({
          generated,
          overrideEntries: [
            p({ bin: 'convert', effname: 'imagemagick', },),
          ],
        },);
        expect(result[0]?.available?.has('apt',),).toBe(true,);
        expect(result[0]?.available?.has('dnf',),).toBe(true,);
        expect(result[0]?.available?.has('brew',),).toBe(false,);
      },
    },),
    it({
      name: 'leaves non-matching entries untouched',
      fn: async () => {
        const generated = [p('curl',), p({ effname: 'ripgrep', },),];
        const result = mergeOverrides({
          generated,
          overrideEntries: [
            p({ bin: 'rg', effname: 'ripgrep', },),
          ],
        },);
        expect(result[0]?.bin,).toBe('curl',);
        expect(result[0]?.check,).toBe('--version',);
        expect(result[1]?.bin,).toBe('rg',);
      },
    },),
    it({
      name: 'keeps default check when override only changes bin',
      fn: async () => {
        const generated = [p({ effname: 'ripgrep', },),];
        const result = mergeOverrides({
          generated,
          overrideEntries: [
            p({ bin: 'rg', effname: 'ripgrep', },),
          ],
        },);
        expect(result[0]?.check,).toBe('--version',);
      },
    },),
  ],
},);

//endregion mergeOverrides
