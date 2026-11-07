/**
 The gate: take one of the host-wide slots, wait only while every slot is held by a live process, run the command in the slot,
 and give the slot back.

 Each slot is a numbered claim series (`src/owner-claim.ts`), so a free slot is taken with one exclusive `link` and a slot
 whose holder died is taken over by the next generation without a person. The cap is the slot count: a gate holds exactly one
 slot's claim for as long as its command runs, and the kernel lock on that slot's `slot-<n>.lock` file too, taken once without
 waiting after the claim and given back before it, so a waiting gate of the earlier release (which polls that lock and reads no
 claim) is refused (`src/legacy-flock.ts`). When all slots are held the gate says so, naming the holders, and waits for a
 change in the slot directory (a release renames a file there) with a bounded re-check as the fallback, because a holder that
 was killed makes no event. The wait is never a tight poll: it ends at the next change, the next notice or the re-check,
 whichever is first.
 */
import { setTimeout as sleep } from 'node:timers/promises';

import { noticeGraceMs } from './constants.ts';
import {
  type DirectoryWatch,
  watchDirectoryChanges,
} from './directory-watch.ts';
import {
  type GateSlotOptions,
  slotClaimPrefix,
  slotLockPath,
} from './gate-slot.ts';
import { probeLegacyLock } from './legacy-flock.ts';
import {
  liveOwnerOf,
  type OwnerClaim,
  takeClaim,
} from './owner-claim.ts';
import { noLiveOwner } from './claim-liveness.ts';
import { holdLegacyLockOrGiveBack } from './legacy-hold-or-give-back.ts';

/**
 Milliseconds in a second, to say how long a gate has waited.
 */
const millisecondsPerSecond = 1_000;

/**
 A slot held by a live process of this release.
 */
export type ProcessHolder = {
  /**
   Marks the holder as a process with a claim.
   */
  readonly kind: 'process';
  /**
   The slot, counted from one.
   */
  readonly slot: number;
  /**
   The holder's process id.
   */
  readonly pid: number;
};

/**
 A slot held by a gate of the earlier release, which holds a kernel lock on the slot file.
 */
export type EarlierReleaseHolder = {
  /**
   Marks the holder as a gate of the earlier release.
   */
  readonly kind: 'earlier-release';
  /**
   The slot, counted from one.
   */
  readonly slot: number;
};

/**
 Who holds a slot.
 */
export type SlotHolder = ProcessHolder | EarlierReleaseHolder;

/**
 A slot this gate took.
 */
export type SlotTaken = {
  /**
   Marks the attempt as having taken a slot.
   */
  readonly kind: 'taken';
  /**
   The slot, counted from one.
   */
  readonly slot: number;
  /**
   The claim, to release when the command ends.
   */
  readonly claim: OwnerClaim;
  /**
   The kernel lock on the slot file, which keeps a gate of the earlier release that polls the slot from starting a command
   in it; released with the claim.
   */
  readonly legacyLock: AsyncDisposable;
};

/**
 Every slot is held.
 */
export type SlotsFull = {
  /**
   Marks the attempt as having found every slot held.
   */
  readonly kind: 'full';
  /**
   The holder of each slot, in slot order.
   */
  readonly holders: readonly SlotHolder[];
};

/**
 Runs the gated command in a slot and returns its exit status.
 */
export type RunCommand = (parameters: {
  readonly slot: number;
  readonly command: readonly string[]
}) => Promise<number>;

/**
 Everything {@link runInSlot} needs besides the settings, each replaceable by a test.
 */
export type GateEnvironment = {
  /**
   Runs the command in the slot.
   */
  readonly runCommand: RunCommand;
  /**
   Starts watching the slot directory; absent to use the real watch.
   */
  readonly watchDirectory?: (directory: string) => DirectoryWatch;
  /**
   Writes a notice; absent to write to standard error.
   */
  readonly write?: (text: string) => void;
  /**
   The clock in milliseconds; absent to use the real one.
   */
  readonly now?: () => number;
};

