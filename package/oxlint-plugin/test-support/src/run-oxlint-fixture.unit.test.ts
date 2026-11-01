import {
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type OxlintFixMode,
  OxlintOutputParseError,
  type OxlintRuleDiagnostic,
  runOxlintFixture,
} from '../dist/final/node/index.mjs';
import { PROBE_CREATE_ONCE_FAILURE_TEXT, } from './harness-probe-create-once-plugin.test-fixture.ts';
import {
  PROBE_FAILURE_TEXT,
  PROBE_FIX_MARKER,
  PROBE_THROW_VARIABLE,
} from './harness-probe-plugin.test-fixture.ts';

//region Helpers

/** Plugin that reports, stays quiet, throws, or fixes, depending on which of its rules a config enables. */
const PROBE_PLUGIN = resolve(
  import.meta.dirname,
  'harness-probe-plugin.test-fixture.ts',
);

/** Plugin whose only rule throws while oxlint sets it up, which makes oxlint print text instead of JSON. */
const CREATE_ONCE_PROBE_PLUGIN = resolve(
  import.meta.dirname,
  'harness-probe-create-once-plugin.test-fixture.ts',
);

/** Text of the source every probe project holds, which no rule has any reason to report. */
const UNFIXED_SOURCE = 'export const answer = 42;\n';

/** Disposable project holding one valid source and a config enabling the named probe rules. */
type ProbeProject = {
  /** Absolute path of the config. */
  readonly configPath: string;
  /** Absolute path of the source, which no rule has any reason to report. */
  readonly sourcePath: string;
  /** Removes the project. */
  readonly [Symbol.dispose]: () => void;
};

/** Plugin file and enabled rules of a probe project. */
type ProbeProjectParams = {
  /** Absolute path of the plugin file the config loads. */
  readonly plugin: string;
  /** Rule ids to enable, each in `plugin-name/rule-name` form. */
  readonly ruleIds: readonly string[];
};

/**
 Writes a project whose config loads one plugin file and enables the given rules.

 @example
 ```ts
 using project = writeProbeProject({ plugin: PROBE_PLUGIN, ruleIds: ['harness-probe/always-throws',], },);
 ```
 */
