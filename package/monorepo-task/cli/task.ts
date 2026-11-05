/**
 Package tasks for `@monochromatic-dev/monorepo-task-cli`, the task CLI
 itself, so `task-run` can type-check the package every workflow runs it from.
 Build and test the CLI artifact as well as the source entry used by CI.
 */

import type { PackageTaskFile, } from './src/types.ts';

export default {
  'build': { extends: 'build', },
  'build:js': { extends: 'build:js', },
  'build:js:node': { extends: 'build:js:node', },
  'lint:types': { extends: 'lint:types', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'test:unit': { extends: 'test:unit', depends: ['build',], },
} satisfies PackageTaskFile;