/**
 Where a wait stands: when it began, when its next notice is due, and whether a notice was written yet.
 */
type WaitClock = {
  readonly startedAt: number;
  readonly nextNoticeAt: number;
  readonly noticed: boolean;
};

/**
 Try the slots in order from one, probing the earlier release's lock file first, and take the first free one.

 @param options - slot settings
 
 @param slot - the slot to try now
 
 @param holders - who held the slots tried so far

 @returns the slot taken, or who holds every slot

 @example
 ```ts
 await tryFrom({ options, slot: 1, holders: [] });
 ```
 */
async function tryFrom({
  options,
  slot,
  holders,
}: {
  readonly options: GateSlotOptions;
  readonly slot: number;
  readonly holders: readonly SlotHolder[];
}): Promise<SlotTaken | SlotsFull> {
  if (slot > options.slotCount)
    return {
      kind: 'full',
      holders,
    };
  /**
   The earlier release's lock file of this slot.
   */
  const lockFile = slotLockPath({
    lockDirectory: options.lockDirectory,
    slot,
  });
  // A gate of the earlier release holds a kernel lock on the slot file instead of a claim, and a gate of this release holds
  // that lock too while its command runs: a held lock is a live owner either way. The file is never removed, because a gate of
  // the earlier release that is still waiting creates and locks it again on each poll.
  /**
   What the probe found at the earlier release's lock file.
   */
  const standing = await probeLegacyLock({
    path: lockFile,
    retireWhenFree: false,
  });
  if (standing === 'held')
    return await tryFrom({
      options,
      slot: slot + 1,
      holders: [
        ...holders,
        await holderOfLockedSlot({
          options,
          slot,
        }),
      ],
    });
  /**
   What taking this slot's claim came to.
   */
  const outcome = await takeClaim({
    directory: options.lockDirectory,
    prefix: slotClaimPrefix(slot),
  });
  if (outcome.kind === 'held') {
    /**
     The kernel lock a gate of the earlier release would poll for, kept for as long as the claim.
     */
    const hold = await holdLegacyLockOrGiveBack({
      path: lockFile,
      claim: outcome.claim,
    });
    if (hold.kind === 'kept')
      return {
        kind: 'taken',
        slot,
        claim: outcome.claim,
        legacyLock: hold.lock,
      };
    // A gate of the earlier release took the lock after the probe: give the claim back, and count the slot as held by it.
    await outcome.claim
      .release();
    return await tryFrom({
      options,
      slot: slot + 1,
      holders: [
        ...holders,
        {
          kind: 'earlier-release',
          slot,
        },
      ],
    });
  }
  return await tryFrom({
    options,
    slot: slot + 1,
    holders: [
      ...holders,
      {
        kind: 'process',
        slot,
        pid: outcome.owner
          .pid,
      },
    ],
  });
}

/**
 Name who holds a slot whose lock file is locked: a live process with a claim in the slot, which holds the lock too, or else a
 gate of the earlier release.

 @param options - slot settings

 @param slot - the slot

 @returns the holder

 @throws CargoMutantsToolingError when the slot's claim cannot be judged

 @example
 ```ts
 await holderOfLockedSlot({ options, slot: 1 });
 ```
 */
async function holderOfLockedSlot({
  options,
  slot,
}: {
  readonly options: GateSlotOptions;
  readonly slot: number;
}): Promise<SlotHolder> {
  /**
   The live owner of the slot's claim, if any.
   */
  const owner = await liveOwnerOf({
    directory: options.lockDirectory,
    prefix: slotClaimPrefix(slot),
  });
  if (owner === noLiveOwner)
    return {
      kind: 'earlier-release',
      slot,
    };
  return {
    kind: 'process',
    slot,
    pid: owner.pid,
  };
}

/**
 Take the lowest free slot at once, or report who holds every slot. Nothing waits.

 @param options - slot settings

 @returns the slot taken, or the holder of each slot

 @throws CargoMutantsToolingError when a slot's claim or the earlier release's lock cannot be judged

 @example
 ```ts
 const attempt = await takeAnySlot({ options });
 ```
 */
