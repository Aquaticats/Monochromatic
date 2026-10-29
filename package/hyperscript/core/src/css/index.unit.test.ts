import {
  cssCalc,
  cssClamp,
  cssCompounded,
  type CssDeclarations,
  cssMax,
  cssMin,
  cssNum,
  cssOklch,
  cssPercent,
  cssRandom,
  cssRem,
  type CssValue,
  cssVar,
  cssVi,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

await describe({
  name: $.name,
  children: [
    //region Style rules

    it({
      name: 'creates a simple rule with declarations',
      fn: async () => {
        expect($({ rule: '.card', decls: { display: 'flex', gap: cssRem(1,), }, },),)
          .toBe(
          '.card{display:flex;gap:1rem}',
          );
      },
    },),
    it({
      name: 'creates a rule with no declarations',
      fn: async () => {
        expect($({ rule: '.empty', },),).toBe('.empty{}',);
      },
    },),
    it({
      name: 'creates a rule with a single declaration',
      fn: async () => {
        expect($({ rule: 'body', decls: { margin: '0', }, },),).toBe('body{margin:0}',);
      },
    },),
    it({
      name: 'preserves CSS custom property names',
      fn: async () => {
        expect($({ rule: ':root', decls: { '--color-fg': 'oklch(0.2 0 0)', }, },),).toBe(
          ':root{--color-fg:oklch(0.2 0 0)}',
        );
      },
    },),
    it({
      name: 'handles complex selectors',
      fn: async () => {
        expect(
          $({ rule: '.card > .title:first-child', decls: { 'font-weight': 'bold', }, },),
        )
          .toBe(
          '.card > .title:first-child{font-weight:bold}',
          );
      },
    },),

    //endregion

    //region Nested rules

    it({
      name: 'nests child rules inside a parent',
      fn: async () => {
        expect($({
          rule: '.card',
          decls: { display: 'flex', },
          children: [
            $({ rule: '&:hover', decls: { opacity: cssNum(0.8,), }, },),
          ],
        },),)
          .toBe('.card{display:flex;&:hover{opacity:0.8}}',);
      },
    },),
    it({
      name: 'nests multiple children',
      fn: async () => {
        expect($({
          rule: '.btn',
          decls: { padding: '0.5rem' as CssValue, },
          children: [
            $({ rule: '&:hover', decls: {
              'background-color': cssOklch({ l: 0.45, c: 0.31, h: 264, },),
            }, },),
            $({ rule: '&:focus-visible',
              decls: { outline: '2px solid' as CssValue, }, },),
          ],
        },),)
          .toBe(
          '.btn{padding:0.5rem;&:hover{background-color:oklch(0.45 0.31 264)}&:focus-visible{outline:2px solid}}',
          );
      },
    },),
    it({
      name: 'nests children without parent declarations',
      fn: async () => {
        expect($({
          rule: '.wrapper',
          children: [
            $({ rule: '& > *', decls: { flex: '1', }, },),
          ],
        },),)
          .toBe('.wrapper{& > *{flex:1}}',);
      },
    },),
    it({
      name: 'deeply nests rules',
      fn: async () => {
        expect($({
          rule: '.a',
          children: [
            $({
              rule: '& .b',
              children: [
                $({ rule: '& .c', decls: { color: cssVar('fg',), }, },),
              ],
            },),
          ],
        },),)
          .toBe('.a{& .b{& .c{color:var(--fg)}}}',);
      },
    },),

    //endregion

    //region At-rules with children

    it({
      name: 'creates a media query with child rules',
      fn: async () => {
        expect($({
          at: 'media',
          params: '(prefers-color-scheme: dark)',
          children: [
            $({ rule: ':root', decls: { '--color-fg': 'oklch(0.9 0 0)', }, },),
          ],
        },),)
          .toBe('@media (prefers-color-scheme: dark){:root{--color-fg:oklch(0.9 0 0)}}',);
      },
    },),
    it({
      name: 'creates a layer block with rules',
      fn: async () => {
        expect($({
          at: 'layer',
          params: 'components',
          children: [
            $({ rule: '.btn', decls: { display: 'inline-flex', }, },),
          ],
        },),)
          .toBe('@layer components{.btn{display:inline-flex}}',);
      },
    },),
    it({
      name: 'creates a scope block',
      fn: async () => {
        expect($({
          at: 'scope',
          params: '(.card)',
          children: [
            $({ rule: '.title', decls: { 'font-size': cssRem(1.5,), }, },),
          ],
        },),)
          .toBe('@scope (.card){.title{font-size:1.5rem}}',);
      },
    },),
    it({
      name: 'nests at-rules inside at-rules',
      fn: async () => {
        expect($({
          at: 'layer',
          params: 'theme',
          children: [
            $({
              at: 'media',
              params: '(prefers-color-scheme: dark)',
              children: [
                $({ rule: ':root', decls: { '--bg': 'black', }, },),
              ],
            },),
          ],
        },),)
          .toBe('@layer theme{@media (prefers-color-scheme: dark){:root{--bg:black}}}',);
      },
    },),

    //endregion

    //region At-rules with declarations

    it({
      name: 'creates a @property rule',
      fn: async () => {
        expect($({
          at: 'property',
          params: '--color-fg',
          decls: {
            syntax: '"<color>"',
            inherits: 'true',
            'initial-value': 'black',
          },
        },),)
          .toBe(
          '@property --color-fg{syntax:"<color>";inherits:true;initial-value:black}',
          );
      },
    },),
    it({
      name: 'creates a @font-face rule',
      fn: async () => {
        expect($({
          at: 'font-face',
          decls: {
            'font-family': '"Aquaticat"',
            src: 'url("aquaticat.woff2") format("woff2")',
            'font-display': 'swap',
          },
        },),)
          .toBe(
          '@font-face{font-family:"Aquaticat";src:url("aquaticat.woff2") format("woff2");font-display:swap}',
          );
      },
    },),
    it({
      name: 'creates a @counter-style rule',
      fn: async () => {
        expect($({
          at: 'counter-style',
          params: 'dash',
          decls: { system: 'cyclic', symbols: '"–"', suffix: '" "', },
        },),)
          .toBe('@counter-style dash{system:cyclic;symbols:"–";suffix:" "}',);
      },
    },),

    //endregion

    //region Statement at-rules

    it({
      name: 'creates a @layer statement',
      fn: async () => {
        expect($({ at: 'layer', params: 'tokens, base, components', },),).toBe(
          '@layer tokens, base, components;',
        );
      },
    },),
    it({
      name: 'creates a @charset statement',
      fn: async () => {
        expect($({ at: 'charset', params: '"UTF-8"', },),).toBe('@charset "UTF-8";',);
      },
    },),
    it({
      name: 'creates an @import statement',
      fn: async () => {
        expect($({ at: 'import', params: 'url("reset.css")', },),).toBe(
          '@import url("reset.css");',
        );
      },
    },),
    it({
      name: 'creates an @import with layer',
      fn: async () => {
        expect($({ at: 'import', params: 'url("theme.css") layer(theme)', },),).toBe(
          '@import url("theme.css") layer(theme);',
        );
      },
    },),
    it({
      name: 'creates a @namespace statement',
      fn: async () => {
        expect($({ at: 'namespace', params: 'url("http://www.w3.org/1999/xhtml")', },),)
          .toBe(
          '@namespace url("http://www.w3.org/1999/xhtml");',
          );
      },
    },),
    it({
      name: 'creates an at-rule with no params',
      fn: async () => {
        expect($({ at: 'layer', },),).toBe('@layer;',);
      },
    },),

    //endregion

    //region Raw content

    it({
      name: 'injects raw CSS inside a rule',
      fn: async () => {
        expect($({ rule: '.card', raw: 'display:flex;gap:1rem', },),).toBe(
          '.card{display:flex;gap:1rem}',
        );
      },
    },),
    it({
      name: 'injects raw CSS inside an at-rule',
      fn: async () => {
        expect($({
          at: 'media',
          params: '(width >= 768px)',
          raw: '.sidebar{display:block}',
        },),)
          .toBe('@media (width >= 768px){.sidebar{display:block}}',);
      },
    },),
    it({
      name: 'separates decls from following raw with semicolon',
      fn: async () => {
        expect($({
          rule: '.card',
          decls: { display: 'flex', },
          raw: 'background-image:url(a)',
        },),)
          .toBe('.card{display:flex;background-image:url(a)}',);
      },
    },),
    it({
      name: 'separates raw from following children with semicolon',
      fn: async () => {
        expect($({
          rule: '.card',
          raw: 'display:flex',
          children: [
            $({ rule: '&:hover', decls: { opacity: cssNum(0.8,), }, },),
          ],
        },),)
          .toBe('.card{display:flex;&:hover{opacity:0.8}}',);
      },
    },),
    it({
      name: 'chains decls, raw, and children with proper separators',
      fn: async () => {
        expect($({
          rule: '.card',
          decls: { display: 'flex', },
          raw: 'background-image:url(a)',
          children: [
            $({ rule: '&:hover', decls: { opacity: cssNum(0.8,), }, },),
          ],
        },),)
          .toBe(
          '.card{display:flex;background-image:url(a);&:hover{opacity:0.8}}',
          );
      },
    },),

    //endregion

    //region Empty segments do not emit stray separators

    it({
      name: 'omits separator when decls is empty and children follow',
      fn: async () => {
        expect($({
          rule: '.card',
          decls: {},
          children: [$({ rule: '&:hover', decls: { opacity: cssNum(1,), }, },),],
        },),)
          .toBe('.card{&:hover{opacity:1}}',);
      },
    },),
    it({
      name: 'omits separator when raw is empty and children follow',
      fn: async () => {
        expect($({
          rule: '.card',
          raw: '',
          children: [$({ rule: '&:hover', decls: { opacity: cssNum(1,), }, },),],
        },),)
          .toBe('.card{&:hover{opacity:1}}',);
      },
    },),
    it({
      name: 'concatenates duplicate child strings without stray separators',
      fn: async () => {
        const child = $({ rule: '&:focus', decls: { color: cssVar('fg',), }, },);
        expect($({
          rule: '.btn',
          decls: { display: 'flex', },
          children: [child, child,],
        },),)
          .toBe(
          '.btn{display:flex;&:focus{color:var(--fg)}&:focus{color:var(--fg)}}',
          );
      },
    },),

    //endregion

    //region Math function constructors

    it({
      name: 'cssMin joins values with comma-space',
      fn: async () => {
        expect(cssMin([cssRem(20,), cssPercent(100,),],),).toBe('min(20rem, 100%)',);
      },
    },),
    it({
      name: 'cssMax joins values with comma-space',
      fn: async () => {
        expect(cssMax([cssRem(1,), cssPercent(10,),],),).toBe('max(1rem, 10%)',);
      },
    },),
    it({
      name: 'cssMax accepts a calc() expression as a value',
      fn: async () => {
        expect(cssMax(['100cqi', cssCalc('100% - 2rem',),],),).toBe(
          'max(100cqi, calc(100% - 2rem))',
        );
      },
    },),
    it({
      name: 'cssClamp emits min, ideal, max in order',
      fn: async () => {
        expect(
          cssClamp({ min: cssRem(1,), ideal: cssVi(2,), max: cssRem(2,), },),
        )
          .toBe('clamp(1rem, 2vi, 2rem)',);
      },
    },),
    it({
      name: 'cssClamp accepts mixed string and branded values',
      fn: async () => {
        expect(
          cssClamp({
            min: '0',
            ideal: cssCalc('100% - 2rem',),
            max: cssPercent(100,),
          },),
        )
          .toBe('clamp(0, calc(100% - 2rem), 100%)',);
      },
    },),
    it({
      name: 'cssRandom emits min and max as a two-argument call',
      fn: async () => {
        expect(cssRandom({ min: 0, max: 1, },),).toBe('random(0, 1)',);
      },
    },),
    it({
      name: 'cssRandom appends step as a bare positional fourth argument',
      fn: async () => {
        expect(
          cssRandom({ min: 1, max: 1_000, step: 1, },),
        )
          .toBe('random(1, 1000, 1)',);
      },
    },),
    it({
      name: 'cssRandom prepends a random-key when one is supplied',
      fn: async () => {
        expect(
          cssRandom({ key: '--ticker-seed', min: 0, max: 8, step: 1, },),
        )
          .toBe('random(--ticker-seed, 0, 8, 1)',);
      },
    },),
    it({
      name: 'cssRandom omits step when a key is supplied without one',
      fn: async () => {
        expect(
          cssRandom({ key: 'element-scoped', min: 0, max: 1, },),
        )
          .toBe('random(element-scoped, 0, 1)',);
      },
    },),

    //endregion

    //region Composition patterns (mixin replacement)

    it({
      name: 'composes declarations via object spread',
      fn: async () => {
        function flexCenter(): CssDeclarations {
          return { display: 'flex', 'align-items': 'center',
            'justify-content': 'center', };
        }

        expect($({ rule: '.hero', decls: { ...flexCenter(), gap: cssRem(2,), }, },),)
          .toBe(
          '.hero{display:flex;align-items:center;justify-content:center;gap:2rem}',
          );
      },
    },),
    it({
      name: 'composes parameterized declarations',
      fn: async () => {
        function touchTarget(
          { size = '48px' as CssValue, }: { size?: CssValue; } = {},
        ): CssDeclarations {
          return { 'min-inline-size': size, 'min-block-size': size, };
        }

        expect(
          $({ rule: '.btn',
            decls: { ...touchTarget({ size: '44px' as CssValue, },),
              cursor: 'pointer', }, },),
        )
          .toBe(
          '.btn{min-inline-size:44px;min-block-size:44px;cursor:pointer}',
          );
      },
    },),

    //endregion

    //region Real-world patterns from existing CSS

    it({
      name: 'generates a design token layer',
      fn: async () => {
        const result = $({
          at: 'layer',
          params: 'tokens',
          children: [
            $({
              rule: ':root',
              decls: {
                '--color-fg': 'oklch(0.2 0 0)',
                '--color-bg': 'oklch(0.98 0 0)',
                '--gap-md': '1rem',
              },
            },),
            $({
              at: 'media',
              params: '(prefers-color-scheme: dark)',
              children: [
                $({
                  rule: ':root',
                  decls: {
                    '--color-fg': 'oklch(0.9 0 0)',
                    '--color-bg': 'oklch(0.1 0 0)',
                  },
                },),
              ],
            },),
          ],
        },);
        expect(result,).toBe(
          '@layer tokens{:root{--color-fg:oklch(0.2 0 0);--color-bg:oklch(0.98 0 0);--gap-md:1rem}@media (prefers-color-scheme: dark){:root{--color-fg:oklch(0.9 0 0);--color-bg:oklch(0.1 0 0)}}}',
        );
      },
    },),
    it({
      name: 'generates a component with nested hover and focus',
      fn: async () => {
        const result = $({
          rule: '.card',
          decls: {
            display: 'flex',
            'flex-direction': 'column',
            gap: cssVar('gap-md',),
            'border-radius': cssRem(0.5,),
          },
          children: [
            $({ rule: '&:hover', decls: {
              'box-shadow': cssCompounded([0, '2px' as CssValue, '8px' as CssValue,
                cssOklch({ l: 0, c: 0, h: 0, a: 0.1, },),],),
            }, },),
            $({ rule: '&:focus-visible', decls: {
              outline: cssCompounded(['2px' as CssValue, 'solid',
                cssVar('color-accent',),],),
            }, },),
            $({ rule: '& > .title',
              decls: { 'font-size': cssRem(1.25,), 'font-weight': 600, }, },),
          ],
        },);
        expect(result,).toBe(
          '.card{display:flex;flex-direction:column;gap:var(--gap-md);border-radius:0.5rem;&:hover{box-shadow:0 2px 8px oklch(0 0 0 / 0.1)}&:focus-visible{outline:2px solid var(--color-accent)}& > .title{font-size:1.25rem;font-weight:600}}',
        );
      },
    },),
    //endregion
  ],
},);
