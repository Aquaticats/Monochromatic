/**
 Exercises the built extension through Pi's real discovery and extension-loading path.
 
 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  createEventBus,
  discoverAndLoadExtensions,
} from '@earendil-works/pi-coding-agent';

import {
  createAgentSettledEvent,
  createExtensionContext,
} from './test-fixtures.ts';

//region Constants

/**
 Built extension package directory consumed by Pi's package-manifest loader.
 */
const BUILT_PACKAGE_PATH = '.';

/**
 Empty directory under current agent scratch root that excludes unrelated global extensions.
 */
const EMPTY_AGENT_DIRECTORY = join(
  homedir(),
  'temp',
  'agent',
  'pi-agent-settled-notification-loader-check',
);

/**
 Pi lifecycle event whose handler must reach the desktop-notification boundary.
 */
const AGENT_SETTLED_EVENT = 'agent_settled';

//endregion Constants

//region Runtime verification

/**
 Discovers the built package through Pi and emits its loaded settlement handler.
 
 @returns verification result text after the real desktop-notification handler completes
 
 @throws when Pi discovery reports an error or the handler cannot complete
 
 @example
 ```ts
 console.log(await verifyPiLoader());
 ```
 */
async function verifyPiLoader(): Promise<string> {
  /**
   Pi discovery result for the built package, isolated from user-global extensions.
   */
  const result = await discoverAndLoadExtensions(
    [BUILT_PACKAGE_PATH,],
    process.cwd(),
    EMPTY_AGENT_DIRECTORY,
    createEventBus(),
  );
  if (result.errors
    .length
    > 0)
    throw new Error(`Pi discovery reported: ${result.errors
      .map(function errorText(error: Readonly<(typeof result.errors)[number]>,): string {
      return error.error;
    },)
      .join('; ')}`,);
  if (result.extensions
    .length
    !== 1)
    throw new Error(`expected one discovered extension, received ${result.extensions
      .length}`,);

  /**
   Package extension loaded by Pi's manifest and factory adapter.
   */
  const [extension,] = result.extensions;
  if (extension === undefined)
    throw new Error('Pi discovery returned no extension',);
  /**
   Handler collection Pi registered for agent settlement.
   */
  const handlers = extension.handlers
    .get(AGENT_SETTLED_EVENT,);
  if (handlers === undefined)
    throw new Error(`missing ${AGENT_SETTLED_EVENT} handler`,);
  if (handlers.length !== 1)
    throw new Error(`expected one ${AGENT_SETTLED_EVENT} handler, received ${handlers.length}`,);
  /**
   Sole settlement handler loaded by Pi and driven without a provider request.
   */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error(`missing ${AGENT_SETTLED_EVENT} handler`,);

  await handler(
    createAgentSettledEvent({ aborted: false, },),
    createExtensionContext(),
  );

  return 'agent-settled-notification Pi-loader verification passed';
}

//endregion Runtime verification

console.log(await verifyPiLoader(),);
