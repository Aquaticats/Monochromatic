/**
 UI-bound session pieces: progress display and Escape cancellation.

 @module
 */

import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { BashPokeSettings, } from './config-types.ts';
import { PROGRESS_WIDGET_KEY, } from './constants.ts';
import { createEscapeListener, } from './escape-intercept.ts';
import type { JobRegistry, } from './job-registry.ts';
import type { RunningJob, } from './job-runner.ts';
import { bashPokeLogger, } from './logger.ts';
import {
  createProgressView,
  type ProgressSurface,
  type ProgressView,
} from './progress-widget.ts';

//region Types

/**
 Pieces that hold a reference to one Pi context and must die with it.
 
 Pi invalidates an extension context on reload and on session replacement, so
 anything capturing a context is rebuilt rather than reused across sessions.
 */
type SessionBinding = {
  /**
   Progress display drawn above the editor.
   */
  readonly progress: ProgressView;

  /**
   Releases the terminal input listener and clears the widget.
   */
  readonly dispose: () => void;
};

//endregion Types

//region Binding

/**
 Builds the progress display and Escape cancellation for one Pi context.
 
 Escape cancellation exists only in the interactive TUI, because raw terminal
 input is the sole place a lone Escape can be observed and other modes deliver
 no keystrokes at all.
 
 @param ctx - Pi context whose UI and idle state are read
 
 @param settings - loaded settings controlling display and cancellation
 
 @param registry - tracked jobs the display reads and Escape cancels
 
 @param now - clock, injectable so throttling is deterministic in tests
 
 @returns binding to dispose when the context is replaced
 
 @example
 ```ts
 const binding = createSessionBinding({ ctx, settings, registry, },);
 binding.dispose();
 ```
 */
function createSessionBinding(
  {
    ctx,
    settings,
    registry,
    now,
  }: {
    readonly ctx: ForeignBorrowed<ExtensionContext>;
    readonly settings: BashPokeSettings;
    readonly registry: JobRegistry;
    readonly now?: () => number;
  },
): SessionBinding {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: createSessionBinding.name,
    l: bashPokeLogger,
  }, );

  /**
   Widget drawing surface, which owns the widget key so the view only supplies
   lines and never handles Pi's optional-content signature.
   */
  const surface: ProgressSurface = {
    draw(lines: readonly string[], ): void {
      ctx.ui
        .setWidget(
        PROGRESS_WIDGET_KEY,
        [...lines, ],
        );
    },
    clear(): void {
      ctx.ui
        .setWidget(
        PROGRESS_WIDGET_KEY,
        undefined,
        );
    },
  };

  /**
   Progress display, inert when the host offers no widget surface.
   */
  const progress = createProgressView({
    surface,
    enabled: settings.progressWidget && ctx.hasUI,
    tailLines: settings.progressTailLines,
    refreshMs: settings.progressRefreshMs,
    tickMs: settings.progressTickMs,
    jobs: function listJobs(): readonly RunningJob[] {
      return registry.list();
    },
    ...(now === undefined ? {} : { now, }),
  }, );

  /**
   Terminal input unsubscribe, present only in the interactive TUI because other
   modes deliver no keystrokes to observe.
   */
  const unsubscribe = ctx.mode === 'tui'
    ? ctx.ui
      .onTerminalInput(createEscapeListener({
        gate: {
          isIdle: function isIdle(): boolean {
            return ctx.isIdle();
          },
          editorText: function editorText(): string {
            return ctx.ui
              .getEditorText();
          },
          jobCount: function jobCount(): number {
            return registry.size();
          },
        },
        onCancel(): void {
        /**
         Jobs this keystroke cancelled, reported so the effect is visible.
         */
          const cancelled = registry.cancelAll();
          l.debug(`escape cancelled ${String(cancelled)} background job(s)`, );
          ctx.ui
            .notify(
            `Cancelled ${String(cancelled)} background command(s)`,
            'info',
            );
          progress.refresh();
        },
      }, ), )
    : undefined;

  return {
    progress,
    dispose(): void {
      if (unsubscribe !== undefined)
        unsubscribe();
      progress.clear();
      l.debug('disposed session binding', );
    },
  };
}

//endregion Binding

export { createSessionBinding, };

export type { SessionBinding, };
