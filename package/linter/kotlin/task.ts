/**
 Package tasks for `@monochromatic-dev/linter-kotlin`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'build': {
    description: 'Compile the Kotlin linter jar',
    run: './gradlew jar --no-daemon --console=plain',
  },
  'publish:bundle': {
    description: 'Build the signed Central bundle',
    run: './gradlew centralBundle --no-daemon --console=plain',
  },
  'verify:publication': {
    description: 'Build the signed Central bundle and resolve it from a disposable consumer project',
    depends: ['publish:bundle', 'verify:publication:consumer',],
  },
  'verify:publication:consumer': {
    description: 'Resolve the built bundle from a disposable consumer project',
    run: {
      shell: 'node --input-type=module-typescript -e',
      run: `
const packageRoot = join(process.env.TASK_REPO_ROOT, 'package', 'linter', 'kotlin')
const properties = readFileSync(join(packageRoot, 'gradle.properties'), 'utf8')
const versionLine = properties.split('\\n').find((line) => line.startsWith('version='))
if (versionLine === undefined) { throw new Error('gradle.properties has no version entry') }
const version = versionLine.slice('version='.length).trim()
const consumerRoot = join(packageRoot, 'build', 'publication-consumer')
const repositoryRoot = join(packageRoot, 'build', 'central-staging')
const { rmSync, mkdirSync, writeFileSync } = await import('node:fs')
rmSync(consumerRoot, { force: true, recursive: true })
mkdirSync(consumerRoot, { recursive: true })
writeFileSync(join(consumerRoot, 'settings.gradle'), "rootProject.name = 'publication-consumer'\\n")
writeFileSync(join(consumerRoot, 'build.gradle'), [
  "plugins { id 'java' }",
  'repositories {',
  '  maven { url = uri(' + JSON.stringify(repositoryRoot) + ') }',
  '  mavenCentral()',
  '}',
  'dependencies {',
  "  implementation 'com.monochromatic.linter:kotlin:" + version + "'",
  '}',
].join('\\n'))
execFileSync('./gradlew', ['--project-dir', consumerRoot, 'compileJava', '--no-daemon', '--console=plain'], { stdio: 'inherit', cwd: packageRoot })
`,
    },
  },
  'lint': {
    description: 'Compile Kotlin main and test sources',
    run: './gradlew compileKotlin compileTestKotlin --no-daemon --console=plain',
  },
  'test': {
    description: 'Run the Kotlin linter test suite',
    run: './gradlew test --no-daemon --console=plain',
  },
  'lint:detekt': {
    description: 'Run detekt over the Kotlin sources',
    run: './gradlew detektCheck -PdetektInput=src/main/kotlin --no-daemon --console=plain',
  },
} satisfies PackageTaskFile;
