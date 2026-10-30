import { realpathSync } from 'node:fs';
import { resolve } from 'node:path';

import {
  createFakePodman,
  type FakePodman,
  runScript,
  type ScriptRun,
} from './fake-podman-fixture.ts';

/**
 Repository package directory, three levels above this package's `src`.
 */
const packageRoot = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
);

/**
 One run of a product's mutation script whose observable behavior is pinned.
 */
export type GoldenScenario = {
  readonly name: string;
  readonly script: string;
  readonly args: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
};

/**
 What a scenario produced, with the paths and random suffixes that differ between runs replaced.
 */
export type GoldenOutcome = {
  readonly status: number;
  readonly errorMessage: string;
  readonly stdout: string;
  readonly calls: readonly (readonly string[])[];
  readonly containerfiles: readonly string[];
  readonly evidenceFiles: Readonly<Record<string, string>>;
};

/**
 Absolute path of a product script.

 @param relative - path below the repository's `package` directory
 @returns the absolute path
 @example
 ```ts
 productScript('git-policy/cli/bin/mutate-native-container.ts');
 ```
 */
function productScript(relative: string): string {
  return resolve(
    packageRoot,
    relative,
  );
}

/**
 Environment of the native git-policy campaign scenarios: skip the planted controls and name the image tag.
 */
const nativeEnvironment: Readonly<Record<string, string>> = {
  NATIVE_MUTATION_SKIP_PLANTED: '1',
  GIT_POLICY_NATIVE_IMAGE_TAG: 'golden',
};

/**
 Environment of the linter campaign scenarios: name the image tag.
 */
const linterEnvironment: Readonly<Record<string, string>> = { MONOCHROMATIC_LINT_IMAGE_TAG: 'golden' };

/**
 Native wrapper campaign scenarios, run against the script at the given path.
 */
const nativeArgumentSets: readonly (readonly [string, readonly string[], Readonly<Record<string, string>>])[] = [
  ['default', [], {}],
  ['scoped-files-and-timeout', ['--file', 'src/native/lock.rs', '--file', 'src/native/push_env.rs', '--timeout', '60'], {}],
  ['examine', ['--examine', 'replace foo|bar::baz', '--file', 'src/native/a.rs'], {}],
  ['container-limits', ['--memory', '4g', '--cpus', '3', '--pids-limit', '64', '--file', 'src/native/x.rs'], {}],
  ['baseline-skip', ['--file', 'src/native/a.rs'], { NATIVE_MUTATION_BASELINE_SKIP: 'module::tests::name' }],
  ['reject-timeout-zero', ['--timeout', '0'], {}],
  ['reject-timeout-large', ['--timeout', '3601'], {}],
  ['reject-timeout-fraction', ['--timeout', '1.5'], {}],
  ['reject-timeout-missing', ['--timeout'], {}],
  ['reject-file-outside-native', ['--file', 'src/other/x.rs'], {}],
  ['reject-file-space', ['--file', 'src/native/a b.rs'], {}],
  ['reject-file-missing-glob', ['--file'], {}],
  ['reject-unknown-option', ['--bogus', '1'], {}],
  ['reject-examine-semicolon', ['--examine', 'a;b'], {}],
  ['reject-memory-zero', ['--memory', '0g'], {}],
  ['reject-memory-unit', ['--memory', '2x'], {}],
  ['reject-cpus-zero', ['--cpus', '0'], {}],
  ['reject-cpus-decimals', ['--cpus', '1.234'], {}],
  ['reject-pids-zero', ['--pids-limit', '0'], {}],
  ['reject-baseline-skip', ['--file', 'src/native/a.rs'], { NATIVE_MUTATION_BASELINE_SKIP: 'Bad' }],
];

/**
 Linter campaign scenarios.
 */
const linterArgumentSets: readonly (readonly [string, readonly string[], Readonly<Record<string, string>>])[] = [
  ['default', [], {}],
  ['rust-style', ['--rust-style'], {}],
  ['markdown', ['--markdown'], {}],
  ['markdown-parent', ['--markdown-parent'], {}],
  ['processors', ['--processors'], {}],
  ['inferred-constants', ['--inferred-constants'], {}],
  ['executable', ['--executable'], {}],
  ['core', ['--core'], {}],
  ['semantic', ['--semantic'], {}],
  ['shard', ['--markdown', '--shard', '1/3'], {}],
  ['examine', ['--core', '--re', 'Foo::bar'], {}],
  ['list-core', ['--list', '--core'], {}],
  ['list-unscoped', ['--list'], {}],
  ['coverage', ['--coverage'], {}],
  ['reject-unknown-scope', ['--bogus'], {}],
  ['reject-shard', ['--markdown', '--shard', '3/3'], {}],
  ['reject-empty-re', ['--core', '--re', ''], {}],
  ['reject-re-on-reselecting-scope', ['--markdown-parent', '--re', 'x'], {}],
  ['reject-two-scopes', ['--core', '--markdown'], {}],
  ['reject-coverage-extra', ['--coverage', '--core'], {}],
];

