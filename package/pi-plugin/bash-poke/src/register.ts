/**
 Extension wiring for bash-poke.

 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  SessionStartEvent,
  UserBashEvent,
  UserBashEventResult,
} from '@earendil-works/pi-coding-agent';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { BashPokeSettings, } from './config-types.ts';
import { PENDING_NOTE, } from './constants.ts';
import { createJobRegistry, } from './job-registry.ts';
import {
  startJob,
  type RunningJob,
} from './job-runner.ts';
import { bashPokeLogger, } from './logger.ts';
import { registerPokeRenderer, } from './message-renderer.ts';
import { deliverPoke, } from './poke-delivery.ts';
import {
  buildPokeContent,
  jobOutputText,
  type PokeDetails,
} from './poke-text.ts';
import {
  createSessionBinding,
  type SessionBinding,
} from './session-binding.ts';
import type { ShellInvocation, } from './shell-resolution.ts';

//region Types

/**
 Placeholder Pi records in context while a job is still running.
 
 Shaped like Pi's bash result without importing its type, which the package
 root does not export.
 */
type PendingBashResult = {
  /**
   Text the transcript and the model see until the poke arrives.
   */
  readonly output: string;

  /**
   Exit code, absent because the command has not finished.
   */
  readonly exitCode: undefined;

  /**
   Cancellation flag, false because nothing was cancelled yet.
   */
  readonly cancelled: boolean;

  /**
   Truncation flag, false because the placeholder is already short.
   */
  readonly truncated: boolean;
};

/**
 Everything needed to wire one extension instance.
 */
type Registration = {
  /**
   Pi extension API receiving the registrations.
   */
  readonly pi: ForeignBorrowed<ExtensionAPI>;

  /**
   Loaded settings controlling content, display, and cancellation.
   */
  readonly settings: BashPokeSettings;

  /**
   Resolved shell, injected so registration stays synchronous and testable.
   */
  readonly shell: ShellInvocation;

  /**
   Temp root for spool files, injectable so tests use disposable directories.
   */
  readonly tmp?: string;

  /**
   Clock, injectable so elapsed time is deterministic in tests.
   */
  readonly now?: () => number;
};

//endregion Types

//region Placeholder

/**
 Builds the result Pi records while a job runs in the background.
 
 @returns placeholder bash result
 
 @example
 ```ts
 pendingBashResult();
 ```
 */
function pendingBashResult(): PendingBashResult {
  return {
    output: PENDING_NOTE,
    exitCode: undefined,
    cancelled: false,
    truncated: false,
  };
}

//endregion Placeholder

//region Registration

/**
 Registers bash-poke against one Pi extension API.
 
 @param pi - Pi extension API receiving the registrations
 
 @param settings - loaded settings
 
 @param shell - resolved shell invocation
 
 @param tmp - temp root for spool files
 
 @param now - clock used for start timestamps
 
 @mutates pi - one renderer registration and two event registrations are stored in the Pi host
 
 @example
 ```ts
 registerBashPoke({ pi, settings, shell, },);
 ```
 */
