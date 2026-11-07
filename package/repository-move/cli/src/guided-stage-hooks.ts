/**
 The functions the stage data hands to the engine: each takes the engine's context and calls one computation of the
 runner, so the stage lists stay plain data.

 @module
 */

import {
  preflight,
  windowMaximum,
} from './guided-computations.ts';
import { holderScanCommand, } from './guided-holders.ts';
import { prepareUnitEdit, } from './guided-unit-edit.ts';
import { selectVerify, } from './guided-verify-set.ts';
import type { InternalContext, } from './guided-types.ts';

/**
 Checks the space rule.

 @param config - the runner's configuration
 
 @returns the lines to show

 @example
 ```ts
 import { hookPreflight, } from './guided-stage-hooks.ts';
 
 const result = await hookPreflight();
 ```
 */
export async function hookPreflight({ config, }: InternalContext,): Promise<string> {
  return await preflight(config,);
}

/**
 Derives and records the window maximum.

 @param config - the runner's configuration
 
 @returns the lines to show

 @example
 ```ts
 import { hookWindowMaximum, } from './guided-stage-hooks.ts';
 
 const result = await hookWindowMaximum();
 ```
 */
export async function hookWindowMaximum({ config, }: InternalContext,): Promise<string> {
  return await windowMaximum(config,);
}

/**
 Chooses the subvolumes to compare by content.

 @param config - the runner's configuration
 
 @returns the line to show

 @example
 ```ts
 import { hookSelectVerify, } from './guided-stage-hooks.ts';
 
 const result = await hookSelectVerify();
 ```
 */
export async function hookSelectVerify({ config, }: InternalContext,): Promise<string> {
  return await selectVerify(config,);
}

/**
 Prepares the edited copy of the 860 mount unit.

 @param config - the runner's configuration
 
 @returns the lines to show

 @example
 ```ts
 import { hookUnitEdit, } from './guided-stage-hooks.ts';
 
 const result = await hookUnitEdit();
 ```
 */
export async function hookUnitEdit({ config, }: InternalContext,): Promise<string> {
  return await prepareUnitEdit(config,);
}

/**
 Builds the holder scan command that runs before the freeze.

 @param config - the runner's configuration
 
 @returns the shell text

 @example
 ```ts
 import { hookHoldersBefore, } from './guided-stage-hooks.ts';
 
 const result = await hookHoldersBefore();
 ```
 */
export async function hookHoldersBefore({ config, }: InternalContext,): Promise<string> {
  return await holderScanCommand({
    config,
    name: 'holders-before',
  },);
}

/**
 Builds the holder scan command that runs after the freeze.

 @param config - the runner's configuration
 
 @returns the shell text

 @example
 ```ts
 import { hookHoldersFrozen, } from './guided-stage-hooks.ts';
 
 const result = await hookHoldersFrozen();
 ```
 */
export async function hookHoldersFrozen({ config, }: InternalContext,): Promise<string> {
  return await holderScanCommand({
    config,
    name: 'holders-frozen',
  },);
}
