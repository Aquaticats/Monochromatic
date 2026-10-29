/**
 In-container entrypoint for one catalog-tighten matrix scenario.
 
 Runs inside a podman container with the monorepo mounted read-only at `/repo`
 and a writable tmpfs at `/work`. Reads a {@link Scenario} from argv, writes the
 fixture workspace into `/work`, installs it with pnpm (via corepack) under the
 scenario's layout, provisions pnpm on PATH, applies the scenario's post-install
 mutation, runs catalog-tighten against the fixture, and asserts the tool
 tightens, reports a MISS or an UNDCL, or fails cleanly as the scenario expects.
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import spawn from 'nano-spawn';

import {
  buildRootPackageJson,
  buildWorkspaceYaml,
  CONSUMER_DIRS,
  consumerPackageJson,
  EXPECTED_TIGHTENED,
  FIXTURE_PACKAGE,
  FIXTURE_PNPM_ENV,
  type Scenario,
} from './combos.ts';
import {
  PNPM_BIN_DIR,
  TOOL_ENTRY,
  WORK_DIR,
} from './container-paths.ts';
import {
  applyMutation,
} from './mutations.ts';

//region Types

/**
 Outcome of running the tool: whether it exited zero, and its combined output.
 */
type ToolResult = {
  /**
   Whether the tool exited zero.
   */
  readonly ok: boolean;
  /**
   Combined stdout and stderr; the tighten line is on stdout, the MISS line on stderr.
   */
  readonly output: string;
};

//endregion Types

//region Setup steps

/**
 Writes the fixture workspace for `scenario` into {@link WORK_DIR}: the
 workspace file built by {@link buildWorkspaceYaml}, the root manifest pinned
 to `fixturePnpm` via {@link buildRootPackageJson}, and both consumer packages
 written via {@link consumerPackageJson}.
 
 @param scenario - scenario whose settings shape the workspace file
 
 @param fixturePnpm - `pnpm@<version>` spec the monorepo resolved, written into `packageManager`
 
 @example
 ```ts
 await writeFixture({ scenario: SCENARIOS[0], fixturePnpm: "pnpm\@11.11.0" });
 ```
 */
async function writeFixture(
  {
    scenario,
    fixturePnpm,
  }: {
    readonly scenario: Scenario;
    readonly fixturePnpm: string;
  },
): Promise<void> {
  await writeFile(
    join(
      WORK_DIR,
      'pnpm-workspace.yaml',
    ),
    buildWorkspaceYaml(scenario,),
  );
  await writeFile(
    join(
      WORK_DIR,
      'package.json',
    ),
    buildRootPackageJson(fixturePnpm,),
  );
  await Promise.all(CONSUMER_DIRS.map(async function writeConsumer(dir,): Promise<void> {
    /**
     Absolute consumer directory; created before its manifest is written.
     */
    const absoluteDir = join(
      WORK_DIR,
      dir,
    );
    await mkdir(
      absoluteDir,
      { recursive: true, },
    );
    await writeFile(
      join(
        absoluteDir,
        'package.json',
      ),
      consumerPackageJson(dir,),
    );
  },),);
}

/**
 Installs the fixture with `fixturePnpm` via corepack. Layout settings live in
 the workspace file, so this is a plain install; corepack caches pnpm under
 `HOME` (tmpfs) and the download prompt is disabled for non-interactive runs.
 
 @param fixturePnpm - `pnpm@<version>` spec the monorepo resolved
 
 @example
 ```ts
 await installFixture("pnpm\@11.11.0");
 ```
 */
async function installFixture(fixturePnpm: string,): Promise<void> {
  await spawn(
    'corepack',
    [
      fixturePnpm,
      'install',
      '--ignore-scripts',
      '--config.confirmModulesPurge=false',
    ],
    {
      cwd: WORK_DIR,
      env: {
        ...process.env,
        COREPACK_ENABLE_DOWNLOAD_PROMPT: '0',
        HOME: '/tmp',
      },
    },
  );
}

/**
 Installs the corepack `pnpm` shim into {@link PNPM_BIN_DIR}, so the tool's
 `pnpm config get modules-dir` finds `pnpm` on PATH under the read-only rootfs.
 The shim resolves the pinned version from the fixture's `packageManager`
 field, reusing the cached install offline.
 
 @example
 ```ts
 await enablePnpm();
 ```
 */
async function enablePnpm(): Promise<void> {
  await mkdir(
    PNPM_BIN_DIR,
    { recursive: true, },
  );
  await spawn(
    'corepack',
    [
      'enable',
      '--install-directory',
      PNPM_BIN_DIR,
      'pnpm',
    ],
    {
      env: {
        ...process.env,
        COREPACK_ENABLE_DOWNLOAD_PROMPT: '0',
        HOME: '/tmp',
      },
    },
  );
}

//endregion Setup steps

//region Run and assert

