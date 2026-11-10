/**
 Verifies built current-time-context extension registration and message shape.
 
 @module
 */

import type { ExtensionFactory, } from '@earendil-works/pi-coding-agent';
import {
  createBeforeAgentStartEvent,
  createExtensionContext,
  fakePiApi,
  getBeforeAgentStartHandler,
} from './pi-api-fixture.ts';
import { isTimeContextContent, } from './time-context-shape.ts';

//region Constants

/**
 Built extension path consumed by Pi.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Expected event registration from the extension entry point.
 */
const EXPECTED_REGISTRATION = 'event:before_agent_start';

/**
 Expected custom message type emitted by the extension.
 */
const EXPECTED_CUSTOM_TYPE = 'current-time-context';

//endregion Constants

//region Types

/**
 Built current-time-context extension module shape.
 */
type CurrentTimeContextExtensionModule = {
  /**
   Pi extension factory.
   */
  default: ExtensionFactory;
};

//endregion Types

//region Verification

/**
 Verifies built extension registers and emits hidden time context: builds a
 {@link fakePiApi} harness, drives the captured handler (found via
 {@link getBeforeAgentStartHandler}) with a {@link createBeforeAgentStartEvent}
 event and {@link createExtensionContext}, then checks the resulting message
 content with {@link isTimeContextContent}.
 
 @returns verification result text
 
 @throws when built extension import, registration, or handler output fails
 
 @example
 ```typescript
 console.log(await verifyBuiltExtension());
 ```
 */
async function verifyBuiltExtension(): Promise<string> {
  /**
   Built extension module imported through package output.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isCurrentTimeContextExtensionModule(mod,)) {
    throw new Error(
      'built current-time-context extension does not export a default extension factory',
    );
  }

  /**
   Fake Pi API harness for registration and handler verification.
   */
  const harness = fakePiApi();
  await mod.default(harness.api,);

  if (!harness.registrations
    .includes(EXPECTED_REGISTRATION,)) {
    throw new Error(
      `missing current-time-context registration: ${EXPECTED_REGISTRATION}`,
    );
  }

  /**
   Captured before-agent-start handler.
   */
  const handler = getBeforeAgentStartHandler(harness.handlers,);

  /**
   Handler result emitted by the built extension.
   */
  const result = await handler(
    createBeforeAgentStartEvent(),
    createExtensionContext(),
  );
  if (result === undefined)
    throw new Error('before_agent_start handler returned no result',);

  /**
   Custom message returned by the extension.
   */
  const { message, } = result;
  if (message === undefined)
    throw new Error('current-time-context handler returned no message',);
  if (message.customType
    !== EXPECTED_CUSTOM_TYPE) {
    throw new Error(
      `unexpected custom type: ${message.customType}`,
    );
  }
  if ((typeof message.content) !== 'string')
    throw new Error('current-time-context content was not a string',);
  if (!isTimeContextContent(message.content,)) {
    throw new Error(
      `current-time-context content had unexpected shape: ${message.content}`,
    );
  }
  if (!Object.is(
    message.display,
    false,
  )) {
    throw new Error('current-time-context message was not hidden',);
  }

  return 'current-time-context extension verified: event:before_agent_start, hidden <time>HH:MM</time>';
}

/**
 Detects built current-time-context extension module shape.
 
 @param value - imported module namespace
 
 @returns whether module exports extension factory
 
 @example
 ```typescript
 isCurrentTimeContextExtensionModule(await import('../dist/final/node/index.mjs'));
 ```
 */
function isCurrentTimeContextExtensionModule(
  value: unknown,
): value is CurrentTimeContextExtensionModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value) && ((typeof value.default) === 'function');
}

//endregion Verification

console.log(await verifyBuiltExtension(),);
