# `eslint/one-var` is set to `never`

Ratified 2026-08-13 by the user, who identified the cause.

## The decision

`package/config/oxlint/src/rule/style.ts` sets `'eslint/one-var': ['warn', 'never']`.
 One declaration per statement, which is the direction this repository has
 always been written in.

## What was actually wrong

The rule was never configured. It arrived enabled by the `categories.style`
 sweep carrying its opposite default, and told every consecutive `const` to
 merge into its predecessor: 1447 warnings across code nobody had touched.

Those warnings were invisible for some time because the working tree had been
 linting against a `node_modules` stale relative to the lockfile. An unrelated
 `pnpm install` synced them and the warnings appeared all at once, which is what
 made it look like a regression rather than a setting.

## A wrong diagnosis, recorded because the shape recurs

The agent claimed the rule conflicted with the repository's TSDoc-per-declaration
 requirement and proposed disabling it on those grounds. That was asserted, not
 tested, and it is false. Measured with a probe file and the real linter:

-   A combined declaration carrying ONE TSDoc lints clean. `require-tsdoc` does
    not fire.
-   An inner TSDoc placed before a second declarator is also accepted.

There is a real objection, but it is a different one, and it is only visible in
 a real flagged site rather than in a constructed example: the rule's advice is
 unfollowable where an exported and a module-private constant are adjacent,
 since one statement cannot be half exported.

## Verified in both directions

A rule that has been silenced and a rule that has been corrected produce the
 same warning count, so both directions were checked:

-   Before 1447 warnings in `module-translation-repair`, after 0, and 0
    repo-wide.
-   Positive control: a deliberately combined `const alpha = 1, beta = 2;` is
    still flagged, now reading "Split 'const' declarations into multiple
    statements".
