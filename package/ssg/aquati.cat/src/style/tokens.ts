/**
 Design token CSS custom properties.
 
 Defines all color tokens as a `:root` block with dark mode overrides
 and inverse-mode overrides for the theme toggle.
 Every color value in the site references these tokens via `var(--token-name)`.
 */
import {
  cssCalc,
  type CssDeclarations,
  cssOklchFrom,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 Light mode token values (excluding theme-invariant primary colors).
 
 @returns declarations object reusable across `:root` and inverse overrides
 
 @example
 ```ts
 const decls = lightTokenDecls();
 // { '--color-fg': ..., '--color-bg': ..., ... }
 ```
 */
function lightTokenDecls(): CssDeclarations {
  return {
    '--color-fg': cssOklchFrom({
      from: cssVar('primary-dark',),
      l: cssCalc('l * 0.5',),
      c: cssCalc('c * 0.5',),
    },),
    '--color-bg': cssOklchFrom({
      from: cssVar('primary-light',),
      l: cssCalc('1 - ((1 - l) * 0.01)',),
      c: cssCalc('c * 0.5',),
    },),
    '--color-link': cssVar('primary-dark',),
    '--color-link-visited': '#551a8b',
    '--color-code-bg': '#f5f5f5',
    '--color-border': '#e0e0e0',
    '--color-muted': '#666',
    '--color-subtle': '#888',
    '--color-focus-ring': '#0066cc',
    '--hl-keyword': '#a626a4',
    '--hl-string': '#50a14f',
    '--hl-comment': '#a0a1a7',
    '--hl-number': '#986801',
    '--hl-type': '#c18401',
    '--hl-function': '#4078f2',
    '--hl-property': '#e45649',
    '--hl-heading': '#4078f2',
    '--hl-link': '#0066cc',
    '--hl-emphasis': '#c18401',
  };
}

/**
 Dark mode token values.
 
 @returns declarations object reusable across dark media query and inverse overrides
 
 @example
 ```ts
 const decls = darkTokenDecls();
 // { '--color-fg': ..., '--color-bg': ..., ... }
 ```
 */
function darkTokenDecls(): CssDeclarations {
  return {
    '--color-fg': cssOklchFrom({
      from: cssVar('primary-light',),
      l: cssCalc('1 - ((1 - l) * 0.5)',),
      c: cssCalc('c * 0.2',),
    },),
    '--color-bg': cssOklchFrom({
      from: cssVar('primary-dark',),
      l: cssCalc('l * 0.2',),
      c: cssCalc('c * 0.5',),
    },),
    '--color-link': cssVar('primary-light',),
    '--color-link-visited': '#c084fc',
    '--color-code-bg': cssOklchFrom({
      from: cssVar('color-bg',),
      c: cssCalc('c * 0.1',),
    },),
    '--color-border': '#333',
    '--color-muted': '#999',
    '--color-subtle': '#aaa',
    '--color-focus-ring': '#6db3f2',
    '--hl-keyword': '#c678dd',
    '--hl-string': '#98c379',
    '--hl-comment': '#5c6370',
    '--hl-number': '#d19a66',
    '--hl-type': '#e5c07b',
    '--hl-function': '#61afef',
    '--hl-property': '#e06c75',
    '--hl-heading': '#61afef',
    '--hl-link': '#6db3f2',
    '--hl-emphasis': '#e5c07b',
  };
}

/**
 Generates the `:root` block with all color custom properties.
 
 @returns CSS string with custom property declarations
 
 @example
 ```ts
 const css = tokenStyles();
 // ':root { --primary-light: #bf97e3; ... }'
 ```
 */
export function tokenStyles(): string {
  return $({
    rule: ':root',
    decls: {
      '--primary-light': '#bf97e3',
      '--primary-dark': '#4e318f',
      ...lightTokenDecls(),
    },
  },);
}

/**
 Generates the dark mode override block via `prefers-color-scheme`.
 
 Swaps light color values for dark-appropriate alternatives.
 
 @returns CSS `@media` block string with dark color overrides
 
 @example
 ```ts
 const css = darkModeTokenStyles();
 // '\@media (prefers-color-scheme: dark) { :root { ... } }'
 ```
 */
export function darkModeTokenStyles(): string {
  return $({
    at: 'media',
    params: '(prefers-color-scheme: dark)',
    children: [
      $({
        rule: ':root',
        decls: darkTokenDecls(),
      },),
    ],
  },);
}

/**
 Generates inverse-mode token overrides driven by the theme toggle checkbox.
 
 When `#theme-toggle` is checked, `:has()` flips the active token set:
 system-light users get dark tokens, system-dark users get light tokens.
 
 @returns CSS string with inverse override rules
 
 @example
 ```ts
 const css = inverseTokenStyles();
 ```
 */
export function inverseTokenStyles(): string {
  return [
    $({
      rule: ':root:has(#theme-toggle:checked)',
      decls: darkTokenDecls(),
    },),
    $({
      at: 'media',
      params: '(prefers-color-scheme: dark)',
      children: [
        $({
          rule: ':root:has(#theme-toggle:checked)',
          decls: lightTokenDecls(),
        },),
      ],
    },),
  ]
    .join('\n',);
}
