/**
 Package tasks for `@monochromatic-dev/module-p-map-fork`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/dev-script-task-cli`.
 */

import type { PackageTaskFile, } from '../../dev-script/task-cli/src/types.ts';

export default {
  'build': { extends: 'build', },
  'build:js': { extends: 'build:js', },
  'build:js:browser': { extends: 'build:js:browser', },
  'watch:build': { extends: 'watch:build', },
  'watch:build:js': { extends: 'watch:build:js', },
  'watch:build:js:browser': { extends: 'watch:build:js:browser', },
  'lint': { extends: 'lint', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
} satisfies PackageTaskFile;
