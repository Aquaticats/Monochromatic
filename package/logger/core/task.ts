/**
 Package tasks for `@monochromatic-dev/logger-core`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'build': { extends: 'build', },
  'build:js': { extends: 'build:js', },
  'build:js:browser': { extends: 'build:js:browser', },
  'build:js:node': { extends: 'build:js:node', },
  'watch:build': { extends: 'watch:build', },
  'watch:build:js': { extends: 'watch:build:js', },
  'watch:build:js:browser': { extends: 'watch:build:js:browser', },
  'watch:build:js:node': { extends: 'watch:build:js:node', },
  'lint': { extends: 'lint', },
  'test:unit': { extends: 'test:unit', },
  'lint:types': { extends: 'lint:types', },
  'watch:lint:types': { extends: 'watch:lint:types', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'test:burst-capacity': {
    description: 'Burst-capacity scenario for the logger',
    run: 'node --max-old-space-size=512 --expose-gc src/burst-capacity.test.ts',
  },
} satisfies PackageTaskFile;
