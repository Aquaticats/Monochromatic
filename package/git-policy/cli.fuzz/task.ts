/**
 Package tasks for the git-policy fuzz sidecar, the mise-free mirror of this
 package's `mise.toml` entries that CI runs through
 `@monochromatic-dev/monorepo-task-cli` (`doc/decision/mise-free-ci.md`).

 Only the tasks CI runs live here: `lint:types` is the compile that proves the
 fuzz helper library, every fuzz target and the generator controls still build
 against `package/git-policy/cli`, run by
 `.github/workflows/git-policy-cli-fuzz.yml`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'lint:types': {
    description: 'Check the fuzz helper library, every fuzz target and the generator controls compile against the subject, with denied Clippy warnings',
    // `--locked` makes cargo refuse a sidecar Cargo.lock that fell behind the
    // subject's dependency graph, so a stale lock fails here before a fuzz
    // campaign would, and `--offline` keeps the run free of registry traffic.
    run: [
      'cargo check --all-targets --locked --offline',
      'cargo clippy --all-targets --locked --offline -- -D warnings',
    ],
  },
} satisfies PackageTaskFile;