export async function takeAnySlot({ options }: { readonly options: GateSlotOptions }): Promise<SlotTaken | SlotsFull> {
  return await tryFrom({
    options,
    slot: 1,
    holders: [],
  });
}

/**
 Say who holds a slot.

 @param holder - the holder

 @returns the words for a notice

 @example
 ```ts
 describeHolder({ kind: 'process', slot: 1, pid: 42 });
 ```
 */
function describeHolder(holder: SlotHolder): string {
  if (holder.kind === 'earlier-release')
    return `slot ${String(holder.slot)}: a gate of the earlier release, which holds the kernel lock on its slot file`;
  return `slot ${String(holder.slot)}: process ${String(holder.pid)}`;
}

/**
 Write the notice of a gate that keeps waiting.

 @param options - slot settings
 
 @param holders - who holds every slot
 
 @param elapsedMs - how long the gate has waited
 
 @param first - true for the first notice of the wait

 @returns the text, one line

 @example
 ```ts
 noticeText({ options, holders, elapsedMs: 0, first: true });
 ```
 */
function noticeText({
  options,
  holders,
  elapsedMs,
  first,
}: {
  readonly options: GateSlotOptions;
  readonly holders: readonly SlotHolder[];
  readonly elapsedMs: number;
  readonly first: boolean;
}): string {
  /**
   The holders, in slot order.
   */
  const held = holders.map(describeHolder)
    .join('; ');
  /**
   How long the wait has lasted, in whole seconds.
   */
  const seconds = String(Math.floor(elapsedMs / millisecondsPerSecond));
  if (first)
    return `gate-slot: waiting for a free slot in ${options.lockDirectory}: all ${String(options.slotCount)} slots are held (${held}); a slot frees when its holder ends.\n`;
  return `gate-slot: still waiting for a free slot in ${options.lockDirectory} after ${seconds} seconds (${held}).\n`;
}

/**
 Wait for a change in the watched directory or the timeout, whichever is first.

 @param watch - the watch, started before the slots were looked at
 
 @param timeoutMs - the longest to wait

 @example
 ```ts
 await waitForChange({ watch, timeoutMs: 15_000 });
 ```
 */
async function waitForChange({
  watch,
  timeoutMs,
}: {
  readonly watch: DirectoryWatch;
  readonly timeoutMs: number;
}): Promise<void> {
  /**
   Cancels the timer once the watch fires, so no timer outlives the wait.
   */
  const stop = new AbortController();
  /**
   The timer, which ends quietly when cancelled.
   */
  const timer = (async function pause(): Promise<void> {
    try {
      await sleep(
        timeoutMs,
        undefined,
        { signal: stop.signal },
      );
    }
    catch (error) {
      if (!stop.signal
        .aborted)
        throw error;
    }
  })();
  await Promise.race([
    watch.changed,
    timer
  ]);
  stop.abort();
  await timer;
}

/**
 What one look at the slots, and the wait after it, came to.
 */
type LookResult = {
  readonly kind: 'taken';
  readonly taken: SlotTaken
} | {
  readonly kind: 'again';
  readonly clock: WaitClock
};

/**
 Look at the slots once and, when all are held, write the notice if one is due and wait for a change or the timeout.

 The watch lives only inside this call, so a long wait never holds more than one.

 @param options - slot settings
 
 @param environment - the watch, the notice sink and the clock
 
 @param clock - when the wait began and when the next notice is due

 @returns the slot taken, or the clock for the next look

 @example
 ```ts
 await lookOnce({ options, environment, clock });
 ```
 */
