/**
 Exercises the built extension through Pi's real discovery and extension-loading path.
 
 @module
 */

import {
  mkdir,
  mkdtempDisposable,
} from 'node:fs/promises';
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
 Repository-wide agent scratch root holding this verifier's disposable directories.
 */
const AGENT_SCRATCH_ROOT = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Prefix of the Pi agent directory created fresh for a single verification run.
 */
const AGENT_DIRECTORY_PREFIX = 'pi-agent-settled-notification-loader-check-';

/**
 Pi lifecycle event whose handler must reach the desktop-notification boundary.
 */
const AGENT_SETTLED_EVENT = 'agent_settled';

//endregion Constants

//region Runtime verification

/**
 Discovers the built package through Pi and emits its loaded settlement handler.
 
 Pi discovers global extensions under `<agentDir>/extensions/`, so the agent
 directory is created fresh for this run: a reused fixed path would let leftover
 content there change the discovery count this verifier asserts.
 
 @returns verification result text after the real desktop-notification handler completes
 
 @throws when the disposable agent directory cannot be created, Pi discovery reports an error, or the handler cannot complete
 
 @example
 ```ts
 console.log(await verifyPiLoader());
 ```
 */
async function verifyPiLoader(): Promise<string> {
  await mkdir(
    AGENT_SCRATCH_ROOT,
    { recursive: true, },
  );
  /**
   Pi agent directory owned by this run, empty by construction and removed when this scope ends.
   */
  await using agentDirectory = await mkdtempDisposable(join(
    AGENT_SCRATCH_ROOT,
    AGENT_DIRECTORY_PREFIX,
  ),);

  /**
   Pi discovery result for the built package, isolated from user-global extensions.
   */
  const result = await discoverAndLoadExtensions(
    [BUILT_PACKAGE_PATH,],
    process.cwd(),
    agentDirectory.path,
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