function writeProbeProject(params: ProbeProjectParams,): ProbeProject {
  /** Plugin file and rule ids to write into the config. */
  const {
    plugin,
    ruleIds,
  } = params;
  /** Unique temp directory. */
  const directory = mkdtempSync(join(
    scratchRoot(),
    'oxlint-harness-probe-',
  ),);
  /** Config path. */
  const configPath = join(
    directory,
    '.oxlintrc.json',
  );
  /** Source path. */
  const sourcePath = join(
    directory,
    'valid.ts',
  );
  writeFileSync(
    configPath,
    JSON.stringify({
      jsPlugins: [plugin,],
      rules: Object.fromEntries(ruleIds.map(function enable(ruleId,): [string, string,] {
        return [ruleId, 'error',];
      },),),
    },),
  );
  writeFileSync(
    sourcePath,
    UNFIXED_SOURCE,
  );
  return {
    configPath,
    sourcePath,
    [Symbol.dispose]: function cleanup(): void {
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
 Writes a project whose config enables the named rules of the probe plugin.

 @example
 ```ts
 using project = probeProject(['always-throws',],);
 ```
 */
function probeProject(rules: readonly string[],): ProbeProject {
  return writeProbeProject({
    plugin: PROBE_PLUGIN,
    ruleIds: rules.map(function qualify(rule,): string {
      return `harness-probe/${rule}`;
    },),
  },);
}

/**
 Runs the fixture runner over a project, keeping the failure when it fails.

 @example
 ```ts
 const outcome = await attempt(project,);
 const fixing = await attempt(project, 'fixes',);
 const asked = await attempt(project, undefined, { [PROBE_THROW_VARIABLE]: '1', },);
 ```
 */
async function attempt(
  project: ProbeProject,
  fix?: OxlintFixMode,
  environment?: Readonly<Record<string, string>>,
): Promise<{ readonly diagnostics: readonly OxlintRuleDiagnostic[]; readonly failure: unknown; }> {
  try {
    return {
      diagnostics: await runOxlintFixture({
        codePrefix: 'harness-probe(',
        configFlag: '--config',
        fixtureConfig: project.configPath,
        target: project.sourcePath,
        threads: 1,
        ...fix === undefined ? {} : { fix, },
        ...environment === undefined ? {} : { environment, },
      },),
      failure: undefined,
    };
  }
  catch (error: unknown) {
    return { diagnostics: [], failure: error, };
  }
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: runOxlintFixture.name,
      children: [
        it({
          name: 'returns the diagnostics of a plugin that reports',
          fn: async () => {
            using project = probeProject(['reports-program',],);
            const outcome = await attempt(project,);

            expect(outcome.failure,).toBeUndefined();
            expect(outcome.diagnostics.map(function codeOf(diagnostic,): string {
              return diagnostic.code;
            },),).toEqual(['harness-probe(reports-program)',],);
          },
        },),
        it({
          name: 'returns no diagnostics for a plugin that stays quiet on a valid source',
          fn: async () => {
            using project = probeProject(['reports-nothing',],);
            const outcome = await attempt(project,);

            expect(outcome.failure,).toBeUndefined();
            expect(outcome.diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'fails when the plugin throws on a valid source, naming the plugin error',
          fn: async () => {
            using project = probeProject(['always-throws',],);
            const outcome = await attempt(project,);

            expect(outcome.failure,).toBeInstanceOf(Error,);
            expect(String(outcome.failure,),).toContain(PROBE_FAILURE_TEXT,);
            expect(String(outcome.failure,),).toContain(project.sourcePath,);
          },
        },),
        it({
          name: 'fails when the plugin throws even though another rule of it reported',
          fn: async () => {
            using project = probeProject(['reports-program', 'always-throws',],);
            const outcome = await attempt(project,);

            expect(outcome.failure,).toBeInstanceOf(Error,);
            expect(String(outcome.failure,),).toContain(PROBE_FAILURE_TEXT,);
          },
        },),
        it({
          name: 'runs oxlint with the environment it is given, so a plugin can see it',
          fn: async () => {
            using project = probeProject(['throws-when-asked',],);
            const outcome = await attempt(
              project,
              undefined,
              { [PROBE_THROW_VARIABLE]: '1', },
            );

            expect(outcome.failure,).toBeInstanceOf(Error,);
            expect(String(outcome.failure,),).toContain(PROBE_FAILURE_TEXT,);
          },
        },),
        it({
          name: 'runs a plugin that checks the environment quietly when the variable is not given',
          fn: async () => {
            using project = probeProject(['throws-when-asked',],);
            const outcome = await attempt(project,);

            expect(outcome.failure,).toBeUndefined();
            expect(outcome.diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'passes the environment to a fix run as well',
          fn: async () => {
            using project = probeProject(['throws-when-asked',],);
            const outcome = await attempt(
              project,
              'fixes',
              { [PROBE_THROW_VARIABLE]: '1', },
            );

            expect(outcome.failure,).toBeInstanceOf(Error,);
            expect(String(outcome.failure,),).toContain(PROBE_FAILURE_TEXT,);
          },
        },),
        it({
          name: 'leaves the source unchanged when no fix mode is asked, though a rule offers a fix',
          fn: async () => {
            using project = probeProject(['fixes-program',],);
            const outcome = await attempt(project,);

            expect(outcome.failure,).toBeUndefined();
            expect(outcome.diagnostics.map(function codeOf(diagnostic,): string {
              return diagnostic.code;
            },),).toEqual(['harness-probe(fixes-program)',],);
            expect(readFileSync(
              project.sourcePath,
              'utf8',
            ),).toBe(UNFIXED_SOURCE,);
          },
        },),
        it({
          name: 'applies the fixes of a plugin that works in a fix run and reports nothing left over',
          fn: async () => {
            using project = probeProject(['fixes-program',],);
            const outcome = await attempt(
              project,
              'fixes',
            );

            expect(outcome.failure,).toBeUndefined();
            expect(outcome.diagnostics,).toEqual([],);
            expect(readFileSync(
              project.sourcePath,
              'utf8',
            ),).toBe(`${PROBE_FIX_MARKER}${UNFIXED_SOURCE}`,);
          },
        },),
        it({
          name: 'fails a fix run when the plugin throws on a valid source, naming the plugin error',
          fn: async () => {
            using project = probeProject(['always-throws',],);
            const outcome = await attempt(
              project,
              'fixes',
            );

            expect(outcome.failure,).toBeInstanceOf(Error,);
            expect(String(outcome.failure,),).toContain(PROBE_FAILURE_TEXT,);
            expect(String(outcome.failure,),).toContain(project.sourcePath,);
          },
        },),
        it({
          name: 'fails a fix run when the plugin throws even though another rule of it fixed',
          fn: async () => {
            using project = probeProject(['fixes-program', 'always-throws',],);
            const outcome = await attempt(
              project,
              'fixes-and-suggestions',
            );

            expect(outcome.failure,).toBeInstanceOf(Error,);
            expect(String(outcome.failure,),).toContain(PROBE_FAILURE_TEXT,);
          },
        },),
        it({
          name: 'fails with the oxlint output and the command when oxlint prints text instead of JSON',
          fn: async () => {
            using project = writeProbeProject({
              plugin: CREATE_ONCE_PROBE_PLUGIN,
              ruleIds: ['harness-probe-create-once/create-once-throws',],
            },);
            const outcome = await attempt(project,);

            expect(outcome.failure,).toBeInstanceOf(OxlintOutputParseError,);
            expect(Error.isError(outcome.failure,) ? outcome.failure.cause : undefined,).toBeInstanceOf(SyntaxError,);
            expect(String(outcome.failure,),).toContain(PROBE_CREATE_ONCE_FAILURE_TEXT,);
            expect(String(outcome.failure,),).toContain(project.sourcePath,);
            expect(String(outcome.failure,),).toContain(project.configPath,);
          },
        },),
      ],
    },),
  ],
},);
