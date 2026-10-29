/**
 Throttled progress display for running background jobs.

 @module
 */

import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { formatTrackedDuration, } from '@monochromatic-dev/numeric-format-core/ts';

import {
  JOB_LINE_PREFIX,
  TAIL_LINE_INDENT,
} from './constants.ts';
import type { RunningJob, } from './job-runner.ts';
import { bashPokeLogger, } from './logger.ts';

//region Types

/**
 Drawing surface this package renders progress onto.
 
 Narrowed to draw and clear so this module never handles the widget key or
 Pi's optional-content signature, and so a test can supply two closures.
 */
type ProgressSurface = {
  /**
   Replaces the displayed lines.
   */
  readonly draw: (lines: readonly string[]) => void;

  /**
   Removes the display entirely.
   */
  readonly clear: () => void;
};

/**
 Handle on the progress display for one extension instance.
 */
type ProgressView = {
  /**
   Requests a redraw, coalescing requests inside the refresh interval.
   */
  readonly refresh: () => void;

  /**
   Removes the display and drops any pending redraw.
   */
  readonly clear: () => void;
};

/**
 Mutable throttle state, kept in an object so no function-root binding mutates.
 */
type ThrottleState = {
  /**
   Timestamp of the last completed draw, zero before the first one.
   */
  lastDraw: number;

  /**
   Pending coalesced redraw, absent when none is scheduled.
   */
  timer?: NodeJS.Timeout;

  /**
   Elapsed-time ticker, armed only while at least one job is displayed.
   */
  ticker?: NodeJS.Timeout;
};

//endregion Types

//region Rendering

/**
 Builds display lines for the jobs currently running.
 
 @param jobs - tracked jobs in start order
 
 @param tailLines - output lines shown per job
 
 @param now - timestamp the elapsed values are measured against
 
 @returns lines to draw, empty when nothing is running
 
 @example
 ```ts
 renderProgressLines({ jobs: [], tailLines: 3, now: 0, },);
 ```
 */
function renderProgressLines(
  {
    jobs,
    tailLines,
    now,
  }: {
    readonly jobs: readonly RunningJob[];
    readonly tailLines: number;
    readonly now: number;
  },
): string[] {
  return jobs.flatMap(function renderJob(job: RunningJob, ): string[] {
    /**
     Whole seconds since the job started, the unit the shared formatter takes.
     */
    const elapsedSeconds = Math.floor((now - job.startedAt) / MS_PER_SECOND, );

    /**
     Heading naming the command and its elapsed time, so a job is identifiable
     by more than its output alone.
     */
    const heading = `${JOB_LINE_PREFIX}${job.command} (${formatTrackedDuration(elapsedSeconds, )})`;

    /**
     Freshest output lines, indented so they read as belonging to the heading.
     */
    const tail: string[] = job.recorder
      .tailLines(tailLines, )
      .map(function indent(line: string, ): string {
        return `${TAIL_LINE_INDENT}${line}`;
      }, );
    return [
      heading,
      ...tail,
    ];
  }, );
}

//endregion Rendering

//region View

/**
 Creates the progress display for one extension instance.
 
 Redraws are coalesced because a chatty command emits far more chunks per
 second than a terminal can usefully paint, and painting each one starves the
 renderer that is also drawing the transcript.
 
 @param surface - drawing surface, usually Pi's widget area
 
 @param enabled - whether progress is drawn at all
 
 @param tailLines - output lines shown per job
 
 @param refreshMs - minimum milliseconds between draws
 
 @param tickMs - milliseconds between elapsed-time redraws, zero to disable them
 
 @param jobs - reads the tracked jobs at draw time
 
 @param now - clock, injectable so throttling is deterministic in tests
 
 @returns handle used to request redraws and to clear the display
 
 @example
 ```ts
 const view = createProgressView({
   surface: { draw() {}, clear() {}, },
   enabled: true,
   tailLines: 3,
   refreshMs: 250,
   tickMs: 1000,
   jobs: () => [],
 },);
 view.refresh();
 view.clear();
 ```
 */
function createProgressView(
  {
    surface,
    enabled,
    tailLines,
    refreshMs,
    tickMs,
    jobs,
    now = Date.now,
  }: {
    readonly surface: ProgressSurface;
    readonly enabled: boolean;
    readonly tailLines: number;
    readonly refreshMs: number;
    readonly tickMs: number;
    readonly jobs: () => readonly RunningJob[];
    readonly now?: () => number;
  },
): ProgressView {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: createProgressView.name,
    l: bashPokeLogger,
  }, );

  /**
   Throttle state shared by refresh, draw, and clear.
   */
  const state: ThrottleState = { lastDraw: 0, };

  /**
   Arms the elapsed-time ticker, which a silent job such as a long sleep needs
   because output-driven redraws never arrive for it.
   */
  function armTicker(): void {
    if ((state.ticker !== undefined) || (tickMs <= 0))
      return;
    state.ticker = setInterval(
      function onTick(): void {
        draw();
      },
      tickMs,
    );
    // A ticking clock must never be the reason a process stays alive.
    state.ticker
      .unref();
  }

  /**
   Disarms the ticker once nothing is displayed.
   */
  function disarmTicker(): void {
    if (state.ticker === undefined)
      return;
    clearInterval(state.ticker, );
    delete state.ticker;
  }

  /**
   Paints the current job list, or clears the display when nothing is running.
   */
  function draw(): void {
    state.lastDraw = now();

    /**
     Lines for the jobs running at this instant.
     */
    const lines = renderProgressLines({
      jobs: jobs(),
      tailLines,
      now: state.lastDraw,
    }, );
    if (lines.length === 0) {
      disarmTicker();
      surface.clear();
    }
    else {
      armTicker();
      surface.draw(lines, );
    }
    l.debug(`drew progress with ${String(lines.length)} lines`, );
  }

  /**
   Schedules one coalesced redraw, ignoring requests while one is pending.
   
   @param delayMs - milliseconds until the redraw
   */
  function scheduleDraw(delayMs: number, ): void {
    if (state.timer !== undefined)
      return;
    state.timer = setTimeout(
      function onCoalescedDraw(): void {
        delete state.timer;
        draw();
      },
      delayMs,
    );
    // A pending redraw must never be the reason a process stays alive.
    state.timer
      .unref();
  }

  return {
    refresh(): void {
      if (!enabled)
        return;

      /**
       Milliseconds since the last completed draw.
       */
      const elapsed = now() - state.lastDraw;
      if ((state.lastDraw === 0) || (elapsed >= refreshMs)) {
        draw();
        return;
      }
      scheduleDraw(refreshMs - elapsed, );
    },

    clear(): void {
      if (state.timer !== undefined) {
        clearTimeout(state.timer, );
        delete state.timer;
      }
      disarmTicker();
      if (!enabled)
        return;
      surface.clear();
      l.debug('cleared progress display', );
    },
  };
}

//endregion View

export {
  createProgressView,
  renderProgressLines,
};

export type {
  ProgressSurface,
  ProgressView,
  ThrottleState,
};
