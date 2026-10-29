#!/usr/bin/env node
/**
 Entry point for the android-exempt-unused CLI.
 
 Drives the flow: pick a device, load third-party apps and their current
 exemption state, present a live multiselect, diff the selection, confirm, and
 apply both exempt and revert changes. The multiselect's checked set is the
 desired state, so one list handles both directions with no mode toggle.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  cancel,
  intro,
  log,
  note,
  outro,
  spinner,
} from '@clack/prompts';

import { applyChanges, } from './apply.ts';
import {
  type Changes,
  computeChanges,
} from './changes.ts';
import { CONNECTED_STATE, } from './constants.ts';
import { listDevices, } from './devices.ts';
import {
  AdbError,
  NoDevicesError,
  PromptCancelledError,
} from './errors.ts';
import {
  listExempted,
  listThirdPartyPackages,
} from './packages.ts';
import {
  confirmApply,
  pickApps,
  pickDevice,
} from './prompts.ts';

/**
 Module-level tagged logger; each function wraps it with its own name.
 */
const l = tagged({ tag: 'main', },);

/**
 Render a human-readable summary of pending changes for the confirm note.
 
 @param changes - Exempt and revert partitions to describe.
 
 @returns Multi-line block listing each app under its direction heading.
 */
function summarize({ changes, }: { readonly changes: Changes; },): string {
  /**
   Lines describing apps to exempt, or empty when none.
   */
  const exemptLines = changes.toExempt
    .length
    > 0
    ? [
      `Exempt (${String(changes.toExempt
        .length,)}):`,
      ...changes.toExempt
        .map(function plusLine(name,): string {
          return `  + ${name}`;
        },),
    ]
    : [];
  /**
   Lines describing apps to revert, or empty when none.
   */
  const revertLines = changes.toRevert
    .length
    > 0
    ? [
      `Revert to default (${String(changes.toRevert
        .length,)}):`,
      ...changes.toRevert
        .map(function minusLine(name,): string {
          return `  - ${name}`;
        },),
    ]
    : [];
  return [
    ...exemptLines,
    ...revertLines,
  ].join('\n',);
}

/**
 Run the interactive flow end to end. Throws on adb failure or cancellation;
 the top-level handler renders both.
 
 @throws {@link NoDevicesError} when no authorized device is connected.
 
 @throws {@link PromptCancelledError} when the user cancels any prompt.
 */
async function runCli(): Promise<void> {
  /**
   Tagged logger for the main flow.
   */
  const fl = tagged({
    tag: runCli.name,
    l,
  },);
  intro('android-exempt-unused',);

  /**
   Every device adb reports, with state.
   */
  const devices = await listDevices();
  /**
   Serials of devices ready to accept shell commands.
   */
  const connected = devices
    .filter(function isConnected(device,): boolean {
      return device.state === CONNECTED_STATE;
    },)
    .map(function toSerial(device,): string {
      return device.serial;
    },);
  fl.info(`found ${String(connected.length,)} connected device(s)`,);
  if (connected.length === 0) {
    throw new NoDevicesError(
      'No authorized adb devices found. Connect a device with USB debugging enabled, then re-run (verify with `adb devices`).',
    );
  }

  /**
   Chosen device serial.
   */
  const serial = await pickDevice({ serials: connected, },);

  /**
   Spinner shown while loading device state.
   */
  const load = spinner();
  load.start('Listing third-party apps',);
  /**
   Every third-party application id on the device.
   */
  const all = await listThirdPartyPackages({ serial, },);
  load.message('Reading current exemption state',);
  /**
   Application ids currently exempt from auto-revoke.
   */
  const exempted = await listExempted({
    serial,
    packages: all,
  },);
  load.stop(`Found ${String(all.length,)} third-party apps (${String(exempted.length,)} currently exempted)`,);

  if (all.length === 0) {
    outro('No third-party apps are installed; nothing to do.',);
    return;
  }

  /**
   Application ids the user left checked.
   */
  const selected = await pickApps({
    all,
    currentlyExempted: exempted,
  },);
  /**
   Diff of selection against current device state.
   */
  const changes = computeChanges({
    all,
    currentlyExempted: exempted,
    selected,
  },);

  if ((changes.toExempt
    .length
    === 0) && (changes.toRevert
      .length
      === 0)) {
    outro('No changes; the device already matches your selection.',);
    return;
  }

  note(
    summarize({ changes, },),
    'Pending changes',
  );

  /**
   Whether the user confirmed applying the changes.
   */
  const confirmed = await confirmApply({ changes, },);
  if (!confirmed) {
    cancel('Aborted; no changes were applied.',);
    return;
  }

  /**
   Total number of writes to apply.
   */
  const total = changes.toExempt
    .length
    + changes.toRevert
      .length;
  /**
   Spinner shown while writing changes.
   */
  const apply = spinner();
  apply.start('Applying changes',);
  /**
   Apps whose write failed, if any.
   */
  const failures = await applyChanges({
    serial,
    changes,
    onProgress: function reportProgress({
      done,
      packageName,
    },): void {
      apply.message(`Applying ${String(done + 1,)}/${String(total,)}: ${packageName}`,);
    },
  },);
  apply.stop(`Applied ${String(total - failures.length,)}/${String(total,)} changes`,);

  if (failures.length > 0) {
    log.error(
      failures
        .map(function toFailureLine(failure,): string {
          return `${failure.packageName} (${failure.mode}): ${failure.message}`;
        },)
        .join('\n',),
    );
    process.exitCode = 1;
    outro('Completed with errors.',);
    return;
  }

  outro('Done.',);
}

/**
 Map a thrown value to terminal output. Cancellation is a clean exit; known
 {@link AdbError}s become a message plus a non-zero exit code; anything else
 is rethrown for Node to surface with a stack.
 
 @param error - Value caught from runCli.
 
 @throws Re-throws any error that is not a {@link PromptCancelledError} or
   {@link AdbError}.
 */
function handleTopLevelError({ error, }: { readonly error: unknown; },): void {
  if (error instanceof PromptCancelledError) {
    cancel('Cancelled.',);
    return;
  }
  if (error instanceof AdbError) {
    log.error(error.message,);
    process.exitCode = 1;
    return;
  }
  throw error;
}

try {
  await runCli();
} catch (error) {
  handleTopLevelError({ error, },);
}
