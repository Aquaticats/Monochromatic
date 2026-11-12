import {
  decimalDigits,
  usesOnly,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';

import { isReleaseVersion } from './version.ts';

/**
 What a version's label says about where it goes: a plain number, a release candidate, a beta, an alpha, or a form
 the rule does not name.
 */
export type ReleaseLabel = 'stable' | 'rc' | 'beta' | 'alpha' | 'unlabelled';

/**
 Labels that go to crates.io, by the owner's rule of 2026-11-07 ("Label decides"): a plain number and `-rc.N`.
 Versions ending `-alpha.N` or `-beta.N` go to GitHub releases only.
 */
const cratesIoLabels: ReadonlySet<ReleaseLabel> = new Set<ReleaseLabel>([
  'stable',
  'rc'
]);

/**
 Pre-release labels the rule names, each followed by one number: `-alpha.N`, `-beta.N` and `-rc.N`.
 */
const namedPreReleaseLabels: readonly ReleaseLabel[] = [
  'alpha',
  'beta',
  'rc'
];

/**
 Give a version the label the owner's rule reads from it.

 The label is the first dot-separated identifier of the pre-release part, and only the exact shape `<label>.<N>` with
 one number `N` counts. Build metadata after a plus sign is ignored. A text that is not a release version, a label
 without a number, a longer pre-release, another label and another letter case are all `unlabelled`, so a form the rule
 does not name never reaches crates.io, where a publish cannot be undone. The workflow action
 `.github/actions/decide-whether-a-version-goes-to-crates-io` makes the same decision in shell, and a unit test compares
 the two over one list of versions.

 @param version - version text

 @returns `stable` for a plain number, `rc`, `beta` or `alpha` for `-rc.N`, `-beta.N` or `-alpha.N`, else `unlabelled`

 @example
 ```ts
 releaseLabel('0.2.7-alpha.1');
 ```
 */
export function releaseLabel(version: string): ReleaseLabel {
  if (!isReleaseVersion(version))
    return 'unlabelled';
  /**
   The version without its build metadata.
   */
  const withoutBuild = version.split('+')[0] ?? version;
  /**
   Position of the hyphen that starts the pre-release.
   */
  const hyphen = withoutBuild.indexOf('-');
  if (hyphen === (-1))
    return 'stable';
  /**
   The label and the number of the pre-release, and anything after them.
   */
  const [name, number, ...extra] = withoutBuild.slice(hyphen + 1)
    .split('.');
  /**
   The label the name spells, when the rule names it.
   */
  const named = namedPreReleaseLabels.find(function spellsName(label) {
    return label === name;
  });
  if ((named === undefined) || (number === undefined)
    || (extra.length > 0)
    || (!usesOnly({
      text: number,
      allowed: decimalDigits
    })))
    return 'unlabelled';
  return named;
}

/**
 Decide whether a version is published to crates.io.

 @param version - version text

 @returns true for a plain number and `-rc.N`; false for `-alpha.N`, `-beta.N` and every form the rule does not name

 @example
 ```ts
 goesToCratesIo('0.2.7-alpha.1');
 ```
 */
export function goesToCratesIo(version: string): boolean {
  return cratesIoRoute(version)
    .goes;
}

/**
 Say whether a version is an alpha, the label the owner's standing rule of 2026-11-11 exempts from the work a release
 otherwise does.

 The predicate is `releaseLabel` itself, so the pipeline never decides what an alpha is a second time: a version the
 label rule does not name is not an alpha, whatever it looks like.

 @param version - version text

 @returns true for `-alpha.N` only; false for `-beta.N`, `-rc.N`, a plain number and every form the rule does not name

 @example
 ```ts
 isAlphaVersion('0.2.7-alpha.1');
 ```
 */
export function isAlphaVersion(version: string): boolean {
  return releaseLabel(version) === 'alpha';
}

/**
 Where a version goes with respect to crates.io: there, or only to GitHub releases with the reason why not.
 */
export type CratesIoRoute =
  | { readonly goes: true }
    | {
      readonly goes: false;
    /**
     Why the version is not published to crates.io, written for a person reading a report.
     */
      readonly reason: string;
    };

/**
 Say where a version goes with respect to crates.io, and why not when it stays on GitHub.

 @param version - version text

 @returns `{ goes: true }` for a plain number and `-rc.N`, else `{ goes: false, reason }` that names the label

 @example
 ```ts
 cratesIoRoute('0.2.7-alpha.1');
 ```
 */
export function cratesIoRoute(version: string): CratesIoRoute {
  /**
   The label of the version.
   */
  const label = releaseLabel(version);
  if (cratesIoLabels.has(label))
    return { goes: true };
  return {
    goes: false,
    reason: label === 'unlabelled'
      ? 'an unlabelled version is not published to crates.io (only a plain number and -rc.N are)'
      : `the ${label} label is not published to crates.io (only a plain number and -rc.N are)`,
  };
}
