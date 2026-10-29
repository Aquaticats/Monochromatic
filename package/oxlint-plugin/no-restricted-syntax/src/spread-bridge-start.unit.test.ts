import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  fixtureConfigPath,
  OXLINT_PLUGIN_TEST_ROOT as ROOT,
  runOxlintFixture,
} from '@monochromatic-dev/oxlint-plugin-test-support/ts';
import spawn from 'nano-spawn';

//region Helpers

/** Observer of native TypeScript child spawns, loaded before the code under test. */
const OBSERVE_NATIVE_SPAWN = pathToFileURL(resolve(
  import.meta.dirname,
  'observe-native-api-spawn.test-fixture.ts',
),).href;

/** Built plugin, the artifact Oxlint loads. */
const BUILT_PLUGIN = pathToFileURL(resolve(
  import.meta.dirname,
  '../dist/final/node/index.mjs',
),).href;

/** Longest a spawned process may run before the case gives up on it. */
const PROCESS_TIMEOUT_MILLISECONDS = 120_000;

/** Disposable typed or untyped project holding one source. */
type ScratchProject = {
  /** Absolute path of the source. */
  readonly filePath: string;
  /** Removes the project. */
  readonly [Symbol.dispose]: () => void;
};

/**
 Writes one source into a fresh temp directory, with a tsconfig when the bridge is meant to type it.

 @example
 ```ts
 using project = scratchProject({ source, typed: true });
 ```
 */
function scratchProject(
  {
    source,
    typed,
  }: {
    readonly source: string;
    readonly typed: boolean;
  },
): ScratchProject {
  /** Unique temp directory. */
  const directory = mkdtempSync(join(
    scratchRoot(),
    'oxlint-spread-bridge-start-',
  ),);
  writeFileSync(
    join(
      directory,
      'case.ts',
    ),
    source,
  );
  if (typed) {
    writeFileSync(
      join(
        directory,
        'tsconfig.json',
      ),
      JSON.stringify({
        compilerOptions: {
          lib: ['ESNext',],
          target: 'esnext',
          module: 'preserve',
          strict: true,
          noEmit: true,
          types: [],
        },
        files: ['case.ts',],
      },),
    );
  }
  return {
    filePath: join(
      directory,
      'case.ts',
    ),
    [Symbol.dispose]: function removeProject(): void {
      rmSync(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Lints a source with only the prefer-spread rule at Oxlint's own worker count, which is what an editor
 or a bare `oxlint` run uses, and returns the messages the rule reported.

 @example
 ```ts
 await preferSpreadMessages(project.filePath,);
 ```
 */
async function preferSpreadMessages(target: string,): Promise<readonly string[]> {
  /** Diagnostics of the plugin; the runner fails the case when the plugin throws. */
  const diagnostics = await runOxlintFixture({
    codePrefix: 'no-restricted-syntax(',
    configFlag: '-c',
    fixtureConfig: fixtureConfigPath({
      fixturePackageName: 'oxlint-no-restricted-syntax',
      fileName: '.oxlintrc.prefer-spread.fixture.json',
    },),
    target,
  },);
  return diagnostics.map(function messageOf(diagnostic,): string {
    return diagnostic.message;
  },);
}

//endregion Helpers

await describe({
  name: 'spread rules and the TypeScript bridge at the default worker count',
  concurrency: 1,
  children: [
    it({
      name: 'starts the native TypeScript child when the plugin loads, before any file is linted',
      fn: async () => {
        /** Process that only loads the built plugin, so any spawn it makes happened at load. */
        const loaded = await spawn(
          process.execPath,
          [
            '--import',
            OBSERVE_NATIVE_SPAWN,
            '--input-type=module',
            '--eval',
            `await import(${JSON.stringify(BUILT_PLUGIN,)},);`,
          ],
          {
            cwd: ROOT,
            signal: AbortSignal.timeout(PROCESS_TIMEOUT_MILLISECONDS,),
          },
        );

        expect(loaded.stderr,).toContain('started 1 native spawns, failed 0',);
      },
      timeout: PROCESS_TIMEOUT_MILLISECONDS,
    },),
    it({
      name: 'types a source of a configured project, so a slice of an array is reported as a plain copy',
      fn: async () => {
        using project = scratchProject({
          source: 'declare const toppings: readonly string[];\nexport const copy = toppings.slice();\n',
          typed: true,
        },);

        /** Messages for the one slice. */
        const messages = await preferSpreadMessages(project.filePath,);

        expect(messages,).toHaveLength(1,);
        expect(messages[0],).toContain('only copies it; write',);
        expect(messages[0],).not.toContain('No type information is available here',);
      },
      timeout: PROCESS_TIMEOUT_MILLISECONDS,
    },),
    it({
      name: 'reports a slice as ambiguous in a source of no configured project',
      fn: async () => {
        using project = scratchProject({
          source: 'declare const toppings: readonly string[];\nexport const copy = toppings.slice();\n',
          typed: false,
        },);
        /** Messages for the one slice. */
        const messages = await preferSpreadMessages(project.filePath,);

        expect(messages,).toHaveLength(1,);
        expect(messages[0],).toContain('No type information is available here',);
      },
      timeout: PROCESS_TIMEOUT_MILLISECONDS,
    },),
  ],
},);
