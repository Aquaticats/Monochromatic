/**
 Package tasks for `@monochromatic-dev/module-logger-fuzz`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'bundle:browser-properties': {
    description: 'Bundle the browser property layer for the Playwright harness page',
    extends: 'build:js:client',
  },
  'lint': { extends: 'lint', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'lint:types': { extends: 'lint:types', },
  'test:unit': { extends: 'test:unit', },
  'fuzz': {
    description: 'Build module-logger, then run *.property.unit.test.ts as a time-budgeted fast-check campaign under node',
    usage: 'flag "--budget <ms>" help="Per-property time budget in milliseconds (default 60000)"',
    run: {
      shell: 'node --input-type=module-typescript -e',
      run: `
const budget = (process.env.usage_budget ?? '60000').trim()
runTaskCli('//package/logger/core:build')
const { glob } = await import('node:fs/promises')
const files = []
for await (const file of glob('src/**/*.property.unit.test.ts')) { files.push(file) }
console.log('Fuzz campaign over ' + files.length + ' property files at ' + budget + 'ms each')
for (const file of files) {
  console.log('fuzzing ' + file)
  execFileSync(process.execPath, [file], { stdio: 'inherit', env: { ...process.env, LOGGER_FUZZ_BUDGET_MS: budget } })
}
`,
    },
  },
  'fuzz:coverage': {
    description: 'Deterministic V8 line-coverage reachability gate over the module-logger runtime src; --write refreezes the baseline',
    usage: 'flag "--write" help="Refreeze the committed baseline from this run instead of checking against it"',
    run: {
      shell: 'node --input-type=module-typescript -e',
      run: `
const root = join(process.env.TASK_REPO_ROOT, 'package', 'logger', 'core.fuzz')
const mode = (process.env.usage_write === 'true') ? 'write' : 'check'
const dir = mkdtempSync(join(scratchBase(), 'logger-cov-'))
try {
  execFileSync(process.execPath, ['--localstorage-file', join(dir, 'localstorage.sqlite'), join(root, 'src', 'coverage-driver.ts')], { stdio: 'inherit', env: { ...process.env, NODE_V8_COVERAGE: join(dir, 'v8') } })
  execFileSync(process.execPath, [join(root, 'src', 'coverage-report.ts'), mode, join(dir, 'v8'), join(root, 'coverage-baseline.json')], { stdio: 'inherit' })
} finally {
  const { rmSync } = await import('node:fs')
  rmSync(dir, { recursive: true, force: true })
}
`,
    },
  },
} satisfies PackageTaskFile;
