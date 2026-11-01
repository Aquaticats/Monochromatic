/**
 Shared constants and cleanup utilities for template baking pipelines.
 Used by both Linux and Windows template creation modules.
 */

import { rm, } from 'node:fs/promises';
import { join, } from 'node:path';

import { VMS_DIR, } from './config.ts';
import {
  destroyVm,
  undefineVm,
} from './virsh.ts';

/**
 Name used for the temporary VM during template creation.
 */
export const TEMPLATE_VM_NAME = 'template-setup';

/**
 Creates an `AsyncDisposable` guard that cleans up the template VM
 when the enclosing `await using` block exits (normally or via error).
 
 @param rl - Logger for status messages
 
 @returns disposable that calls {@link cleanupTemplateVm} on dispose
 
 @example
 ```ts
 await using _guard = templateVmGuard(rl);
 // ... VM operations ...
 // cleanupTemplateVm runs automatically on block exit
 ```
 */
export function templateVmGuard(
  rl: { readonly debug: (msg: string,) => void; },
): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await cleanupTemplateVm(rl,);
    },
  };
}

/**
 Cleans up the temporary template VM and its directory.
 Tolerates errors since the VM may already be stopped or undefined.
 
 @param rl - Logger for status messages
 
 @example
 ```ts
 await cleanupTemplateVm(rl);
 ```
 */
async function cleanupTemplateVm(rl: { readonly debug: (msg: string,) => void; },): Promise<void> {
  rl.debug('cleaning up template VM...',);
  try {
    await destroyVm({ name: TEMPLATE_VM_NAME, },);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.debug('template VM was already stopped',);
  }
  try {
    await undefineVm({ name: TEMPLATE_VM_NAME, },);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.debug('template VM was not defined, skipping undefine',);
  }
  await rm(
    join(
      VMS_DIR,
      TEMPLATE_VM_NAME,
    ),
    {
      force: true,
      recursive: true,
    },
  );
}
