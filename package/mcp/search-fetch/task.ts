/**
 Package tasks for `@monochromatic-dev/mcp-search-fetch`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/dev-script-task-cli`.
 */

import type { PackageTaskFile, } from '../../dev-script/task-cli/src/types.ts';

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
    description: 'Build the node MCP-server bin, then run unit tests against the built artifact',
    run: [
      'node package/dev-script/task-cli/src/run.ts //package/mcp/search-fetch:build',
      'node package/dev-script/task-cli/src/run.ts //package/mcp/search-fetch:test:unit',
    ],
  },
} satisfies PackageTaskFile;