/**
 Runs catalog-tighten `--dry-run` from {@link TOOL_ENTRY} against the
 fixture, prepending the pnpm shim directory to PATH. Captures a non-zero
 exit as `ok: false` rather than throwing, so the caller can assert the
 error scenarios.
 
 @returns whether the tool exited zero, and its stdout
 
 @example
 ```ts
 const result = await runTool();
 ```
 */
async function runTool(): Promise<ToolResult> {
  /**
   Current container PATH value; may be unset.
   */
  const { PATH: rawPath, } = process.env;
  /**
   PATH defaulted to empty when unset.
   */
  const basePath = rawPath
    ?? '';
  /**
   PATH with the pnpm shim directory prepended, so the tool finds `pnpm`.
   */
  const toolPath = `${PNPM_BIN_DIR}:${basePath}`;
  try {
    /**
     Tool result on a zero exit; stdout carries the per-entry status lines.
     */
    const result = await spawn(
      'node',
      [
        TOOL_ENTRY,
        '--dry-run',
      ],
      {
        cwd: WORK_DIR,
        env: {
          ...process.env,
          PATH: toolPath,
        },
      },
    );
    return {
      ok: true,
      output: `${result.stdout}\n${result.stderr}`,
    };
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return {
      ok: false,
      output: '',
    };
  }
}

/**
 Asserts the tool result matches the scenario's expectation, checking the
 output against {@link EXPECTED_TIGHTENED} and the {@link FIXTURE_PACKAGE}
 MISS and UNDCL lines, throwing a labelled error on mismatch.
 
 @param scenario - scenario under test
 
 @param result - tool result to check
 
 @example
 ```ts
 assertOutcome({ scenario: SCENARIOS[0], result: { ok: true, stdout: '...' } });
 ```
 */
function assertOutcome(
  {
    scenario,
    result,
  }: {
    readonly scenario: Scenario;
    readonly result: ToolResult;
  },
): void {
  /**
   Tool exit status and combined output for this scenario.
   */
  const {
    ok,
    output,
  } = result;
  /**
   Whether the output reports a MISS for the fixture package (the MISS line is on stderr).
   */
  const reportedMiss = output.includes(`MISS  ${FIXTURE_PACKAGE}`,);
  /**
   Whether the output reports an UNDCL for the fixture package (store-present but undeclared; on stderr).
   */
  const reportedUndeclared = output.includes(`UNDCL ${FIXTURE_PACKAGE}`,);
  /**
   Whether the output reports the expected tightened line (on stdout).
   */
  const tightened = output.includes(EXPECTED_TIGHTENED,);

  if (scenario.expect
    === 'error') {
    if (ok)
      throw new Error(`[${scenario.label}] expected the tool to fail, but it exited 0:\n${output}`,);
    return;
  }
  if (scenario.expect
    === 'miss') {
    if (!ok)
      throw new Error(`[${scenario.label}] tool failed unexpectedly:\n${output}`,);
    if ((!reportedMiss) || tightened)
      throw new Error(`[${scenario.label}] expected a MISS, got:\n${output}`,);
    return;
  }
  if (scenario.expect
    === 'undeclared') {
    if (!ok)
      throw new Error(`[${scenario.label}] tool failed unexpectedly:\n${output}`,);
    if ((!reportedUndeclared) || tightened)
      throw new Error(`[${scenario.label}] expected an UNDCL, got:\n${output}`,);
    return;
  }
  if ((!ok) || (!tightened))
    throw new Error(`[${scenario.label}] expected "${EXPECTED_TIGHTENED}", got:\n${output}`,);
}

//endregion Run and assert

//region Main

/**
 argv index of the scenario JSON the orchestrator passes.
 */
const SCENARIO_ARG_INDEX = 2;

/**
 Raw scenario JSON argument; absent only on misuse.
 */
const scenarioJson = process.argv[SCENARIO_ARG_INDEX];
if (scenarioJson === undefined)
  throw new Error('Usage: in-container.ts <scenario-json>',);

/**
 Scenario to run, deserialised from the orchestrator's argument.
 */
// oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the orchestrator serialises a Scenario it owns
const scenario = JSON.parse(scenarioJson,) as Scenario;

/**
 Monorepo `pnpm@<version>` spec the orchestrator resolved and passed in, so the
 fixture installs with the pnpm the repo actually runs.
 */
const fixturePnpm = process.env[FIXTURE_PNPM_ENV];
if (fixturePnpm === undefined) {
  throw new Error(
    `${FIXTURE_PNPM_ENV} not set; the orchestrator must pass the monorepo pnpm spec into the container`,
  );
}

await writeFixture({
  scenario,
  fixturePnpm,
},);
await installFixture(fixturePnpm,);
await enablePnpm();
await applyMutation(scenario,);

/**
 Tool outcome to assert against the scenario's expectation.
 */
const result = await runTool();
assertOutcome({
  scenario,
  result,
},);
console.info(`[${scenario.label}] PASS (${scenario.expect})`,);



//endregion Main
