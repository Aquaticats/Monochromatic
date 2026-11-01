/**
 Package tasks for `@monochromatic-dev/module-toml-edit-fuzz`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'lint': { extends: 'lint', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'lint:types': { extends: 'lint:types', },
  'test:unit': { extends: 'test:unit', },
  'test:scope': {
    description: 'Exercise the exact pre-install workflow launcher against disposable Git repositories',
    run: 'node src/ci-scope.unit.test.ts',
  },
  'lint:scope': {
    description: 'Lint the dependency-free scope entry and its subprocess regression tests',
    run: [
      { shell: 'node --input-type=module-typescript -e', run: 'ensureOxlintConfig()', },
      {
        shell: 'node --input-type=module-typescript -e',
        run: 'runWorkspaceNode(\'package/monorepo-task/core-exec\', \'oxlint-wrapper\', [\'src/ci-scope.ts\', \'src/ci-scope-git.ts\', \'src/ci-scope-error.ts\', \'src/ci-scope.unit.test.ts\'])',
      },
    ],
  },
  'fuzz': {
    description: 'Build module-toml-edit, then run *.property.unit.test.ts as a time-budgeted fast-check campaign under node',
    usage: 'flag "--budget <ms>" help="Per-property time budget in milliseconds (default 60000)"',
    run: {
      shell: 'node --input-type=module-typescript -e',
      run: `
const budget = (process.env.usage_budget ?? '60000').trim()
runTaskCli('//package/toml-edit/core:build')
const { glob } = await import('node:fs/promises')
const files = []
for await (const file of glob('src/**/*.property.unit.test.ts')) { files.push(file) }
console.log('Fuzz campaign over ' + files.length + ' property files at ' + budget + 'ms each')
for (const file of files) {
  console.log('fuzzing ' + file)
  execFileSync(process.execPath, [file], { stdio: 'inherit', env: { ...process.env, TOML_EDIT_FUZZ_BUDGET_MS: budget } })
}
`,
    },
  },
  'fuzz:coverage': {
    description: 'Deterministic V8 line-coverage reachability gate over the module-toml-edit runtime src; --write refreezes the baseline',
    usage: 'flag "--write" help="Refreeze the committed baseline from this run instead of checking against it"',
    run: {
      shell: 'node --input-type=module-typescript -e',
      run: `
const root = join(process.env.TASK_REPO_ROOT, 'package', 'toml-edit', 'core.fuzz')
const mode = (process.env.usage_write === 'true') ? 'write' : 'check'
const dir = mkdtempSync(join(scratchBase(), 'toml-edit-cov-'))
try {
  execFileSync(process.execPath, [join(root, 'src', 'coverage-driver.ts')], { stdio: 'inherit', env: { ...process.env, MONOCHROMATIC_WARN: 'false', NODE_V8_COVERAGE: dir } })
  execFileSync(process.execPath, [join(root, 'src', 'coverage-report.ts'), mode, dir, join(root, 'coverage-baseline.json')], { stdio: 'inherit' })
} finally {
  const { rmSync } = await import('node:fs')
  rmSync(dir, { recursive: true, force: true })
}
`,
    },
  },
} satisfies PackageTaskFile;
