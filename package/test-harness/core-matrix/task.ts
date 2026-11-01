/**
 Package tasks for `@monochromatic-dev/test-harness-core-matrix`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'build': { extends: 'build', },
  'build:js': { extends: 'build:js', },
  'build:js:node': { extends: 'build:js:node', },
  'watch:build': { extends: 'watch:build', },
  'watch:build:js': { extends: 'watch:build:js', },
  'watch:build:js:node': { extends: 'watch:build:js:node', },
  'lint': { extends: 'lint', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'lint:types': { extends: 'lint:types', },
  'test': {
    description: 'Run matrix self-tests against the built artifact (builds first)',
    depends: ['build',],
    run: [
      'node src/distro.unit.test.ts',
      'node src/runtime.unit.test.ts',
      'node src/container.unit.test.ts',
      'node src/cachyos-image.unit.test.ts',
      'node src/discover.unit.test.ts',
    ],
  },
} satisfies PackageTaskFile;