async function lookOnce({
  options,
  environment,
  clock,
}: {
  readonly options: GateSlotOptions;
  readonly environment: Required<Omit<GateEnvironment, 'runCommand'>>;
  readonly clock: WaitClock;
}): Promise<LookResult> {
  // The watch starts before the slots are looked at: a release between the look and the wait is then an event, not a lost wake.
  /**
   The watch on the slot directory, closed when this look ends.
   */
  using watch = environment.watchDirectory(options.lockDirectory);
  /**
   What the look at the slots found.
   */
  const attempt = await takeAnySlot({ options });
  if (attempt.kind === 'taken')
    return {
      kind: 'taken',
      taken: attempt,
    };
  /**
   The time now.
   */
  const current = environment.now();
  /**
   Whether a notice is due.
   */
  const due = current >= clock.nextNoticeAt;
  if (due)
    environment.write(noticeText({
      options,
      holders: attempt.holders,
      elapsedMs: current - clock.startedAt,
      first: !clock.noticed,
    }));
  /**
   When the next notice is due.
   */
  const nextNoticeAt = due ? current + options.noticeIntervalMs : clock.nextNoticeAt;
  await waitForChange({
    watch,
    timeoutMs: Math.min(
      options.pollIntervalMs,
      Math.max(
        1,
        nextNoticeAt - current,
      ),
    ),
  });
  return {
    kind: 'again',
    clock: {
      startedAt: clock.startedAt,
      nextNoticeAt,
      noticed: clock.noticed || due,
    },
  };
}

/**
 Look at the slots until one is taken, waiting between looks.

 @param options - slot settings
 
 @param environment - the watch, the notice sink and the clock
 
 @param clock - when the wait began and when the next notice is due

 @returns the slot taken

 @example
 ```ts
 await acquireSlot({ options, environment, clock });
 ```
 */
async function acquireSlot({
  options,
  environment,
  clock,
}: {
  readonly options: GateSlotOptions;
  readonly environment: Required<Omit<GateEnvironment, 'runCommand'>>;
  readonly clock: WaitClock;
}): Promise<SlotTaken> {
  /**
   What this look came to.
   */
  const result = await lookOnce({
    options,
    environment,
    clock,
  });
  if (result.kind === 'taken')
    return result.taken;
  return await acquireSlot({
    options,
    environment,
    clock: result.clock,
  });
}

/**
 Write a notice to standard error.

 @param text - the notice

 @example
 ```ts
 writeStderr('gate-slot: waiting\n');
 ```
 */
function writeStderr(text: string): void {
  process.stderr
    .write(text);
}

/**
 Wait for a free slot, run the command in it, and give the slot back.

 @param options - slot settings and the command

 @param runCommand - runs the command in the slot

 @param watchDirectory - starts watching the slot directory; absent to use the real watch

 @param write - writes a notice; absent to write to standard error

 @param now - the clock in milliseconds; absent to use the real one

 @returns the command's exit status

 @throws CargoMutantsToolingError when a slot's claim or the earlier release's lock cannot be judged

 @example
 ```ts
 const status = await runInSlot({ options, runCommand: async ({ command }) => runNicedCommand({ command }) });
 ```
 */
export async function runInSlot({
  options,
  runCommand,
  watchDirectory = watchDirectoryChanges,
  write = writeStderr,
  now = Date.now,
}: GateEnvironment & { readonly options: GateSlotOptions }): Promise<number> {
  /**
   When the wait began.
   */
  const startedAt = now();
  /**
   The slot this gate took.
   */
  const taken = await acquireSlot({
    options,
    environment: {
      watchDirectory,
      write,
      now,
    },
    clock: {
      startedAt,
      nextNoticeAt: startedAt + Math.min(
        noticeGraceMs,
        options.noticeIntervalMs,
      ),
      noticed: false,
    },
  });
  /**
   Gives the slot back when the command ends, however it ends; a failed release is reported, not thrown over the command's
   status, because the claim then names this process until it ends.
   */
  await using held = {
    async [Symbol.asyncDispose](): Promise<void> {
      // The kernel lock goes first, so a gate that sees the claim given back also finds the lock free.
      await taken.legacyLock[Symbol.asyncDispose]();
      try {
        await taken.claim
          .release();
      }
      catch (error) {
        console.error(`The claim ${taken.claim
          .path} could not be released: ${String(error)}`);
      }
    },
  };
  return await runCommand({
    slot: taken.slot,
    command: options.command,
  });
}
