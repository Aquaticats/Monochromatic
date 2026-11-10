/**
 Package tasks for `@monochromatic-dev/agent-harness-shared-neuralwatt-listing`,
 mirroring this package's `mise.toml` tasks as node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.

 CI reads only this file, so `build` has to be declared here for `pnpr-publish` to
 build the package before packing it (`doc/decision/mise-free-ci.md`).
 `refresh-snapshot` stays in `mise.toml` alone: it fetches the live listing, so no
 CI path runs it.
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
  'buildAndTest': {
    description: 'Build the Node bundle, then run unit tests against the built artifact',
    depends: [
      'build',
      'test:unit',
    ],
  },
} satisfies PackageTaskFile;
