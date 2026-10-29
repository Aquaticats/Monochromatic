/**
 Internal type definitions for the CSS hyperscript factory.
 
 Separated from the builders to stay within the line budget.
 */

import type { AtRule, } from 'csstype';

import type {
  StrictAtRuleName,
  StrictCssDeclarations,
} from './properties.ts';

//region Types

/**
 Maps at-rule names that have typed descriptor interfaces in csstype
 to their kebab-case descriptor types.
 
 At-rules not in this map (e.g. `media`, `supports`, `layer`) either
 take children instead of descriptors or have no typed descriptors in csstype.
 */
type AtRuleDeclsMap = {
  'counter-style': AtRule.CounterStyleHyphen;
  'font-face': AtRule.FontFaceHyphen;
  page: AtRule.PageHyphen;
  property: AtRule.PropertyHyphen;
  'view-transition': AtRule.ViewTransitionHyphen;
};

/**
 At-rule options for at-rules with typed descriptor interfaces.
 
 When `at` is `'font-face'`, `decls` accepts `AtRule.FontFaceHyphen`;
 when `at` is `'property'`, `decls` accepts `AtRule.PropertyHyphen`; etc.
 TypeScript narrows `decls` based on the `at` discriminant.
 */
type TypedAtRuleOptions = {
  [K in keyof AtRuleDeclsMap]: {
    /**
     At-rule name: narrows `decls` to the matching descriptor type
     */
    at: K;
    /**
     At-rule prelude/parameters (e.g. `'--color-fg'` for `@property`)
     */
    params?: string;
    /**
     At-rule descriptors with editor intellisense per at-rule type
     */
    decls?: AtRuleDeclsMap[K];
    /**
     Raw CSS string to inject inside the block (NOT escaped; caller responsible)
     */
    raw?: string;
    /**
     Nested rules or at-rules inside this block
     */
    children?: readonly string[];
  };
}[keyof AtRuleDeclsMap];

/**
 At-rule names that do not have typed descriptor interfaces in csstype,
 minus disallowed at-rules.
 */
type UntypedAtRuleName = Exclude<StrictAtRuleName, keyof AtRuleDeclsMap>;

/**
 At-rule options for at-rules without typed descriptors.
 
 Accepts any `Record<string, string>` for `decls` (if applicable)
 and `(string & {})` as an escape hatch for non-standard at-rule names.
 */
type UntypedAtRuleOptions = {
  /**
   At-rule name: standard names get autocomplete, arbitrary strings accepted via `(string & {})`
   */
  at: UntypedAtRuleName | (string & {});
  /**
   At-rule prelude/parameters (e.g. `'(prefers-color-scheme: dark)'` for `@media`)
   */
  params?: string;
  /**
   At-rule descriptor declarations (untyped for at-rules without csstype descriptors)
   */
  decls?: Record<string, string>;
  /**
   Raw CSS string to inject inside the block (NOT escaped; caller responsible)
   */
  raw?: string;
  /**
   Nested rules or at-rules inside this block
   */
  children?: readonly string[];
};

/**
 Options for a CSS at-rule.
 
 At-rules come in three forms:
 - **Block with children**: `@media (...) { rules }`, provide `children`
 - **Block with declarations**: `@property --x { syntax: ... }`, provide `decls`
 - **Statement**: `@layer tokens;`, omit `decls`, `raw`, and `children` entirely
 
 For at-rules with csstype descriptor interfaces (`font-face`, `property`,
 `counter-style`, `page`, `view-transition`),
 the `decls` field is typed to the matching descriptor interface.
 For all other at-rules, `decls` accepts `Record<string, string>`.
 
 **Statement-form output requires omitting all three of `decls`, `raw`,
 and `children`.** Passing an empty `decls: {}`, empty `raw: ''`, or
 empty `children: []` still yields a block. For example, `$({ at: 'font-face', decls: {} })`
 produces `\@font-face{}`, not `\@font-face;`. This means the library cannot
 emit `\@font-face;` (which is invalid CSS anyway, since `\@font-face` requires
 a block); to emit a statement at-rule the input must contain no body fields at all.
 */
export type AtRuleOptions = TypedAtRuleOptions | UntypedAtRuleOptions;

/**
 Options for a CSS style rule.
 
 Produces `selector { declarations; children }`.
 */
export type RuleOptions = {
  /**
   CSS selector (e.g. `'.card'`, `'&:hover'`, `':root'`)
   */
  rule: string;
  /**
   CSS declarations: strict property names, strict values, custom properties
   */
  decls?: StrictCssDeclarations;
  /**
   Raw CSS string to inject inside the block (NOT escaped; caller responsible)
   */
  raw?: string;
  /**
   Nested rules or at-rules inside this block
   */
  children?: readonly string[];
};

/**
 Union of all CSS hyperscript option shapes.
 
 Discriminated by presence of `rule` (style rule) vs `at` (at-rule).
 */
export type CssOptions = AtRuleOptions | RuleOptions;

//endregion
