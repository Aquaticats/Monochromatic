/**
 Gated disposable-host verification for the Qwen Token Plan Individual
 override.

 Boots a real pi host in a throwaway home,
 loads the probe extension before the built one,
 and asserts the override serves the checked-in live set where the builtin
 catalog serves its stale list.
 A second throwaway home runs `pi --list-models` with a fixture credential,
 which measures the same override at the user boundary.
 Both homes guard their agent files byte-for-byte,
 so a refresh that writes pi's model stores fails here.
 Gated by `QWEN_TOKEN_PLAN_INDIVIDUAL_VERIFY_HOST=1`
 so ordinary runs never spawn a host.

 The assertion machinery lives in `verify-host-assertions.ts`
 and the expectation reads in `verify-host-expectations.ts`,
 so each file stays inside the line budget.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  qwenTokenPlanIndividualListingSnapshot,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import {
  createDisposableHome,
  GUARDED_AGENT_FILES,
  piCliPath,
  readSnapshots,
  removeDisposableHome,
  runDisposableHost,
  TARGET_PROVIDER,
} from './disposable-host.ts';
import {
  assertSnapshots,
  check,
  LIVE_ONLY_FLASH,
  LIVE_ONLY_PREVIEW,
  sameIdSet,
  type Checks,
} from './verify-host-assertions.ts';
import {
  expectedRows,
  parseListedIds,
  readBuiltinFixture,
} from './verify-host-expectations.ts';

//region Constants

/**
 Environment variable that arms this verification.
 */
const GATE_VARIABLE = 'QWEN_TOKEN_PLAN_INDIVIDUAL_VERIFY_HOST';

/**
 Character count of stderr quoted when the host reports no snapshots.
 */
const STDERR_TAIL_CHARS = 1_200;

//endregion Constants

//region Host runs

/**
 Environment the disposable hosts see:
 no credential names,
 no telemetry,
 and only the throwaway home on disk.

 @param home - throwaway home this run owns

 @param agentDir - agent directory inside the throwaway home

 @returns the child environment map
 */
function hostEnvironment(
  {
    home,
    agentDir,
  }: {
    readonly home: string;
    readonly agentDir: string;
  },
): NodeJS.ProcessEnv {
  return {
    PATH: process.env
      .PATH
      ?? '',
    HOME: home,
    PI_CODING_AGENT_DIR: agentDir,
    PI_OFFLINE: '1',
    PI_TELEMETRY: '0',
    PI_SKIP_VERSION_CHECK: '1',
  };
}

/**
 Read one guarded agent file after the run,
 reporting the read failure text in place of the bytes when it is gone.

 @param path - guarded file to read

 @returns the file's content, or the failure text naming why it is unreadable
 */
async function readGuardedContent({
  path,
}: {
  readonly path: string;
}): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  } catch (error) {
    return `<absent: ${caughtValueText(error,)}>`;
  }
}

//endregion Host runs

//region Verification

/**
 Run the hosts and assert what the override served.

 @returns one line per assertion that passed

 @throws listing every assertion that failed, so one run reports the whole picture

 @example
 ```typescript
 console.log(await verifyInDisposableHost());
 ```
 */
