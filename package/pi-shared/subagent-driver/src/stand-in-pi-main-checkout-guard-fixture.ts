/**
 The stand-in's part in the guard that keeps a run's `edit` and `write` calls out of the main checkout: it plays Pi's side of the
 handover, so a test sees what a real Pi does with the folders the driver hands over.

 A real Pi loads the guard's extension, which reads the folders the driver put in its environment and registers one `tool_call`
 handler. The stand-in does the same with the same module, so the handover, the arming, and Pi's own order of hooks are all
 exercised by one fixture.

 A stand-in whose driver handed it no folders plays a Pi with no guard at all, which is what the runs of 2026-11-27 were: the play
 says so rather than dying of it, so a test can show the fault the guard answers.

 @module
 */

import { env, } from 'node:process';
import type {
  ExtensionAPI,
  ToolCallEvent,
} from '@earendil-works/pi-coding-agent';
import {
  armTheMainCheckoutGuard,
  MAIN_CHECKOUT_GUARD_COMMAND,
  type MainCheckoutGuard,
  mainCheckoutGuardOfPiEnvironment,
  type PiRecord,
} from '../dist/final/node/index.mjs';

/**
 One handler the extension registered, called with the event and the context Pi hands it.
 */
type StandInGuardHandler = (
  event: never,
  ctx: never,
) => unknown;

/**
 What the guard answered one tool call.
 */
export type RefusalOfOneCall =
  | { readonly refusal: string }
    | { readonly allowed: true };

/**
 The records a stand-in's Pi keeps of what the guard registered.
 */
export type StandInMainCheckoutGuard = {
  /**
   The folders this stand-in's Pi guards, or that it guards none because its driver handed it none.
   */
  readonly armed: boolean;

  /**
   The commands the extension registered, as Pi's command list answers them.
   */
  readonly commands: readonly PiRecord[];

  /**
   What the guard answers one tool call with: the refusal's words, or that it refuses nothing.
   */
  readonly refusalOf: (call: {
    readonly toolName: string;
    readonly input: unknown;
  },) => Promise<RefusalOfOneCall>;
};

/**
 A stand-in that guards nothing, which is what a Pi whose driver handed it no folders registers.
 */
const NO_GUARD: StandInMainCheckoutGuard = {
  armed: false,
  commands: [],
  refusalOf: function refuseNothing() {
    return Promise.resolve({ allowed: true, },);
  },
};

/**
 What this stand-in's Pi armed, kept in a holder so every play reads one place without a binding that changes under it.
 */
const armed: { guard: StandInMainCheckoutGuard } = { guard: NO_GUARD, };

/**
 What this stand-in's Pi guards.

 @returns the guard, or one that refuses nothing

 @example
 ```ts
 const guard = standInMainCheckoutGuard();
 ```
 */
export function standInMainCheckoutGuard(): StandInMainCheckoutGuard {
  return armed.guard;
}

/**
 Reads the folders the driver handed this stand-in over in its environment.

 @returns the guard the driver wrote, or a guard that refuses nothing when the driver handed none

 @example
 ```ts
 const guard = guardOfTheStandInsEnvironment();
 ```
 */
function guardOfTheStandInsEnvironment(): MainCheckoutGuard {
  try {
    return mainCheckoutGuardOfPiEnvironment({ environment: env, },);
  }
  catch (error) {
    // A driver that handed nothing over is a run whose guard cannot be in place, which the stand-in plays rather than dies of.
    void error;
    return {
      armed: false,
      why: 'the driver handed this Pi no folders to guard',
    };
  }
}

/**
 Arms the guard this stand-in's Pi was handed, and keeps what it registered for the plays.

 @returns what the guard registered, or a guard that refuses nothing when the driver handed none

 @example
 ```ts
 const guard = armTheStandInMainCheckoutGuard();
 ```
 */
export function armTheStandInMainCheckoutGuard(): StandInMainCheckoutGuard {
  /**
   The folders the driver wrote for this run.
   */
  const guard = guardOfTheStandInsEnvironment();
  if (!guard.armed) {
    armed.guard = NO_GUARD;
    return armed.guard;
  }
  /**
   Every handler the extension registered, by the event it asked for.
   */
  const handlers = new Map<string, StandInGuardHandler>();
  /**
   Every command the extension registered.
   */
  const commands: PiRecord[] = [];
  /**
   The extension API this stand-in plays for it: it records what the extension asks Pi for and does nothing else.
   */
  /* oxlint-disable typescript/no-unsafe-type-assertion -- the stand-in plays Pi's own extension API, and the two members this
     extension registers are the only ones it could exercise; a fake of every method Pi has would be a second Pi. */
  const api = {
    on: function on(
      event: string,
      handler: StandInGuardHandler,
    ) {
      handlers.set(
        event,
        handler,
      );
      return function off(): void {
        handlers.delete(event,);
      };
    },
    registerCommand: function registerCommand(
      name: string,
      options: { readonly description?: string },
    ) {
      commands.push({
        name,
        description: options.description,
        source: 'extension',
      },);
    },
  } as unknown as ExtensionAPI;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  armTheMainCheckoutGuard({
    pi: api,
    guard,
    ...(env.HOME === undefined ? {} : { home: env.HOME, }),
  },);
  armed.guard = {
    armed: true,
    commands,
    /* oxlint-disable typescript/no-unsafe-type-assertion -- this plays Pi's own hook pipeline, whose handler type is Pi's; the
       event below is built to the fields this extension reads. */
    refusalOf: async function refusalThroughTheExtension(call,): Promise<RefusalOfOneCall> {
      /**
       The handler the extension registered for Pi's tool call hook.
       */
      const handler = handlers.get('tool_call',);
      if (handler === undefined) {
        return { allowed: true, };
      }
      /**
       What Pi's own hook answered with, which is a refusal or nothing at all.
       */
      const answered = await handler(
        {
          type: 'tool_call',
          toolCallId: 'call-1',
          toolName: call.toolName,
          input: call.input,
        } as ToolCallEvent as never,
        // The session's working folder, which Pi hands the handler as the context it works in.
        { cwd: process.cwd(), } as never,
      );
      if ((typeof answered) !== 'object') {
        return { allowed: true, };
      }
      if (answered === null) {
        return { allowed: true, };
      }
      if (!('block' in answered)) {
        return { allowed: true, };
      }
      if (answered.block !== true) {
        return { allowed: true, };
      }
      return { refusal: refusalTextOf({ answered, },), };
    },
    /* oxlint-enable typescript/no-unsafe-type-assertion */
  };
  return armed.guard;
}

/**
 Reads the words of the refusal Pi's own answer carries.

 @param answered - what the `tool_call` handler answered, which this fixture has already found to block

 @returns the refusal's words, or empty text when the answer names none
 */
function refusalTextOf({ answered, }: { readonly answered: { readonly block?: unknown } },): string {
  if (!('reason' in answered)) {
    return '';
  }
  return ((typeof answered.reason) === 'string') ? answered.reason : '';
}

/**
 The commands the guard's extension registered, as Pi's command list answers them.

 @param guard - what the stand-in's Pi armed

 @returns the command list entries the extension added

 @example
 ```ts
 const commands = guardCommandsOf({ guard, });
 ```
 */
export function guardCommandsOf({ guard, }: { readonly guard: StandInMainCheckoutGuard },): readonly PiRecord[] {
  return guard.commands
    .filter(function isTheGuardCommand(command,) {
      return command.name === MAIN_CHECKOUT_GUARD_COMMAND;
    },);
}
