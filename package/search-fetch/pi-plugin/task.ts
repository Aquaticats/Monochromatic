/**
 Package tasks for `@monochromatic-dev/search-fetch-pi-plugin`, replacing the
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
  'verify:extension': {
    description: 'Verify the built Pi extension loads and binds its tool surface',
    depends: ['build',],
    run: 'node src/mise.verify-extension.ts',
  },
} satisfies PackageTaskFile;
