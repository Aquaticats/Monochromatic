/**
 {@link safeParse}: parse TOML text and wrap parser failures as a package error.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  type AST,
  ParseError,
  parseTOML,
} from 'toml-eslint-parser';

import { TomlEditError, } from './errors.ts';
import type { TomlEditOptions, } from './types.ts';

/**
 Parse `source` and wrap any parser failure in a {@link TomlEditError}.
 
 Shared by {@link parseTomlEdit} and the sibling text transforms so every entry
 point reports invalid input through the same contract; the export is not
 re-exported from the package entry point.
 
 @param source - TOML text to parse
 
 @param tomlVersion - TOML language version to enforce; `undefined` keeps the parser default
 
 @returns Root parser result retaining foreign AST ownership provenance
 
 @throws {@link TomlEditError} when the parser rejects the text
 
 @example
 ```ts
 safeParse({ source: 'foo = "bar"\n', tomlVersion: undefined, },);
 ```
 */
export function safeParse(
  {
    source,
    tomlVersion,
  }: {
    readonly source: string;
    readonly tomlVersion: TomlEditOptions['tomlVersion'];
  },
): ForeignBorrowed<AST.TOMLProgram> {
  try {
    return parseTOML(
      source,
      tomlVersion === undefined ? undefined : { tomlVersion, },
    );
  }
  catch (e: unknown) {
    if (e instanceof ParseError) {
      throw new TomlEditError(
        `Failed to parse TOML: ${e.message}`,
        { cause: e, },
      );
    }
    // `parseTOML` is a pure parse function, so any non-`ParseError` throw is
    // still input-induced and must surface as `TomlEditError` to honor the
    // documented parse contract. The known case is a `RangeError` from stack
    // overflow on pathologically deep `[`/`{` nesting; wrapping it (rather than
    // leaking the raw exception) keeps the callers total over arbitrary text.
    throw new TomlEditError(
      `Failed to parse TOML: ${caughtValueText(e,)}`,
      { cause: e, },
    );
  }
}