function registerBashPoke(
  {
    pi,
    settings,
    shell,
    tmp,
    now,
  }: Registration,
): void {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: registerBashPoke.name,
    l: bashPokeLogger,
  }, );

  /**
   Jobs started by this extension instance.
   */
  const registry = createJobRegistry();

  /**
   Current UI binding, replaced whenever Pi replaces the context.
   */
  const bindings: { current?: SessionBinding; } = {};

  /**
   Returns the live binding, building one from a context when none exists.
   
   @param ctx - Pi context to bind when no binding is live
   
   @returns binding for the current session
   */
  function bindingFor(ctx: ForeignBorrowed<ExtensionContext>, ): SessionBinding {
    /**
     Binding already live for this session, absent before the first use.
     */
    const existing = bindings.current;
    if (existing !== undefined)
      return existing;
    l.debug('building session binding', );

    /**
     Binding built from the context Pi handed this handler.
     */
    const created = createSessionBinding({
      ctx,
      settings,
      registry,
      ...(now === undefined ? {} : { now, }),
    }, );
    bindings.current = created;
    return created;
  }

  /**
   Releases the live binding so a replaced context is never used again.
   */
  function disposeBinding(): void {
    /**
     Binding to release, absent when no session ever bound.
     */
    const existing = bindings.current;
    if (existing === undefined)
      return;
    existing.dispose();
    delete bindings.current;
  }

  /**
   Waits for one job, then reports its outcome unless the user cancelled it.
   
   @param job - job whose completion is awaited
   
   @param binding - UI binding used to refresh progress
   */
  async function reportCompletion(
    {
      job,
      binding,
    }: {
      readonly job: RunningJob;
      readonly binding: SessionBinding;
    },
  ): Promise<void> {
    /**
     Outcome known only once the child exits and its spool file is closed.
     */
    const outcome = await job.finished;
    registry.remove(job.id, );
    binding.progress
      .refresh();

    if (outcome.cancelled) {
      l.debug(`job ${job.id} was cancelled; no poke sent`, );
      return;
    }

    /**
     Bounded view of everything the job printed.
     */
    const snapshot = job.recorder
      .snapshot();

    /**
     Renderer-only outcome data, excluded from model context by Pi.
     */
    const details: PokeDetails = {
      command: job.command,
      cancelled: false,
      truncated: snapshot.truncated,
      elidedChars: snapshot.elidedChars,
      outputChars: snapshot.totalChars,
      ...(outcome.exitCode === undefined ? {} : { exitCode: outcome.exitCode, }),
      ...(job.spoolPath === undefined ? {} : { spoolPath: job.spoolPath, }),
    };

    /**
     Model-facing poke text.
     */
    const content = buildPokeContent({
      details,
      output: jobOutputText({
        output: snapshot.text,
        ...(outcome.spawnError === undefined ? {} : { spawnError: outcome.spawnError, }),
      }, ),
      instruction: settings.pokeInstruction,
    }, );

    /**
     Delivery state, which is a report rather than a throw because a stale
     runtime is an expected outcome for a job that outlived its session.
     */
    const delivery = deliverPoke({
      pi,
      content,
      details,
    }, );
    if (!delivery.delivered)
      l.warn(`poke for ${job.command} was not delivered`, );
  }

  registerPokeRenderer(pi, );

  pi.on(
    'session_start',
    function handleSessionStart(
      event: Readonly<Pick<SessionStartEvent, 'reason'>>,
      ctx: ForeignBorrowed<ExtensionContext>,
    ): void {
      l.debug(`session start (${event.reason}); rebuilding UI binding`, );
      disposeBinding();
      bindingFor(ctx, );
    },
  );

  pi.on(
    'user_bash',
    async function handleUserBash(
      event: ForeignBorrowed<UserBashEvent>,
      ctx: ForeignBorrowed<ExtensionContext>,
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors Pi's ExtensionHandler<UserBashEvent, UserBashEventResult> contract, whose return is `Promise<R | void> | R | void`; returning nothing is how a `!!` command falls through to Pi's own execution, so absence is the external API's own signal rather than this package's.
    ): Promise<UserBashEventResult | undefined> {
      // `!!` is documented as hidden from the model, and taking it over would
      // both change its meaning and lose Pi's own streaming for it.
      if (event.excludeFromContext) {
        l.debug(`leaving hidden command to Pi: ${event.command}`, );
        return undefined;
      }

      /**
       Binding for this session, created on first use when Pi delivered no
       session start, as happens in modes without a startup event.
       */
      const binding = bindingFor(ctx, );

      /**
       Job running detached, whose completion is reported separately so the
       editor gets control back immediately.
       */
      const job = await startJob({
        command: event.command,
        cwd: event.cwd,
        shell,
        headChars: settings.pokeHeadChars,
        tailChars: settings.pokeTailChars,
        killGraceMs: settings.killGraceMs,
        ...(tmp === undefined ? {} : { tmp, }),
        onChunk(): void {
          binding.progress
            .refresh();
        },
        ...(now === undefined ? {} : { now, }),
      }, );
      registry.add(job, );
      binding.progress
        .refresh();
      l.debug(`job ${job.id} started for: ${event.command}`, );
      // Completion is deliberately not awaited: awaiting it would block the
      // editor until the command exits, which is the behavior being replaced.
      void reportCompletion({
        job,
        binding,
      }, );
      return { result: pendingBashResult(), };
    },
  );
}

//endregion Registration

export {
  pendingBashResult,
  registerBashPoke,
};

export type {
  PendingBashResult,
  Registration,
};
