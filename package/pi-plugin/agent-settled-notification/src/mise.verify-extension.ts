/**
 Verifies the built agent-settled notification extension at its package boundary.
 @module
 */

import type {
  ExtensionAPI,
  ExtensionFactory,
} from '@earendil-works/pi-coding-agent';

import { readFile, } from 'node:fs/promises';
import { lodgedImportsIn, } from './bundle-imports.ts';

import {
  createAgentSettledEvent,
  createExtensionContext,
  createFakePiHarness,
  getOnlyHandler,
} from './test-fixtures.ts';

//region Constants

/**
 Built extension path consumed by Pi package discovery.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 The file that path names, read as text so a specifier the bundle asks for at run time can be read out of it.
 */
const BUILT_EXTENSION_FILE = new URL(
  BUILT_EXTENSION_PATH,
  import.meta.url,
).pathname;

/**
 Sole Pi lifecycle event required by this extension.
 */
const AGENT_SETTLED_EVENT = 'agent_settled';

/**
 Executable expected at the host notification boundary.
 */
const EXPECTED_COMMAND = 'notify-send';

/**
 Title sent to the local desktop-notification service.
 */
const EXPECTED_TITLE = 'Pi agent finished';

/**
 Working folder the verifier reports for the settled agent, so the payload must name it.
 */
const VERIFIED_AGENT_FOLDER = '/verification/settled-agent-folder';

//endregion Constants

//region Types

/**
 One notification request observed at the injected desktop boundary.
 */
type ObservedInvocation = {
  /**
   Executable the extension asked for.
   */
  readonly command: string;

  /**
   Arguments the extension passed outside shell syntax.
   */
  readonly args: readonly string[];
};

/**
 Shape exported by the built extension package.
 */
type AgentSettledNotificationModule = {
  /**
   Default Pi extension factory.
   */
  readonly default: ExtensionFactory;

  /**
   Registration helper with an injectable delivery boundary.
   */
  readonly registerAgentSettledNotification: (
    registration: {
      readonly pi: ExtensionAPI;
      readonly deliver: (
        parts: { readonly invocation: ObservedInvocation },
      ) => Promise<void>;
    },
  ) => {
    /**
     Waits for the deliveries the registration started in the background.
     */
    readonly settled: () => Promise<void>;
  };
};

//endregion Types

//region Verification

/**
 Narrows an imported module to the built extension's public shape.

 @param value - imported module namespace

 @returns whether the module exposes the default factory and the injectable registration helper

 @example
 ```ts
 isAgentSettledNotificationModule(await import('../dist/final/node/index.mjs'));
 ```
 */
function isAgentSettledNotificationModule(
  value: unknown,
): value is AgentSettledNotificationModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value)
    && ((typeof value.default) === 'function')
    && ('registerAgentSettledNotification' in value)
    && ((typeof value.registerAgentSettledNotification) === 'function');
}

/**
 Verifies built extension registration and delivery behavior without touching the real desktop.

 @returns verification result text

 @throws when the built export, the lifecycle registration or the delivery payload is incorrect

 @example
 ```ts
 console.log(await verifyBuiltExtension());
 ```
 */
async function verifyBuiltExtension(): Promise<string> {
  /**
   What the bundle asks for at run time and cannot answer from itself, which is what Pi fails to load.
   */
  const lodged = lodgedImportsIn({
    bundle: await readFile(
      BUILT_EXTENSION_FILE,
      'utf8',
    ),
  },);
  if (lodged.length > 0) {
    throw new Error(
      `the built bundle ${BUILT_EXTENSION_FILE} asks for ${lodged.join(', ')} at run time, so Pi loads it only where that package `
        + 'is beside it. A workspace package belongs inside the bundle: run `mise exec -- pnpm install --offline --frozen-lockfile` '
        + 'in the checkout, then build again.',
    );
  }
  /**
   Module namespace imported from the same built entry Pi loads.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isAgentSettledNotificationModule(mod,))
    throw new Error('built agent-settled-notification package has an unexpected export shape',);

  /**
   Harness used to validate the default extension factory.
   */
  const defaultHarness = createFakePiHarness();
  await mod.default(defaultHarness.api,);
  /**
   Registered event names that must each be the settled-agent event.
   */
  const registeredEvents = [...defaultHarness.handlersByEvent
    .keys()];
  if (registeredEvents.some(function isNotAgentSettled(event: string,): boolean {
    return event !== AGENT_SETTLED_EVENT;
  },))
    throw new Error('built default extension registered a non-settled lifecycle event',);
  if (defaultHarness.handlersByEvent
    .get(AGENT_SETTLED_EVENT,)
    ?.length
    !== 1)
    throw new Error(`built default extension did not register exactly one ${AGENT_SETTLED_EVENT} handler`,);

  /**
   Harness used to drive the built handler through an injected desktop boundary.
   */
  const deliveryHarness = createFakePiHarness();
  /**
   Notification requests observed at the operating-system boundary.
   */
  const invocations: ObservedInvocation[] = [];
  /**
   Registration whose background deliveries this verifier waits for.
   */
  const registration = mod.registerAgentSettledNotification({
    pi: deliveryHarness.api,
    deliver: function captureNotification(
      parts: { readonly invocation: ObservedInvocation },
    ): Promise<void> {
      invocations.push(parts.invocation,);
      return Promise.resolve();
    },
  },);

  /**
   Built settled-event callback driven by the verifier.
   */
  const handler = getOnlyHandler({
    handlersByEvent: deliveryHarness.handlersByEvent,
    event: AGENT_SETTLED_EVENT,
  },);
  /**
   Value the settlement hook returned, which must not be a delivery promise.
   */
  const hookResult = handler(
    createAgentSettledEvent({ aborted: false, },),
    createExtensionContext({ cwd: VERIFIED_AGENT_FOLDER, },),
  );
  if (hookResult !== undefined)
    throw new Error('built settlement hook waited for the desktop instead of starting delivery in the background',);
  await registration.settled();

  /**
   Sole notification boundary invocation produced by one settled event.
   */
  const [invocation,] = invocations;
  if (invocations.length !== 1)
    throw new Error(`expected one desktop notification, received ${invocations.length}`,);
  if (invocation?.command !== EXPECTED_COMMAND)
    throw new Error(`expected notification command ${EXPECTED_COMMAND}, received ${invocation?.command ?? '<none>'}`,);
  if (invocation.args
    .at(1,)
    !== EXPECTED_TITLE)
    throw new Error(`expected notification title ${EXPECTED_TITLE}, received ${invocation.args
      .at(1,)
      ?? '<none>'}`,);
  /**
   Body the extension sent, which must name the agent folder.
   */
  const sentBody = invocation.args
    .at(-1,)
    ?? '';
  if (!sentBody.includes(VERIFIED_AGENT_FOLDER))
    throw new Error(`expected the notification body to name ${VERIFIED_AGENT_FOLDER}, received ${sentBody === '' ? '<none>' : sentBody}`,);

  return 'agent-settled-notification extension verified: one notify-send request naming the agent folder after agent_settled';
}

//endregion Verification

console.log(await verifyBuiltExtension(),);
