/**
 clack prompt wrappers. Each converts clack's cancel sentinel into a thrown
 {@link ./errors.ts PromptCancelledError}, so the orchestrator reads straight
 values and the top-level handler treats cancellation as a clean exit.
 
 @module
 */

import {
  confirm,
  isCancel,
  multiselect,
  select,
} from '@clack/prompts';

import type { Changes, } from './changes.ts';
import { PromptCancelledError, } from './errors.ts';

/**
 One selectable option in a clack `select`/`multiselect` list.
 */
type AppOption = {
  readonly value: string;
  readonly label: string;
  readonly hint?: string;
};

/**
 Prompt for which connected device to target. A single device is returned
 without prompting; multiple devices show a select list.
 
 @param serials - Connected device serials (non-empty; caller guarantees it).
 
 @returns Chosen serial.
 
 @throws {@link PromptCancelledError} when the user cancels the select.
 
 @example
 ```ts
 const serial = await pickDevice({ serials: ['ABC123', 'emulator-5554',], },);
 ```
 */
export async function pickDevice({ serials, }: { readonly serials: readonly string[]; },): Promise<string> {
  /**
   First serial, used to short-circuit the single-device case.
   */
  const [first,] = serials;
  if ((serials.length === 1) && (first !== undefined)) {
    return first;
  }
  /**
   Chosen serial, or clack's cancel sentinel.
   */
  const result = await select<string>({
    message: 'Select a device',
    options: serials.map(function toDeviceOption(serial,): AppOption {
      return {
        value: serial,
        label: serial,
      };
    },),
  },);
  if (isCancel(result,)) {
    throw new PromptCancelledError();
  }
  return result;
}

/**
 Show the live exemption multiselect: every third-party app, with already
 exempted apps pre-checked and hinted. The returned set is the user's desired
 exempt state, fed to {@link ./changes.ts computeChanges}.
 
 @param all - Every third-party application id, in display order.
 
 @param currentlyExempted - Application ids to pre-check.
 
 @returns Application ids the user left checked (may be empty).
 
 @throws {@link PromptCancelledError} when the user cancels.
 
 @example
 ```ts
 const selected = await pickApps({ all, currentlyExempted, },);
 ```
 */
export async function pickApps({
  all,
  currentlyExempted,
}: {
  readonly all: readonly string[];
  readonly currentlyExempted: readonly string[];
},): Promise<readonly string[]> {
  /**
   Lookup of already-exempted ids, used to pre-check and hint.
   */
  const exemptedSet: ReadonlySet<string> = new Set(currentlyExempted,);
  /**
   Lookup of in-scope ids, used to constrain pre-checked values.
   */
  const allSet: ReadonlySet<string> = new Set(all,);
  /**
   Checked application ids, or clack's cancel sentinel.
   */
  const result = await multiselect<string>({
    message: 'Checked apps are exempt from auto-revoke. Pre-checked apps are already exempted; uncheck to revert them.',
    options: all.map(function toAppOption(packageName,): AppOption {
      if (exemptedSet.has(packageName,)) {
        return {
          value: packageName,
          label: packageName,
          hint: 'currently exempted',
        };
      }
      return {
        value: packageName,
        label: packageName,
      };
    },),
    initialValues: currentlyExempted.filter(function inScopeName(name,): boolean {
      return allSet.has(name,);
    },),
    required: false,
  },);
  if (isCancel(result,)) {
    throw new PromptCancelledError();
  }
  return result;
}

/**
 Confirm before mutating the device, showing the total change count.
 
 @param changes - Pending changes whose counts size the prompt.
 
 @returns `true` to apply, `false` to abort without changes.
 
 @throws {@link PromptCancelledError} when the user cancels.
 
 @example
 ```ts
 if (await confirmApply({ changes, },)) await applyChanges({ serial, changes, },);
 ```
 */
export async function confirmApply({ changes, }: { readonly changes: Changes; },): Promise<boolean> {
  /**
   Total number of pending writes across both directions.
   */
  const total = changes.toExempt
    .length
    + changes.toRevert
      .length;
  /**
   User's choice, or clack's cancel sentinel.
   */
  const result = await confirm({
    message: `Apply ${String(total,)} change(s) to the device?`,
  },);
  if (isCancel(result,)) {
    throw new PromptCancelledError();
  }
  return result;
}
