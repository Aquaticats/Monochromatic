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

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import {
  mapQwenTokenPlanIndividualModels,
  qwenTokenPlanIndividualListingSnapshot,
  type QwenTokenPlanResolvedModelFacts,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import {
  projectQwenTokenPlanIndividualModels,
} from '../dist/final/node/index.mjs';
import {
  createDisposableHome,
  GUARDED_AGENT_FILES,
  piCliPath,
  readSnapshots,
  removeDisposableHome,
  runDisposableHost,
  TARGET_PROVIDER,
  type Snapshot,
} from './disposable-host.ts';

//region Constants

/**
 Environment variable that arms this verification.
 */
const GATE_VARIABLE = 'QWEN_TOKEN_PLAN_INDIVIDUAL_VERIFY_HOST';

/**
 Character count of stderr quoted when the host reports no snapshots.
 */
const STDERR_TAIL_CHARS = 1_200;

/**
 Model the live listing serves and the builtin catalog lacks,
 `deepseek-v4.1-flash-1017` documented in
 `doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md`.
 */
const LIVE_ONLY_FLASH = 'deepseek-v4.1-flash-1017';

/**
 Model the live listing serves and the builtin catalog lacks,
 `qwen4-max-preview` documented in
 `doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md`.
 */
const LIVE_ONLY_PREVIEW = 'qwen4-max-preview';

/**
 Column label `pi --list-models` prints above the provider column,
 so parsed rows are recognized by their provider field instead of by position.
 */
const LIST_MODELS_PROVIDER_LABEL = 'provider';

//endregion Constants

//region Types

/**
 Assertion outcomes collected during one run.
 */
type Checks = {
  /**
   Claims that held, each with the measured value behind it.
   */
  readonly passes: string[];
  /**
   Claims that did not hold, each with the measured value behind it.
   */
  readonly failures: string[];
};

/**
 Committed shape of `src/builtin-catalog.fixture.json`,
 the pinned builtin rows the stale-only comparison reads.
 */
type BuiltinCatalogFixture = {
  readonly rows: Readonly<Record<string, { readonly id: string }>>;
};

/**
 Projected row fields the probe reports and the assertions compare.
 */
type QwenTokenPlanIndividualExpectedRow = {
  readonly id: string;
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly reasoning: boolean;
  readonly thinkingLevelCount: number;
};

//endregion Types

//region Assertions

/**
 Record one assertion outcome.

 @param checks - collector the outcome is written to

 @param label - what the assertion claims

 @param ok - whether the claim held

 @param detail - measured value the reader needs when the claim did not hold

 @example
 ```typescript
 check({ checks, label: 'served set matches', ok: true, detail: '14 models' });
 ```
 */
function check(
  {
    checks,
    label,
    ok,
    detail,
  }: {
    readonly checks: Checks;
    readonly label: string;
    readonly ok: boolean;
    readonly detail: string;
  },
): void {
  if (ok)
    checks.passes
      .push(`${label}: ${detail}`,);
  else
    checks.failures
      .push(`${label}: ${detail}`,);
}

/**
 Sorted model ids one snapshot reports for the override route.

 @param snapshot - probe snapshot to read

 @returns the served ids in sorted order
 */
function servedIds(
  snapshot: Snapshot,
): string[] {
  return snapshot.routeModels
    .map(function toId(model,) {
      return model.id;
    },)
    .toSorted();
}

/**
 Whether two sorted id lists name the same set.

 @param actual - ids the host served

 @param expected - ids the projection earns

 @returns `true` when both lists match position by position
 */
function sameIdSet(
  actual: readonly string[],
  expected: readonly string[],
): boolean {
  return (actual.length === expected.length)
    && actual.every(function matches(one, index,) {
      return one === expected[index];
    },);
}

/**
 Assert the served rows against the projection of the checked-in listing.

 @param checks - collector the outcomes are written to

 @param snapshots - the two probe snapshots, before and after the refresh

 @param expected - projected rows the checked-in listing earns

 @param staleOnlyIds - ids the pinned builtin rows serve that the listing lacks

 @param mediaIds - ids the live listing returns that map to no serviceable row

 @example
 ```typescript
 assertSnapshots({ checks, snapshots, expected, staleOnlyIds, mediaIds });
 ```
 */
function assertSnapshots(
  {
    checks,
    snapshots,
    expected,
    staleOnlyIds,
    mediaIds,
  }: {
    readonly checks: Checks;
    readonly snapshots: {
      readonly before: Snapshot;
      readonly after: Snapshot;
    };
    readonly expected: readonly QwenTokenPlanIndividualExpectedRow[];
    readonly staleOnlyIds: readonly string[];
    readonly mediaIds: readonly string[];
  },
): void {
  /**
   Expected ids in sorted order, the comparison's reference set.
   */
  const expectedIds = expected
    .map(function toId(row,) {
      return row.id;
    },)
    .toSorted();
  check({
    checks,
    label: 'snapshot catalog serves the checked-in listing set at session start',
    ok: sameIdSet(servedIds(snapshots.before,), expectedIds),
    detail: `${String(snapshots.before.routeModels.length,)} served against ${String(expectedIds.length,)} expected`,
  },);
  check({
    checks,
    label: 'refresh keeps the served set when the listing fetch cannot land',
    ok: sameIdSet(servedIds(snapshots.after,), servedIds(snapshots.before,)),
    detail: `${String(snapshots.before.routeModels.length,)} -> ${String(snapshots.after.routeModels.length,)} served`,
  },);
  for (const liveOnly of [
    LIVE_ONLY_FLASH,
    LIVE_ONLY_PREVIEW,
  ]) {
    check({
      checks,
      label: `live-only model "${liveOnly}" is served`,
      ok: servedIds(snapshots.after,)
        .includes(liveOnly),
      detail: `present=${String(servedIds(snapshots.after,)
        .includes(liveOnly,),)}`,
    },);
  }
  check({
    checks,
    label: 'listing ids that map to no serviceable row are never served',
    ok: mediaIds.every(function mediaAbsent(id,) {
      return !servedIds(snapshots.after,)
        .includes(id);
    },),
    detail: mediaIds.length === 0
      ? 'the listing carries no media ids'
      : `absent: ${mediaIds.join(', ')}`,
  },);
  check({
    checks,
    label: 'builtin stale-only ids are gone',
    ok: staleOnlyIds.every(function staleAbsent(id,) {
      return !servedIds(snapshots.after,)
        .includes(id);
    },),
    detail: staleOnlyIds.length === 0
      ? 'the pinned builtin rows hold nothing the listing lacks'
      : `absent: ${staleOnlyIds.join(', ')}`,
  },);
  /**
   Served rows whose facts diverge from the projection.
   */
  const divergent = snapshots.after.routeModels
    .filter(function divergesFromProjection(model,) {
      /**
       Projected row the same id earns, absent when the host served an id the
       listing does not map.
       */
      const projected = expected.find(function sameId(row,) {
        return row.id === model.id;
      },);
      if (projected === undefined)
        return true;
      return (model.contextWindow !== projected.contextWindow)
        || (model.maxTokens !== projected.maxTokens)
        || (model.reasoning !== projected.reasoning)
        || (model.thinkingLevelCount !== projected.thinkingLevelCount);
    },);
  check({
    checks,
    label: 'every served row carries the projected facts',
    ok: divergent.length === 0,
    detail: divergent.length === 0
      ? `${String(snapshots.after.routeModels.length,)} rows match`
      : `divergent: ${divergent
        .map(function name(model,) {
          return model.id;
        },)
        .join(', ')}`,
  },);
}

//endregion Assertions

//region Expectations

/**
 The rows the checked-in listing earns under the shared mapping and this
 package's projection.

 @returns the projected rows in listing order

 @example
 ```typescript
 const expected = await expectedRows();
 ```
 */
async function expectedRows(): Promise<QwenTokenPlanIndividualExpectedRow[]> {
  /**
   Resolved facts the shared mapping earns from the checked-in listing.
   */
  const facts: readonly QwenTokenPlanResolvedModelFacts[] =
    mapQwenTokenPlanIndividualModels({
      listing: qwenTokenPlanIndividualListingSnapshot.entries,
    },);
  return projectQwenTokenPlanIndividualModels({ facts, },)
    .map(function toExpectedRow(row,) {
      return {
        id: row.id,
        contextWindow: row.contextWindow,
        maxTokens: row.maxTokens,
        reasoning: row.reasoning,
        thinkingLevelCount: row.thinkingLevelMap === undefined
          ? 0
          : Object.keys(row.thinkingLevelMap,)
            .length,
      };
    },);
}

/**
 Read the pinned builtin rows the stale-only comparison derives from.

 @returns the fixture document

 @example
 ```typescript
 const fixture = await readBuiltinFixture();
 ```
 */
async function readBuiltinFixture(): Promise<BuiltinCatalogFixture> {
  /**
   Pinned fixture text beside this module.
   */
  const text = await readFile(fileURLToPath(new URL(
    './builtin-catalog.fixture.json',
    import.meta.url,
  ),), 'utf8',);
  return JSON.parse(text,) as BuiltinCatalogFixture;
}

//endregion Expectations

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
 Parse the model ids `pi --list-models` prints for the override route.

 @param stdout - everything the listing command printed

 @returns the route's model ids in printed order

 @example
 ```typescript
 parseListedIds({ stdout });
 ```
 */
function parseListedIds({ stdout, }: { readonly stdout: string; },): string[] {
  /**
   Printed ids collected in row order.
   */
  const ids: string[] = [];
  for (const line of stdout.split('\n',)) {
    /**
     Whitespace-separated columns of one printed row.
     */
    const columns = line
      .trim()
      .split(/\s+/u,);
    const [provider, model,] = columns;
    if ((provider === undefined) || (model === undefined))
      continue;
    if (provider === LIST_MODELS_PROVIDER_LABEL)
      continue;
    if (provider !== TARGET_PROVIDER)
      continue;
    ids.push(model,);
  }
  return ids;
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
  const expected = await expectedRows();
  /**
   Pinned builtin rows the stale-only comparison reads.
   */
  const fixture = await readBuiltinFixture();
  /**
   Ids the pinned builtin rows serve that the listing does not map.
   */
  const expectedIds = expected
    .map(function toId(row,) {
      return row.id;
    },);
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
    ok: sameIdSet(listedIds.toSorted(), expectedIds.toSorted()),
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
  for (const [host, runName,] of [
    [rpcHost, 'rpc',],
    [listHost, 'list-models',],
  ] as const) {
    await Promise.all(Object.entries(GUARDED_AGENT_FILES,)
      .map(async function assertGuarded([name, content,]) {
        /**
         Bytes the guarded file carries after the run.
         */
        const afterBytes = await readFile(
          join(host.agentDir, name,),
          'utf8',
        )
          .catch(function readFailed(): string {
            return '<absent>';
          },);
        check({
          checks,
          label: `${runName} leaves ${name} byte-identical`,
          ok: afterBytes === content,
          detail: afterBytes === content
            ? `${String(content.length,)} bytes unchanged`
            : (afterBytes === '<absent>'
              ? 'the file is gone'
              : 'the file changed'),
        },);
      },),);
  }
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
