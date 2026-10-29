/**
 Strict CSS property and at-rule type definitions.
 
 Omits disallowed properties from csstype's `PropertiesHyphen` and excludes
 banned at-rules from the at-rule name union, enforcing the project's
 CSS conventions at the type level (the checks the retired stylelint config
 performed at runtime).
 */
import type {
  AtRules,
  PropertiesHyphen,
} from 'csstype';

import type { DisallowedProperties, } from './disallowed-properties.ts';

import type {
  CssValue,
  StrictValue,
} from './values.ts';

export type { DisallowedProperties, } from './disallowed-properties.ts';

//region Disallowed at-rules

/**
 At-rule names banned by the project's style guide.
 
 - `charset`: unnecessary in UTF-8 workflows; HTTP `Content-Type` header is authoritative
 - `font-palette-values`: not yet broadly supported in the target browser baseline
 
 @example
 ```ts
 // StrictAtRuleName excludes these:
 type Bad = 'charset';    // ✗ not assignable to StrictAtRuleName
 type Good = 'media';     // ✓ assignable to StrictAtRuleName
 ```
 */
export type DisallowedAtRules = 'charset' | 'font-palette-values';

//endregion

//region Identifier properties

/**
 CSS properties whose values are user-defined identifiers, not CSS lengths/colors/keywords.
 
 These properties accept arbitrary strings via `(string & {})` because their values
 are names chosen by the author (e.g. `@keyframes` names), not constrained CSS tokens.
 Requiring `CssValue` constructors for these adds no safety.
 
 @example
 ```ts
 const decls: StrictCssDeclarations = {
   'animation-name': 'slide-in',      // plain string, OK, user-defined identifier
   'counter-reset': 'line',           // plain string, OK, user-defined counter name
   content: 'counter(line)',          // plain string, OK, counter function reference
   gap: '1rem',                       // type error: must use cssRem(1)
 };
 ```
 */
type IdentifierProperties =
  | 'anchor-name'
    | 'animation-name'
    | 'content'
    | 'counter-increment'
    | 'counter-reset'
    | 'counter-set'
    | 'hyphenate-character'
    | 'position-anchor';

//endregion

//region Strict declarations

/* oxlint-disable no-restricted-syntax/no-optional-escape -- external-boundary mirror of csstype's PropertiesHyphen optional-property model: a CSS declarations object names a subset of properties, each with a key-dependent value type (StrictValue<PropertiesHyphen[K]>) that a string index signature cannot express per-property */
/**
 Strict CSS declarations with disallowed properties omitted and values type-checked.
 
 - Property names: csstype's {@link PropertiesHyphen} minus {@link DisallowedProperties}
 - Property values: csstype keyword literals (minus named colors) plus `CssValue` branded type
 - Identifier properties: accept `(string & {})` for user-defined names (e.g. animation names)
 - Custom properties: `--*` accepted with `CssValue` or plain `string` values
 
 @example
 ```ts
 const decls: StrictCssDeclarations = {
   display: 'flex',                     // keyword literal
   gap: cssRem(1),                      // branded constructor
   'background-color': cssVar('bg'),    // branded var reference
   'animation-name': 'slide-in',        // plain string: identifier property
   width: cssRem(10),                   // type error: 'width' is disallowed
   color: 'red',                        // type error: named colors excluded
 };
 ```
 */
export type StrictCssDeclarations = {
  [K in Exclude<keyof PropertiesHyphen, DisallowedProperties>]?: K extends
    IdentifierProperties ? StrictValue<PropertiesHyphen[K]> | (string & {})
    : StrictValue<PropertiesHyphen[K]>;
} & Record<`--${string}`, CssValue | string>;
/* oxlint-enable no-restricted-syntax/no-optional-escape */

//endregion

//region Strict at-rule names

/**
 Strips the `@` prefix from csstype's {@link AtRules} union.
 
 Transforms `"@media" | "@layer" | ...` into `"media" | "layer" | ...`
 to match the h-css `at` field convention (prefix added during string building).
 
 @example
 ```ts
 type Name = StripAtPrefix<'\@media'>; // 'media'
 ```
 */
type StripAtPrefix<T,> = T extends `@${infer Name}` ? Name : never;

/**
 All standard CSS at-rule names without the `@` prefix, minus {@link DisallowedAtRules}.
 
 @example
 ```ts
 const at: StrictAtRuleName = 'media';           // allowed
 const bad: StrictAtRuleName = 'charset';         // type error
 ```
 */
export type StrictAtRuleName = Exclude<StripAtPrefix<AtRules>, DisallowedAtRules>;

//endregion
