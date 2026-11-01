/**
 Checks of the wide session about the 600 px layout breakpoint, hit at exactly 599 and 600 logical pixels.
 
 Compositor 0.3.0 and older rounded a requested size to the parent window's pixel grid in its captures (599 became
 600), so the breakpoint could not be tested at its edge. From compositor 0.4.0 on, both the size the app sees and the
 screenshot are exactly what was asked for, which the runner's start-up check requires.
 
 @example
 ```ts
 import { checkBreakpoint, } from './wide-breakpoint.ts';
 
 await checkBreakpoint({ context, session, },);
 ```
 
 @packageDocumentation
 */

import type { Action, } from './actions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { loadImage, } from './image.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 The height of the window while the width crosses the breakpoint, in logical pixels.
 */
const BREAKPOINT_HEIGHT = 800;

/**
 The last width that folds the layout, in logical pixels.
 */
const FOLDED_WIDTH = 599;

/**
 The first width that unfolds the layout, in logical pixels.
 */
const UNFOLDED_WIDTH = 600;

/**
 Width of the wide window the session returns to, in logical pixels.
 */
const WIDE_WIDTH = 1_000;

/**
 How long the app gets to lay out again after a resize, in milliseconds.
 */
const RELAYOUT_TIMEOUT_MS = 5_000;

/**
 Condition: the layout is folded.
 
 @param state - a state of the app
 
 @returns whether the layout is folded
 
 @example
 ```ts
 isFolded(state,);
 ```
 */
function isFolded(state: UiState,): boolean {
  return !state.unfolded;
}

/**
 Condition: the layout is unfolded.
 
 @param state - a state of the app
 
 @returns whether the layout is unfolded
 
 @example
 ```ts
 isUnfolded(state,);
 ```
 */
function isUnfolded(state: UiState,): boolean {
  return state.unfolded;
}

/**
 Builds the action that resizes the nested screen.
 
 @param width - new logical width
 
 @returns the action
 
 @example
 ```ts
 resizeTo(599,);
 ```
 */
function resizeTo(width: number,): Action {
  return async function resize(session: Session,): Promise<void> {
    await session.ok(`resize ${width} ${BREAKPOINT_HEIGHT}`,);
  };
}

/**
 Records the two breakpoint checks of the wide session. They go last: the window is left at its wide size.
 
 @param context - the run's context
 
 @param session - the wide session
 
 @example
 ```ts
 await checkBreakpoint({ context, session, },);
 ```
 */
export async function checkBreakpoint(
  {
    context,
    session,
  }: {
    readonly context: RunContext;
    readonly session: Session;
  },
): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;
  recorder.setGroup('layout',);
  await recorder.check({
    id: 'layout/breakpoint-599',
    title: 'a window 599 px wide folds the layout, and the screenshot is exactly 599 px wide',
    body: async function width599Folds(): Promise<string> {
      await press({
        session,
        act: resizeTo(FOLDED_WIDTH,),
        until: isFolded,
        what: 'the layout folding at 599 px',
        timeoutMs: RELAYOUT_TIMEOUT_MS,
      },);

      /**
       The screenshot at the folded width.
       */
      const image = await loadImage(await session.shot('breakpoint-599',),);
      expect({
        condition: (image.width === FOLDED_WIDTH) && (image.height === BREAKPOINT_HEIGHT),
        message: `the screenshot is ${image.width}x${image.height}, not ${FOLDED_WIDTH}x${BREAKPOINT_HEIGHT}`,
      },);
      return `folded at ${image.width}x${image.height}`;
    },
  },);
  await recorder.check({
    id: 'layout/breakpoint-600',
    title: 'a window 600 px wide unfolds the layout, and the screenshot is exactly 600 px wide',
    body: async function width600Unfolds(): Promise<string> {
      await press({
        session,
        act: resizeTo(UNFOLDED_WIDTH,),
        until: isUnfolded,
        what: 'the layout unfolding at 600 px',
        timeoutMs: RELAYOUT_TIMEOUT_MS,
      },);

      /**
       The screenshot at the unfolded width.
       */
      const image = await loadImage(await session.shot('breakpoint-600',),);
      expect({
        condition: (image.width === UNFOLDED_WIDTH) && (image.height === BREAKPOINT_HEIGHT),
        message: `the screenshot is ${image.width}x${image.height}, not ${UNFOLDED_WIDTH}x${BREAKPOINT_HEIGHT}`,
      },);
      await session.ok(`resize ${WIDE_WIDTH} ${BREAKPOINT_HEIGHT}`,);
      return `unfolded at ${image.width}x${image.height}`;
    },
  },);
}
