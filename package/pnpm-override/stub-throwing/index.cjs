'use strict';

throw new Error(
  '[blocked-dep] This package was blocklisted in .pnpmfile.policies.json '
    + 'and substituted with @monochromatic-dev/pnpm-override-stub-throwing. '
    + 'See doc/dependency-blocklist.md for the decision rule and rationale.',
);
