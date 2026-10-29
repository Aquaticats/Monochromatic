import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CircularCssMixinError,
  expandCssMixins,
  UnknownCssMixinError,
} from '@monochromatic-dev/css-cli';

await describe({
  name: expandCssMixins.name,
  children: [
    //region Expansion

    it({
      name: 'expands a referenced mixin from shared mixin CSS',
      fn: async () => {
        const result = expandCssMixins({
          css: '.card { @apply --surface; }',
          mixinCss: '@mixin --surface { padding: 1rem; }',
        },);

        expect(result,).toContain('padding: 1rem',);
        expect(result,).not.toContain('@apply',);
        expect(result,).not.toContain('@mixin',);
      },
    },),

    it({
      name: 'collects and strips inline definitions from the consumer CSS',
      fn: async () => {
        const result = expandCssMixins({
          css: '@mixin --local { top: 0; }\n.a { @apply --local; }',
        },);

        expect(result,).toContain('top: 0',);
        expect(result,).not.toContain('@mixin',);
        expect(result,).not.toContain('@apply',);
      },
    },),

    it({
      name: 'lets inline definitions override shared ones on name collision',
      fn: async () => {
        const result = expandCssMixins({
          css: '@mixin --pad { padding: 2rem; }\n.a { @apply --pad; }',
          mixinCss: '@mixin --pad { padding: 1rem; }',
        },);

        expect(result,).toContain('padding: 2rem',);
        expect(result,).not.toContain('padding: 1rem',);
      },
    },),

    it({
      name: 'expands nested mixin references between definitions',
      fn: async () => {
        const result = expandCssMixins({
          css: '.btn { @apply --card; }',
          mixinCss: `
            @mixin --flex-center { display: flex; align-items: center; }
            @mixin --card { @apply --flex-center; padding: 1rem; }
          `,
        },);

        expect(result,).toContain('display: flex',);
        expect(result,).toContain('padding: 1rem',);
        expect(result,).not.toContain('@apply',);
      },
    },),

    it({
      name: 'expands deeply chained references',
      fn: async () => {
        const result = expandCssMixins({
          css: '.x { @apply --c; }',
          mixinCss: `
            @mixin --a { color: red; }
            @mixin --b { @apply --a; margin: 0; }
            @mixin --c { @apply --b; padding: 0; }
          `,
        },);

        expect(result,).toContain('color: red',);
        expect(result,).toContain('margin: 0',);
        expect(result,).toContain('padding: 0',);
      },
    },),

    it({
      name: 'preserves untouched CSS byte-exactly, comments included',
      fn: async () => {
        const result = expandCssMixins({
          css: '/* header */\n.a {\n  color: red; /* keep */\n}\n',
        },);

        expect(result,).toBe('/* header */\n.a {\n  color: red; /* keep */\n}\n',);
      },
    },),

    it({
      name: 'expands mixin bodies containing nested rules',
      fn: async () => {
        const result = expandCssMixins({
          css: ':host { @apply --globals; }',
          mixinCss: '@mixin --globals { button:disabled { opacity: 0.45; } }',
        },);

        expect(result,).toContain('button:disabled',);
        expect(result,).toContain('opacity: 0.45',);
      },
    },),

    //endregion Expansion

    //region Errors

    it({
      name: 'throws UnknownCssMixinError for an unregistered reference',
      fn: async () => {
        let caught: unknown;
        try {
          expandCssMixins({
            css: '.card { @apply --missing; }',
            mixinCss: '@mixin --surface { padding: 1rem; }',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(UnknownCssMixinError,);
        expect((caught as UnknownCssMixinError).message,).toContain('Unknown mixin: --missing',);
        expect((caught as UnknownCssMixinError).mixinName,).toBe('--missing',);
      },
    },),

    it({
      name: 'throws for an unregistered reference nested inside a definition',
      fn: async () => {
        expect(function expandNestedUnknown() {
          expandCssMixins({
            css: '.a { @apply --outer; }',
            mixinCss: '@mixin --outer { @apply --nope; }',
          },);
        },).toThrow('Unknown mixin referenced in nested @apply: --nope',);
      },
    },),

    it({
      name: 'throws CircularCssMixinError naming the exact cycle',
      fn: async () => {
        let caught: unknown;
        try {
          expandCssMixins({
            css: '.a { @apply --a; }',
            mixinCss: `
              @mixin --a { @apply --b; }
              @mixin --b { @apply --a; }
            `,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(CircularCssMixinError,);
        expect((caught as CircularCssMixinError).message,).toContain('--a -> --b -> --a',);
      },
    },),

    it({
      name: 'throws on a nameless definition',
      fn: async () => {
        expect(function expandNameless() {
          expandCssMixins({ css: '@mixin { top: 0; }', },);
        },).toThrow('@mixin requires a name',);
      },
    },),

    it({
      name: 'throws on a bodyless definition',
      fn: async () => {
        expect(function expandBodyless() {
          expandCssMixins({ css: '.btn { @mixin --touch-target; }', },);
        },).toThrow('mixin definition must include body',);
      },
    },),

    it({
      name: 'throws on a whitespace-only definition body',
      fn: async () => {
        expect(function expandEmptyBody() {
          expandCssMixins({ css: '@mixin --empty {  }', },);
        },).toThrow('mixin definition must include body',);
      },
    },),

    it({
      name: 'throws on an @apply without a name',
      fn: async () => {
        expect(function expandNamelessApply() {
          expandCssMixins({ css: '.box { @apply ; }', },);
        },).toThrow('Mixin name is required',);
      },
    },),

    //endregion Errors
  ],
},);