async function verifyInDisposableHost(): Promise<string[]> {
  /**
   Throwaway home holding the RPC run.
   */
  const rpcHost = await createDisposableHome();
  /**
   Throwaway home holding the `--list-models` run.
   */
  const listHost = await createDisposableHome();
  /**
   Disposal that removes both throwaway homes however this run ends.
   */
  await using cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await removeDisposableHome({ root: rpcHost.root, },);
      await removeDisposableHome({ root: listHost.root, },);
    },
  };
  void cleanup;
  /**
   Probe source, loaded before the built extension so it sees both states.
   */
  const probe = fileURLToPath(new URL(
    'verify-probe.ts',
    import.meta.url,
  ),);
  /**
   Built artifact pi actually loads.
   */
  const extension = fileURLToPath(new URL(
    '../dist/final/node/index.mjs',
    import.meta.url,
  ),);
  /**
   RPC host run, which exits on its own once stdin closes.
   */
  const rpcRun = await runDisposableHost({
    args: [
      piCliPath(),
      '--mode',
      'rpc',
      '--no-extensions',
      '--extension',
      probe,
      '--extension',
      extension,
    ],
    cwd: rpcHost.root,
    env: hostEnvironment({
      home: rpcHost.home,
      agentDir: rpcHost.agentDir,
    },),
  },);
  /**
   Listing run at the user boundary, against the same built extension.
   */
  const listRun = await runDisposableHost({
    args: [
      piCliPath(),
      '--no-extensions',
      '--extension',
      extension,
      '--list-models',
      TARGET_PROVIDER,
    ],
    cwd: listHost.root,
    env: hostEnvironment({
      home: listHost.home,
      agentDir: listHost.agentDir,
    },),
  },);
  /**
   Snapshots the probe reported.
   */
  const snapshots = readSnapshots({ stderr: rpcRun.stderr, },);
  /**
   Snapshot taken before the session-start refresh ran.
   */
  const before = snapshots.find(function isBeforePhase(snapshot,) {
    return snapshot.phase === 'session_start';
  },);
  /**
   Snapshot taken after the session-start refresh ran.
   */
  const after = snapshots.find(function isAfterPhase(snapshot,) {
    return snapshot.phase === 'resources_discover';
  },);
  if ((before === undefined) || (after === undefined)) {
    throw new Error(
      `host reported ${String(snapshots.length,)} probe snapshots, needed session_start and resources_discover; stderr tail: ${rpcRun.stderr
        .slice(-STDERR_TAIL_CHARS,)}`,
    );
  }
  /**
   Assertion outcomes for this run.
   */
  const checks: Checks = {
    passes: [],
    failures: [],
  };
  /**
   Projected rows the checked-in listing earns.
   */
  const expected = expectedRows();
  /**
   Pinned builtin rows the stale-only comparison reads.
   */
  const fixture = await readBuiltinFixture();
  /**
   Ids the checked-in listing maps to a serviceable row.
   */
  const expectedIds = expected
    .map(function toId(row,) {
      return row.id;
    },);
  /**
   Ids the pinned builtin rows serve that the listing does not map.
   */
  const staleOnlyIds = Object.values(fixture.rows,)
    .map(function toId(row,) {
      return row.id;
    },)
    .filter(function staleOnly(id,) {
      return !expectedIds.includes(id);
    },);
  /**
   Listing ids that map to no serviceable row (the media kinds).
   */
  const mediaIds = qwenTokenPlanIndividualListingSnapshot.entries
    .map(function toId(entry,) {
      return entry.id;
    },)
    .filter(function unmapped(id,) {
      return !expectedIds.includes(id);
    },);
  assertSnapshots({
    checks,
    snapshots: {
      before,
      after,
    },
    expected,
    staleOnlyIds,
    mediaIds,
  },);
  /**
   Ids `pi --list-models` printed for the override route.
   */
  const listedIds = parseListedIds({ stdout: listRun.stdout, },);
  check({
    checks,
    label: 'pi --list-models prints the live set for the route',
    ok: sameIdSet({
      actual: listedIds.toSorted(),
      expected: expectedIds.toSorted(),
    },),
    detail: `${String(listedIds.length,)} printed against ${String(expectedIds.length,)} expected`,
  },);
  for (const liveOnly of [
    LIVE_ONLY_FLASH,
    LIVE_ONLY_PREVIEW,
  ]) {
    check({
      checks,
      label: `pi --list-models prints the live-only model "${liveOnly}"`,
      ok: listedIds.includes(liveOnly),
      detail: `present=${String(listedIds.includes(liveOnly,),)}`,
    },);
  }
  check({
    checks,
    label: 'pi --list-models omits the builtin stale-only ids',
    ok: staleOnlyIds.every(function staleAbsent(id,) {
      return !listedIds.includes(id);
    },),
    detail: staleOnlyIds.length === 0
      ? 'the pinned builtin rows hold nothing the listing lacks'
      : `absent: ${staleOnlyIds.join(', ')}`,
  },);
  /**
   Guarded-file comparisons this run asserts: one per home and guarded file.
   */
  const guardedComparisons: {
    readonly agentDir: string;
    readonly runName: string;
    readonly name: string;
    readonly content: string;
  }[] = [];
  for (const home of [
    [
      rpcHost,
      'rpc',
    ],
    [
      listHost,
      'list-models',
    ],
  ] as const) {
    /**
     Throwaway home under test and the run name its checks carry.
     */
    const [host, runName,] = home;
    for (const entry of Object.entries(GUARDED_AGENT_FILES,)) {
      /**
       Guarded file name and the fixture content it must keep.
       */
      const [name, content,] = entry;
      guardedComparisons.push({
        agentDir: host.agentDir,
        runName,
        name,
        content,
      },);
    }
  }
  await Promise.all(guardedComparisons
    .map(async function assertGuarded(comparison,) {
      /**
       Bytes the guarded file carries after the run,
       or the read failure text when the file is gone.
       */
      const afterContent = await readGuardedContent({
        path: join(
          comparison.agentDir,
          comparison.name,
        ),
      },);
      check({
        checks,
        label: `${comparison.runName} leaves ${comparison.name} byte-identical`,
        ok: afterContent === comparison.content,
        detail: afterContent === comparison.content
          ? `${String(comparison.content
            .length,)} bytes unchanged`
          : 'the file changed or vanished',
      },);
    },),);
  check({
    checks,
    label: 'the host reports no extension_error record',
    ok: (!rpcRun.stderr
      .includes('extension_error'))
      && (!listRun.stderr
        .includes('extension_error')),
    detail: 'neither run recorded an extension_error',
  },);
  if (checks.failures
    .length
    > 0)
    throw new Error(`disposable host verification failed:\n${checks.failures
      .join('\n',)}`,);
  return checks.passes;
}

//endregion Verification

//region Entry point

if (process.env[GATE_VARIABLE] !== '1') {
  console.log(`skipped: set ${GATE_VARIABLE}=1 to boot a disposable pi host`,);
} else {
  for (const line of await verifyInDisposableHost())
    console.log(`PASS ${line}`,);
  console.log('qwen-token-plan-individual override verified in a disposable pi host',);
}

//endregion Entry point
