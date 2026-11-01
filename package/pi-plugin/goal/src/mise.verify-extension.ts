/**
 Verifies repository-owned goal extension at built-artifact registration boundary.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionFactory,
} from '@earendil-works/pi-coding-agent';

//region Contracts

/**
 Built artifact loaded by Pi package manifest.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Exact lifecycle handler counts installed by goal default factory.
 */
const EXPECTED_EVENT_HANDLER_COUNTS = {
  agent_end: 1,
  agent_settled: 1,
  before_agent_start: 1,
  message_end: 1,
  session_compact: 1,
  session_shutdown: 1,
  session_start: 1,
  session_tree: 1,
  tool_result: 1,
} as const satisfies Readonly<Record<string, number>>;

/**
 Public built module shape required by consumers and deterministic verifiers.
 */
type GoalBuiltModule = {
  readonly default: ExtensionFactory;
  readonly parseGoalCommand: (raw: string) => unknown;
  readonly registerGoalSettlementReview: (...inputs: readonly unknown[]) => unknown;
  readonly registerGoalLifecycle: (...inputs: readonly unknown[]) => unknown;
};

/**
 Captured registration inventory from fake Pi API.
 */
type RegistrationHarness = {
  readonly api: ExtensionAPI;
  readonly events: ReadonlyMap<string, readonly unknown[]>;
  readonly commands: readonly string[];
  readonly tools: readonly string[];
  readonly entryRenderers: readonly string[];
};

//endregion Contracts

//region Harness

/**
 Narrow dynamic built-module namespace to required public surface.
 
 @param value - imported module namespace
 
 @returns whether required extension exports are present
 
 @example
 ```ts
 isGoalBuiltModule(await import('../dist/final/node/index.mjs'));
 ```
 */
function isGoalBuiltModule(value: unknown,): value is GoalBuiltModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value)
    && ((typeof value.default) === 'function')
    && ('parseGoalCommand' in value)
    && ((typeof value.parseGoalCommand) === 'function')
    && ('registerGoalSettlementReview' in value)
    && ((typeof value.registerGoalSettlementReview) === 'function')
    && ('registerGoalLifecycle' in value)
    && ((typeof value.registerGoalLifecycle) === 'function');
}

/**
 Capture default-factory registration without invoking runtime effects.
 
 @returns focused fake API and registration inventory
 
 @example
 ```ts
 const harness = createRegistrationHarness();
 ```
 */
function createRegistrationHarness(): RegistrationHarness {
  /**
   Event handlers grouped by Pi lifecycle event.
   */
  const events = new Map<string, unknown[]>();
  /**
   Slash commands in registration order.
   */
  const commands: string[] = [];
  /**
   Model tools in registration order.
   */
  const tools: string[] = [];
  /**
   Session-entry renderers in registration order.
   */
  const entryRenderers: string[] = [];
  /**
   Minimal registration-only Pi API.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Registration verifier implements only API methods invoked during factory loading.
  const api = {
    on(
      event: string,
      handler: unknown,
    ) {
      events.set(
        event,
        [
          ...events.get(event,) ?? [],
          handler,
        ],
      );
    },
    registerCommand(name: string,) {
      commands.push(name,);
    },
    registerTool(tool: { readonly name: string; },) {
      tools.push(tool.name,);
    },
    registerEntryRenderer(customType: string,) {
      entryRenderers.push(customType,);
    },
  } as unknown as ExtensionAPI;
  return {
    api,
    events,
    commands,
    tools,
    entryRenderers,
  };
}

/**
 Require exact one-element string registration.
 
 @param actual - captured names
 
 @param expected - sole expected name
 
 @param kind - diagnostic registration kind
 
 @throws when captured inventory differs
 
 @example
 ```ts
 requireOnlyRegistration({ actual: ['goal'], expected: 'goal', kind: 'command' });
 ```
 */
function requireOnlyRegistration(
  {
    actual,
    expected,
    kind,
  }: {
    readonly actual: readonly string[];
    readonly expected: string;
    readonly kind: string;
  },
): void {
  if ((actual.length !== 1) || (actual[0] !== expected))
    throw new Error(`expected one ${kind} ${expected}; received ${actual.join(', ') || '<none>'}`,);
}

//endregion Harness

//region Verification

/**
 Import built package and validate default extension inventory.
 
 @returns verification result text
 
 @throws when export shape or registration inventory violates contract
 
 @example
 ```ts
 console.log(await verifyBuiltGoalExtension());
 ```
 */
async function verifyBuiltGoalExtension(): Promise<string> {
  /**
   Side-effect-free namespace import of exact built artifact.
   */
  const imported: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isGoalBuiltModule(imported,))
    throw new Error('built goal extension has unexpected named or default exports',);
  /**
   Registration inventory populated only after explicit default invocation.
   */
  const harness = createRegistrationHarness();
  await imported.default(harness.api,);
  requireOnlyRegistration({
    actual: harness.commands,
    expected: 'goal',
    kind: 'command',
  },);
  if (harness.tools
    .length
    > 0)
    throw new Error(`goal extension exposed primary tools: ${harness.tools
      .join(', ')}`,);
  if (JSON.stringify(harness.entryRenderers
    .toSorted(),) !== JSON.stringify([
      'goal:completion',
      'goal:review-unavailable',
    ],)) {
    throw new Error(`unexpected entry renderers: ${harness.entryRenderers
      .join(', ')}`,);
  }
  if (harness.events
    .has('tool_call',))
    throw new Error('goal extension registered forbidden goal-state tool_call blocker',);
  /**
   Sorted actual lifecycle inventory for exact comparison.
   */
  const actualEvents = [...harness.events
    .keys(),]
    .toSorted();
  /**
   Sorted expected lifecycle inventory for exact name comparison.
   */
  const expectedEvents = Object.keys(EXPECTED_EVENT_HANDLER_COUNTS,)
    .toSorted();
  if (JSON.stringify(actualEvents,) !== JSON.stringify(expectedEvents,))
    throw new Error(`unexpected lifecycle inventory: ${actualEvents.join(', ')}`,);
  /**
   Expected handler counts with safe dynamic event lookup.
   */
  const expectedEventHandlerCounts = new Map(Object.entries(EXPECTED_EVENT_HANDLER_COUNTS,),);
  /**
   First event whose loaded handler count differs from contract.
   */
  const wrongCountEvent = actualEvents.find(function hasWrongCount(event,) {
    /**
     Expected handler count for exact event name.
     */
    const expectedCount = expectedEventHandlerCounts.get(event,);
    if (expectedCount === undefined)
      return true;
    return harness.events
      .get(event,)
      ?.length
      !== expectedCount;
  },);
  if (wrongCountEvent !== undefined)
    throw new Error(`unexpected ${wrongCountEvent} handler count`,);
  return 'pi-goal built extension verified: one command, zero primary tools, human-only outcome renderers, exact lifecycle handlers';
}

//endregion Verification

console.log(await verifyBuiltGoalExtension(),);