/**
 Scanner scenarios that fail on their options. A scanner run that gets past its options vendors the real dependencies
 with cargo, which a stand-in cannot answer, so only these are pinned.
 */
const scannerArgumentSets: readonly (readonly [string, readonly string[]])[] = [
  ['reject-unknown-scope', ['--bogus']],
  ['reject-two-scopes', ['--startup', '--pathname']],
];

/**
 Every pinned scenario of the products whose campaign scripts are being shared.

 @returns the scenarios, native wrapper first
 @example
 ```ts
 goldenScenarios();
 ```
 */
export function goldenScenarios(): readonly GoldenScenario[] {
  return [
    ...nativeArgumentSets.map(function nativeScenario([name, args, environment]) {
      return {
        name: `git-policy-${name}`,
        script: productScript('git-policy/cli/bin/mutate-native-container.ts'),
        args,
        environment: {
          ...nativeEnvironment,
          ...environment,
        },
      };
    }),
    ...linterArgumentSets.map(function linterScenario([name, args, environment]) {
      return {
        name: `linter-${name}`,
        script: productScript('linter/monochromatic-lint/bin/mutate-container.mjs'),
        args,
        environment: {
          ...linterEnvironment,
          ...environment,
        },
      };
    }),
    ...scannerArgumentSets.map(function scannerScenario([name, args]) {
      return {
        name: `scanner-${name}`,
        script: productScript('forbidden-strings/cli/bin/mutate-container.mjs'),
        args,
        environment: {},
      };
    }),
  ];
}

/**
 Replace what differs between runs: absolute paths, random directory suffixes.

 @param text - text from a run
 @param fake - fixture of the run
 @returns the text with stable placeholders
 @example
 ```ts
 normalizeText({ text, fake });
 ```
 */
function normalizeText({
  text,
  fake,
}: {
  readonly text: string;
  readonly fake: FakePodman;
}): string {
  return text
    .replaceAll(
      realpathSync(fake.workingDirectory),
      '<WORK>',
    )
    .replaceAll(
      fake.workingDirectory,
      '<WORK>',
    )
    .replaceAll(
      realpathSync(fake.directory),
      '<FAKE>',
    )
    .replaceAll(
      fake.directory,
      '<FAKE>',
    )
    .replaceAll(
      packageRoot,
      '<PACKAGE>',
    )
    .replaceAll(
      /(verification\/[a-z-]+-)[A-Za-z0-9]{6}\b/gu,
      '$1<ID>',
    )
    .replaceAll(
      /((?:cli-git-native-mutation|monochromatic-lint-mutation|scanner-snapshot)-)[A-Za-z0-9]{6}\b/gu,
      '$1<ID>',
    );
}

/**
 Reduce a run to the outcome that is pinned.

 @param run - what the script did
 @param fake - fixture of the run
 @returns the normalized outcome; a failure keeps only its message, without the error class name or stack
 @example
 ```ts
 normalizeRun({ run, fake });
 ```
 */
export function normalizeRun({
  run,
  fake,
}: {
  readonly run: ScriptRun;
  readonly fake: FakePodman;
}): GoldenOutcome {
  /**
   Line of the error message, without the class name.
   */
  const errorLine = /^\w*Error: (?<message>.*)$/mu.exec(run.stderr)?.groups?.['message'] ?? '';
  /**
   The outcome before placeholders are applied.
   */
  const raw: GoldenOutcome = {
    status: run.status,
    errorMessage: errorLine,
    stdout: run.stdout,
    calls: run.calls,
    containerfiles: run.containerfiles,
    evidenceFiles: run.evidenceFiles,
  };
  return JSON.parse(normalizeText({
    text: JSON.stringify(raw),
    fake,
  })) as GoldenOutcome;
}

/**
 Run one scenario in a fresh fixture and return its normalized outcome.

 @param scenario - the scenario
 @returns the normalized outcome
 @example
 ```ts
 await runScenario(goldenScenarios()[0]);
 ```
 */
export async function runScenario(scenario: GoldenScenario): Promise<GoldenOutcome> {
  await using fake = await createFakePodman();
  /**
   What the script did.
   */
  const run = await runScript({
    fake,
    script: scenario.script,
    args: scenario.args,
    extraEnvironment: scenario.environment,
  });
  return normalizeRun({
    run,
    fake,
  });
}
