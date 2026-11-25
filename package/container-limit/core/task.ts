/**
 Package tasks for `@monochromatic-dev/container-limit-core`, the node-native definitions of the `mise.toml` tasks for
 `@monochromatic-dev/monorepo-task-cli`, so CI's `task-run` reaches this package's unit tests (the container start sites
 ledger among them; issue 6456).
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'lint': { extends: 'lint', },
  'lint:types': { extends: 'lint:types', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'test:unit': { extends: 'test:unit', },
} satisfies PackageTaskFile;
