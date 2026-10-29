/**
 Package tasks for `@monochromatic-dev/toml-edit-core`, replacing the
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
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'lint:types': { extends: 'lint:types', },
  'test:unit': { extends: 'test:unit', },
  'buildAndTest': {
    description: 'Build the neutral bundle, then run unit tests against the built dist',
    run: ['node package/monorepo-task/cli/src/run.ts //package/toml-edit/core:build', 'node package/monorepo-task/cli/src/run.ts //package/toml-edit/core:test:unit',],
  },
  'test:mutation': {
    description: 'Run container-isolated mutation testing against the full toml-edit unit suite',
    usage: 'arg "[args]" var=#true help="Optional mutation-test flags and package-relative source files"',
    run: {
      shell: 'node --input-type=module-typescript -e',
      run: `
const args = parseUsageArgs(process.env.usage_args ?? '')
runWorkspaceNode('package/mutation-test/cli', 'cli', ['--package', 'package/toml-edit/core', '--full-suite', ...args])
`,
    },
  },
  'test:conformance': {
    description: 'Run the upstream toml-test suite against the package decode/encode adapters for TOML 1.0 and 1.1',
    run: {
      shell: 'node --input-type=module-typescript -e',
      run: `
runTaskCli('//package/toml-edit/core:build')
// Follow the latest upstream runner through npx so corpus updates are
// exercised; the version is logged so a future failure can be traced to an
// upstream corpus change versus a local code change.
const runner = ['--yes', 'toml-test']
const env = { ...process.env, MONOCHROMATIC_WARN: 'false' }
execFileSync('npx', [...runner, 'version'], { stdio: 'inherit', env })
const decoder = join(process.env.TASK_REPO_ROOT, 'package', 'module', 'toml-edit', 'src', 'conformance', 'decode.ts')
const encoder = join(process.env.TASK_REPO_ROOT, 'package', 'module', 'toml-edit', 'src', 'conformance', 'encode.ts')
for (const version of ['1.0', '1.1']) {
  console.log('toml-test conformance: TOML ' + version)
  execFileSync('npx', [...runner, 'test',
    '-decoder=node ' + decoder + ' ' + version,
    '-encoder=node ' + encoder + ' ' + version,
    '-toml=' + version, '-timeout=30s'],
    { stdio: 'inherit', env })
}
console.log('toml-test conformance: all decoder, encoder, and invalid cases passed for 1.0 and 1.1')
`,
    },
  },
} satisfies PackageTaskFile;
