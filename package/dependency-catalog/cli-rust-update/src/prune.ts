/**
 Pruning of exclude selectors that no locked version needs any more
 (pnpm's `minimumReleaseAgeExcludePrune` and `trustPolicyExcludePrune`).

 @module
 */

import type { RustCatalogSettings, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';
import {
  parseTomlEdit,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core/ts';

import {
  type LockedVersion,
  versionKey,
} from './lockfile-diff.ts';
import { pruneSelectors, } from './selectors.ts';

//region Prune

/**
 One exclude setting before and after pruning.

 @example
 ```ts
 const change: SelectorChange = { key: 'trustPolicyExclude', before: ['gone',], after: [], };
 ```
 */
type SelectorChange = {
  readonly key: 'minimumReleaseAgeExclude' | 'trustPolicyExclude';
  readonly before: readonly string[];
  readonly after: readonly string[];
};

/**
 Exclude settings that can be pruned, with the flag that enables each.
 */
const PRUNABLE = [
  {
    key: 'minimumReleaseAgeExclude',
    flag: 'minimumReleaseAgeExcludePrune',
  },
  {
    key: 'trustPolicyExclude',
    flag: 'trustPolicyExcludePrune',
  },
] as const;

/**
 Catalog text with stale exclude selectors removed.

 @param source - `rust-catalog.toml` text

 @param settings - parsed settings of that text

 @param locked - every version the lockfiles hold after the update

 @returns new text and the removed selectors; the text is unchanged when nothing was pruned

 @throws TomlEditError when the catalog cannot be edited

 @example
 ```ts
 pruneExcludes({ source, settings, locked, });
 ```
 */
export function pruneExcludes(
  {
    source,
    settings,
    locked,
  }: {
    readonly source: string;
    readonly settings: RustCatalogSettings;
    readonly locked: readonly LockedVersion[];
  },
): {
  readonly source: string;
  readonly removed: readonly string[]
} {
  /**
   Locked `name@version` keys.
   */
  const lockedKeys = new Set(locked.map(function toKey(entry: LockedVersion,): string {
    return versionKey(entry,);
  },),);
  /**
   Settings whose selectors change.
   */
  const changes = PRUNABLE.filter(function isEnabled(prunable,): boolean {
    return settings[prunable.flag];
  },)
    .map(function prune(prunable,): SelectorChange {
      /**
       Current selectors.
       */
      const before = settings[prunable.key];
      /**
       Selectors kept.
       */
      const after = pruneSelectors({
        selectors: before,
        locked: lockedKeys,
      },);
      return {
        key: prunable.key,
        before,
        after,
      };
    },)
    .filter(function changed(change: SelectorChange,): boolean {
      return change.before
        .join('\u0000',)
        !== change.after
          .join('\u0000',);
    },);
  if (changes.length === 0)
    return {
      source,
      removed: [],
    };
  /**
   Edited document.
   */
  const edit = changes.reduce(
    function apply(
      state: ReturnType<typeof parseTomlEdit>,
      change: SelectorChange,
    ): ReturnType<typeof parseTomlEdit> {
      return tomlSet({
        edit: state,
        path: [change.key,],
        value: [...change.after,],
      },);
    },
    parseTomlEdit({ source, },),
  );
  return {
    source: tomlStringify({ edit, },),
    removed: changes.flatMap(function removedOf(change: SelectorChange,): readonly string[] {
      return change.before
        .filter(function isGone(selector: string,): boolean {
          return !change.after
            .includes(selector,);
        },)
        .map(function label(selector: string,): string {
          return `${change.key}: ${selector}`;
        },);
    },),
  };
}

//endregion Prune
