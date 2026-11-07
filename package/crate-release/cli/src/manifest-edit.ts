import { exactPinPrefix } from './constants.ts';
import {
  inlinePinSites,
  tablePinSites,
} from './dependency-pin-sites.ts';
import { CrateReleaseError } from './error.ts';
import { packageVersionLines } from './package-version.ts';
import {
  endsWithComment,
  joinManifest,
  splitManifest,
  spliceValue,
} from './toml-line.ts';

/**
 Result of editing a manifest text.
 */
export type ManifestEdit = {
  /**
   The manifest text after the edit; the input text itself when nothing changed.
   */
  readonly text: string;
  /**
   True when at least one line changed.
   */
  readonly changed: boolean;
  /**
   Number of lines that changed; zero when the manifest already held the target.
   */
  readonly replacedLines: number;
};

/**
 Decide what a located version value means for the release.

 @param found - value found in the manifest

 @param current - value a release replaces

 @param target - value a release writes

 @param description - what the value is, for the message

 @returns the action: replace it, or leave it because it already holds the target

 @throws CrateReleaseError when the value is neither the current one nor the target

 @example
 ```ts
 decideValue({ found: '0.4.2', current: '0.4.2', target: '0.4.3', description: 'x' });
 ```
 */
function decideValue({
  found,
  current,
  target,
  description,
}: {
  readonly found: string;
  readonly current: string;
  readonly target: string;
  readonly description: string;
}): 'replace' | 'keep' {
  if (found === current)
    return 'replace';
  if (found === target)
    return 'keep';
  throw new CrateReleaseError(
    `${description} is ${JSON.stringify(found)}, which is neither the current ${JSON.stringify(current)} nor the target ${JSON.stringify(target)}; fix the manifest or the arguments, then run again.`,
  );
}

/**
 Replace the crate's own version in the package table of a manifest.

 Only the `version` key of the `[package]` table counts: a `rust-version` key, a `version` key of another table, or a
 dependency's version is never touched. The current value must equal `from` exactly (no prefix or substring match);
 a value that already equals `to` is left alone, so a release interrupted after this edit can be run again.

 @param text - manifest text

 @param from - version the manifest must hold now

 @param to - version to write

 @returns the edited text, whether it changed, and how many lines changed

 @throws CrateReleaseError when the package table has no quoted `version`, holds text after the value that is not a
 comment, or holds a version that is neither `from` nor `to`

 @example
 ```ts
 replacePackageVersion({ text: '[package]\nversion = "0.4.2"\n', from: '0.4.2', to: '0.4.3' });
 ```
 */
export function replacePackageVersion({
  text,
  from,
  to,
}: {
  readonly text: string;
  readonly from: string;
  readonly to: string;
}): ManifestEdit {
  /**
   Line bodies and their carriage returns.
   */
  const {
    bodies,
    hadReturn
  } = splitManifest(text);
  /**
   Lines of the package table that carry a quoted `version` key; TOML allows one, so the first is the one.
   */
  const candidates = packageVersionLines(bodies);
  /**
   The line that holds the version.
   */
  const [first] = candidates;
  if (first === undefined)
    throw new CrateReleaseError(
      'The manifest has no quoted version = "..." line in its [package] table (an inherited version.workspace key cannot be released here).',
    );
  if (!endsWithComment(first.body
    .slice(first.location
      .valueEnd
      + 1)))
    throw new CrateReleaseError(
      `The [package] version line ${JSON.stringify(first.body)} has text after the value that is not a comment.`,
    );
  if (decideValue({
    found: first.location
      .value,
    current: from,
    target: to,
    description: 'The [package] version'
  }) === 'keep')
    return {
      text,
      changed: false,
      replacedLines: 0
    };
  return {
    text: joinManifest({
      bodies: bodies.with(
        first.index,
        spliceValue({
          body: first.body,
          location: first.location,
          replacement: to
        })
      ),
      hadReturn,
    }),
    changed: true,
    replacedLines: 1,
  };
}

/**
 Move the exact pin of a dependency on a crate from one version to another.

 A pin is the requirement `"=<version>"` of a dependency written in one of two forms: an inline table on one line,
 `<crate> = { ..., version = "=<version>" }`, or a table of its own, `[dependencies.<crate>]` (also
 `[dev-dependencies.<crate>]` and `[build-dependencies.<crate>]`) with `version = "=<version>"` on a line of its own.
 Every such dependency is checked, and each must hold `=<from>` (changed) or `=<to>` (kept); any other requirement is
 refused, so a loose requirement or a stale pin is never rewritten silently. Only the quoted pin changes: the crate's
 own version, other crates' pins, and a crate whose name merely starts with this one are left alone.

 @param text - manifest text

 @param crateName - crate whose pin moves

 @param from - version the pin must hold now

 @param to - version to pin

 @returns the edited text, whether it changed, and how many lines changed

 @throws CrateReleaseError when no inline table or dependency table of the crate exists, one has no quoted `version`
 key, or a pin is neither `=<from>` nor `=<to>`

 @example
 ```ts
 replaceDependencyPin({ text: 'a = { version = "=1.0.0" }', crateName: 'a', from: '1.0.0', to: '1.0.1' });
 ```
 */
export function replaceDependencyPin({
  text,
  crateName,
  from,
  to,
}: {
  readonly text: string;
  readonly crateName: string;
  readonly from: string;
  readonly to: string;
}): ManifestEdit {
  /**
   Line bodies and their carriage returns.
   */
  const {
    bodies,
    hadReturn
  } = splitManifest(text);
  /**
   Every place that pins the crate, in either form.
   */
  const sites = [
    ...inlinePinSites({
      bodies,
      crateName
    }),
    ...tablePinSites({
      bodies,
      crateName
    }),
  ];
  if (sites.length === 0)
    throw new CrateReleaseError(
      `No dependency of the form ${crateName} = { ... } or [dependencies.${crateName}] (also [dev-dependencies.${crateName}] and [build-dependencies.${crateName}]) exists in the manifest.`,
    );
  /**
   Edited text of each line whose pin moves, by line index.
   */
  const replacements = new Map(sites.flatMap(function planPin({
    index,
    body,
    location,
  }): [
    number,
    string
  ][] {
    return decideValue({
      found: location.value,
      current: `${exactPinPrefix}${from}`,
      target: `${exactPinPrefix}${to}`,
      description: `The pin of ${crateName}`
    }) === 'replace' ? [
      [
        index,
        spliceValue({
          body,
          location,
          replacement: `${exactPinPrefix}${to}`
        })
      ]
    ] : [];
  }));
  if (replacements.size === 0)
    return {
      text,
      changed: false,
      replacedLines: 0
    };
  return {
    text: joinManifest({
      bodies: bodies.map(function applyReplacement(
        body,
        index
      ) {
        return replacements.get(index) ?? body;
      }),
      hadReturn,
    }),
    changed: true,
    replacedLines: replacements.size,
  };
}
