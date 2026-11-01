/**
 Package tasks for `@monochromatic-dev/monorepo-task-cli`, the task CLI
 itself, so `task-run` can type-check the package every workflow runs it from.
 Build, lint and test tasks stay with the package-completeness work recorded in
 `doc/handover/mise-free-ci-migration.md`.
 */

import type { PackageTaskFile, } from './src/types.ts';

export default {
  'lint:types': { extends: 'lint:types', },
} satisfies PackageTaskFile;
