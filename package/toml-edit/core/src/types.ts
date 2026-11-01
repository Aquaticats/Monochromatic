/**
 Shared types for `@monochromatic-dev/toml-edit-core`.
 
 The state is an immutable editable document tree ({@link TomlEditState}); the
 LF-normalized `source` string is retained for verbatim emission of clean
 spans. Every mutating function returns a fresh state with a new `blocks`
 tree, sharing unchanged nodes by reference.
 
 @module
 */

import type { AST, } from 'toml-eslint-parser';

import type { Block, } from './document.ts';

/**
 Path into a TOML document.
 
 String segments name keys (bare or quoted; both forms collapse to the same
 raw string). Numeric segments index into arrays and into array-of-tables.
 
 @example
 ```ts
 const fruitsFirstName = ['fruits', 0, 'name',] as const satisfies TomlPath;
 ```
 */
export type TomlPath = readonly (string | number)[];

/**
 Fidelity mode.
 
 `'splice'`: original bytes are kept verbatim for unmutated nodes; only
 mutated nodes re-emit canonically.
 
 `'canonical'`: every node is rendered from structure, ignoring original
 source ranges.
 */
export type TomlEditMode = 'splice' | 'canonical';

/**
 `Block`-typed comment node from toml-eslint-parser.
 
 The `type` field is always `'Block'` despite TOML having no block-comment
 syntax; this mirrors the ESTree shape the parser implements.
 
 Declared as a deeply readonly structural view of `AST.Comment`,
 whose `range` tuple and `loc` object are mutable upstream.
 The projection preserves immutable edit-state ownership while parser comments
 remain structurally assignable because mutable members satisfy readonly targets.
 */
export type TomlComment = {
  readonly type: AST.Comment['type'];
  readonly value: string;
  readonly range: readonly [
    number,
    number,
  ];
  readonly loc: {
    readonly start: {
      readonly line: number;
      readonly column: number;
    };
    readonly end: {
      readonly line: number;
      readonly column: number;
    };
  };
};

/**
 Formatting options for canonical emission.
 
 Also consulted by splice mode when re-emitting a mutated node.
 */
export type CanonicalOptions = {
  readonly indent: number;
  readonly arrayInlineThreshold: number;
  readonly arrayInlineMaxColumns: number;
  readonly preferDottedKeysForCreate: boolean;
  readonly trailingNewline: boolean;
};

/**
 Caller-supplied subset of {@link CanonicalOptions}.
 
 Every field is individually optional so a caller overrides only the
 formatting knobs it cares about; the rest fall back to
 {@link DEFAULT_CANONICAL_OPTIONS} at merge time. Spelled out as a per-field
 optional type rather than `Partial<CanonicalOptions>` because the
 `no-optional-escape` rule bans the `Partial` mapped-type encoding of
 optionality; under `exactOptionalPropertyTypes` `field?: T` already means
 "absent or T".
 */
export type CanonicalOptionsOverride = {
  readonly indent?: number;
  readonly arrayInlineThreshold?: number;
  readonly arrayInlineMaxColumns?: number;
  readonly preferDottedKeysForCreate?: boolean;
  readonly trailingNewline?: boolean;
};

/**
 Options for {@link parseTomlEdit}.
 */
export type TomlEditOptions = {
  readonly source: string;
  readonly mode?: TomlEditMode;
  readonly canonical?: CanonicalOptionsOverride;
  readonly tomlVersion?: '1.0' | '1.1' | '1.0.0' | '1.1.0' | 'latest' | 'next';
};

/**
 Options for {@link emptyTomlEdit}.
 */
export type TomlEmptyOptions = {
  readonly canonical?: CanonicalOptionsOverride;
};

/**
 Immutable editable document state.
 
 Returned by {@link parseTomlEdit}, {@link emptyTomlEdit}, and every mutating
 function. `source` is the LF-normalized parse input, shared by reference and
 never mutated; `blocks` is the ordered top-level block list, functionally
 updated on every mutation.
 */
export type TomlEditState = {
  readonly source: string;
  readonly blocks: readonly Block[];
  readonly comments: readonly TomlComment[];
  readonly headerComment?: string;
  readonly mode: TomlEditMode;
  readonly canonical: Readonly<CanonicalOptions>;
};

/**
 Tagged value-input shape used to disambiguate JS-to-TOML coercion.
 
 Constructed via the factory functions in `./wrappers.ts`.
 */
export type TomlWrappedInput = {
  readonly tomlKind:
    | 'integer'
      | 'float'
      | 'local-date'
      | 'local-date-time'
      | 'local-time';
  readonly value: number | bigint | string;
};

/**
 Accepted runtime shape for {@link tomlSet}.
 
 Runtime validation in `./values.ts` produces a `TOMLContentNode`-equivalent
 payload; the static type stays `unknown` to avoid restricting the recursive
 shape past the point of usefulness.
 */
export type TomlValueInput = unknown;

/**
 Default canonical-mode options.
 
 Mode is `'splice'` for {@link parseTomlEdit}; {@link emptyTomlEdit} forces
 `'canonical'` since there is no source to splice.
 */
export const DEFAULT_CANONICAL_OPTIONS: CanonicalOptions = Object.freeze({
  indent: 2,
  arrayInlineThreshold: 4,
  arrayInlineMaxColumns: 80,
  preferDottedKeysForCreate: false,
  trailingNewline: true,
},);
