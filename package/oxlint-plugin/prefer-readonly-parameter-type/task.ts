/**
 Package tasks for `@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type`,
 replacing the `mise.toml` tasks CI ran through `mise run` with node-native
 definitions for `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'build': { extends: 'build', },
  'build:js': { extends: 'build:js', },
  'build:js:node': {
    extends: 'build:js:node',
    run: 'node ../../../node_modules/rolldown/bin/cli.mjs --configLoader native --config rolldown.node.config.ts',
  },
  'watch:build': { extends: 'watch:build', },
  'watch:build:js': { extends: 'watch:build:js', },
  'watch:build:js:node': { extends: 'watch:build:js:node', },
  'lint': { extends: 'lint', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'lint:types': { extends: 'lint:types', },
  'test:unit': { extends: 'test:unit', depends: ['build',], },
  'test:semantic-bridge-host': {
    description: 'Exercise built semantic bridge lifecycle on current host',
    run: 'node src/typescript-sync-adapter.unit.test.ts',
  },
  'test:external-consumer-host': {
    description: 'Exercise built package from external consumer on current host',
    run: 'node src/external-consumer.unit.test.ts',
  },
  'test:external-channel-workers': {
    description: 'Exercise external channel worker paths',
    run: 'node src/external-channel-workers.unit.test.ts',
  },
} satisfies PackageTaskFile;
