'use strict';

throw new Error(
  '[blocked-dep] This package was blocklisted in pnpm-workspace.yaml/.pnpmfile.mjs '
    + 'and substituted with @monochromatic-dev/pnpm-override-stub-throwing. '
    + 'See doc/dependency-blocklist.md for the policy table and rationale.',
);
