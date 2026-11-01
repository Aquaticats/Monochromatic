/**
 Pure application of a {@link CargoManifestPlan} to `Cargo.toml` text.
 
 Guarded keyed enforcements run through `tomlSet` in splice mode (comments and
 unowned keys survive byte-identically); absent-block insertions are appended
 as canonical block text, because `tomlSet`'s create path would otherwise emit
 a dotted top-level key rather than a `[table]` block.
 
 @module
 */

import {
  parseTomlEdit,
  type TomlEditState,
  tomlGetValue,
  tomlHas,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core/ts';

import type { Path, } from '../types.ts';
import { deepEqual, } from './deep-equal.ts';
import type {
  CargoEnforcement,
  CargoManifestPlan,
} from './types.ts';

/**
 Writes `value` at `path` only when the current effective value differs,
 returning the input state unchanged otherwise so splice mode keeps
 already-correct text byte-identical.
 
 @param edit - Current parsed state
 
 @param path - Target key path
 
 @param value - Canonical value to converge on
 
 @returns State reflecting the write, or the input state when already equal
 
 @example
 ```ts
 const next = setIfDiffers({ edit, path: ['package', 'edition',], value: '2024', });
 ```
 */
function setIfDiffers(
  {
    edit,
    path,
    value,
  }: {
    readonly edit: TomlEditState;
    readonly path: Path;
    readonly value: CargoEnforcement['value'];
  },
): TomlEditState {
  if (deepEqual({
    left: tomlGetValue({
      edit,
      path,
    },),
    right: value,
  },))
    return edit;

  return tomlSet({
    edit,
    path,
    value,
  },);
}

/**
 Applies one enforcement when its guard resolves, else returns the state
 unchanged.
 
 @param edit - Current parsed state
 
 @param enforcement - Guard, target path, and canonical value
 
 @returns State after the guarded write
 
 @example
 ```ts
 const next = applyEnforcement({ edit, enforcement, });
 ```
 */
function applyEnforcement(
  {
    edit,
    enforcement,
  }: {
    readonly edit: TomlEditState;
    readonly enforcement: CargoEnforcement;
  },
): TomlEditState {
  if (!tomlHas({
    edit,
    path: enforcement.guardPath,
  },))
    return edit;

  return setIfDiffers({
    edit,
    path: enforcement.path,
    value: enforcement.value,
  },);
}

/**
 Applies a plan to manifest text and returns the result.
 
 Absent-block eligibility is decided against the original parse (before any
 enforcement runs) so an inserted block is never double-counted. Blocks are
 appended after a single trailing newline, each separated by one blank line.
 
 @param content - Original `Cargo.toml` text
 
 @param plan - Enforcements and block insertions for that manifest
 
 @returns Updated text; identical to `content` when nothing changed
 
 @example
 ```ts
 const next = applyCargoPlan({ content, plan, });
 ```

 @internal
 */
export function applyCargoPlan(
  {
    content,
    plan,
  }: {
    readonly content: string;
    readonly plan: CargoManifestPlan;
  },
): string {
  /**
   Original parse; absent-block guards read from this pre-edit tree.
   */
  const original = parseTomlEdit({ source: content, },);
  /**
   Blocks whose target table is absent, so their canonical text is appended.
   */
  const pendingBlocks = plan.blocks
    .filter(function isAbsent(block,): boolean {
      return !tomlHas({
        edit: original,
        path: block.absentPath,
      },);
    },);
  /**
   State after every guarded keyed enforcement folds in.
   */
  const edited = plan.enforcements
    .reduce(
    function fold(
      edit,
      enforcement,
    ): TomlEditState {
      return applyEnforcement({
        edit,
        enforcement,
      },);
    },
    original,
    );
  /**
   Serialized body before any block append.
   */
  const body = tomlStringify({ edit: edited, },);
  if (pendingBlocks.length === 0)
    return body;

  return pendingBlocks.reduce(
    function append(
      text,
      block,
    ): string {
      return `${text}\n${block.text}`;
    },
    `${body.trimEnd()}\n`,
  );
}
