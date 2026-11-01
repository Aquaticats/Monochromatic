/**
 Package tasks for `@monochromatic-dev/env-local-json-core`, replacing the
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
  'lint:types': { extends: 'lint:types', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'test:unit': { extends: 'test:unit', },
  'test:container': {
    description: 'Run the reader container matrix in Ubuntu as root and non-root',
    run: 'node src/mise.container-test.ts',
  },
  'buildAndTest': {
    description: 'Build the Node bundle, then run unit tests against the built artifact',
    run: [
      'node package/monorepo-task/cli/src/run.ts //package/env-local-json/core:build',
      'node package/monorepo-task/cli/src/run.ts //package/env-local-json/core:test:unit',
    ],
  },
} satisfies PackageTaskFile;
