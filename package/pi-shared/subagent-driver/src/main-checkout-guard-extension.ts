/**
 The extension a run's Pi loads before it takes a prompt: it refuses an `edit` or `write` call that names the main checkout of the
 repository the run works in, and lets every other call through.

 The driver hands it the folders on {@link MAIN_CHECKOUT_GUARD_VARIABLE} and loads it with `--extension`, beside the extension that
 masks secret values. A Pi the driver did not start loads neither, and a run started in no worktree is handed the word `off`, so
 nothing is refused for either.

 What this cannot refuse is a shell command that changes the main checkout: `bash` calls are not read here, because a shell command
 that reads the checkout is what a brief tells a run to do, and telling a read from a write needs the shell grammar read as a
 whole (`README.md`, "A shell command that changes the main checkout").

 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  ToolCallEvent,
} from '@earendil-works/pi-coding-agent';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  guardArmedDescription,
  MAIN_CHECKOUT_GUARD_COMMAND,
} from './main-checkout-guard-in-pi.ts';
import type { MainCheckoutGuard, } from './main-checkout-guard-of-a-run.ts';
import { mainCheckoutGuardOfPiEnvironment, } from './main-checkout-guard-words.ts';
import {
  NOT_REFUSED,
  refusalOfAnEditOrWrite,
} from './refusal-of-a-write-to-the-main-checkout.ts';

//region Types

/**
 What arming the guard needs.
 */
export type ArmingOfTheMainCheckoutGuard = {
  /**
   The extension API Pi hands its extensions, which owns what a handler can do with it.
   */
  readonly pi: ForeignHostCapability<ExtensionAPI>;

  /**
   The folders this run may not change, or that it has none.
   */
  readonly guard: MainCheckoutGuard;

  /**
   The home folder a `~` path names, which a test names for its own disposable home.
   */
  readonly home?: string;
};

//endregion Types

//region The extension

/**
 Reports the guard, for a person who asks the command what this run may not change.

 Nothing is printed: the command's description holds what there is to say.
 */
function reportTheGuard(): Promise<void> {
  return Promise.resolve();
}

/**
 Arms the guard in one Pi: the hook that refuses a change of the main checkout, and the command that says which folders it holds.

 A guard with nothing to protect registers nothing at all, so a run started in no worktree takes exactly the calls it took before
 this extension existed.

 @param pi - the extension API Pi hands its extensions

 @param guard - the folders this run may not change, or that it has none

 @param home - the home folder a `~` path names

 @example
 ```ts
 armTheMainCheckoutGuard({ pi, guard, home: '/home/a' });
 ```
 */
export function armTheMainCheckoutGuard({
  pi,
  guard,
  home,
}: ArmingOfTheMainCheckoutGuard,): void {
  if (!guard.armed) {
    return;
  }
  pi.on(
    'tool_call',
    async function refuseAChangeOfTheMainCheckout(
      event: ForeignBorrowed<ToolCallEvent>,
      ctx: ExtensionContext,
    ) {
      /**
       What this call would change, when it names the main checkout.
       */
      const refusal = await refusalOfAnEditOrWrite({
        toolName: event.toolName,
        input: event.input,
        cwd: ctx.cwd,
        guard,
        ...(home === undefined ? {} : { home, }),
      },);
      if (refusal === NOT_REFUSED) {
        return undefined;
      }
      return refusal;
    },
  );
  pi.registerCommand(
    MAIN_CHECKOUT_GUARD_COMMAND,
    {
      description: guardArmedDescription({ guard, },),
      handler: reportTheGuard,
    },
  );
}

/**
 The extension Pi loads for a run: it reads the folders the driver wrote in its environment and arms the guard.

 @param pi - the extension API Pi hands its extensions

 @throws MainCheckoutGuardUnreadableError when the run handed it no folders, which stops a run whose guard would otherwise be off

 @example
 ```ts
 // Pi loads this file with `--extension` and calls what it exports with itself.
 ```
 */
export default function mainCheckoutGuard(pi: ExtensionAPI,): void {
  armTheMainCheckoutGuard({
    pi,
    guard: mainCheckoutGuardOfPiEnvironment({ environment: process.env, },),
  },);
}

//endregion The extension
