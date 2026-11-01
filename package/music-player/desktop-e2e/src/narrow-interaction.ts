/**
 The click-through check of the narrow session: while Search covers the whole window, a click on its empty area does
 nothing to the hidden player (its pause button, its mode group, its track list), and typing in the query field still
 works afterwards. The same click on the player page changes the mode, which proves the click point was live.
 
 @example
 ```ts
 import { checkNarrowClickThrough, } from './narrow-interaction.ts';
 
 await checkNarrowClickThrough({ context, session, },);
 ```
 
 @packageDocumentation
 */

import {
  chordKeys,
  tapKey,
} from './actions.ts';
import {
  isPlayerPage,
  isSearchWithField,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { NARROW_CLICK_POINTS, } from './layout.ts';
import {
  noEffect,
  press,
} from './press.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';
import {
  clickLeftThingsAlone,
  clicksAt,
  queryIsTone,
  TYPE_QUERY_THEN_DOWN,
} from './wide-interaction.ts';

/**
 Records the click-through check of the narrow session.
 
 @param context - the run's context
 
 @param session - the narrow session, on the player page
 
 @example
 ```ts
 await checkNarrowClickThrough({ context, session, },);
 ```
 */
export async function checkNarrowClickThrough(
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
  recorder.setGroup('click-through',);
  await recorder.check({
    id: 'click-through/search-narrow',
    title: 'clicks on the empty Search page do not reach the hidden pause button, mode group or track list',
    body: async function narrowSearchTakesItsClicks(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        until: isSearchWithField,
        what: 'the Search page with the query field focused',
      },);
      await noEffect({
        session,
        act: clicksAt(NARROW_CLICK_POINTS.underSearch,),
        watch: clickLeftThingsAlone,
        what: 'clicks on the empty Search page',
      },);
      await session.shot('search-after-clicks',);
      await press({
        session,
        act: TYPE_QUERY_THEN_DOWN,
        until: queryIsTone,
        what: 'the typed query reaching the field after the clicks',
      },);
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'the player page',
      },);

      /**
       The state before the control click.
       */
      const before = await session.last();
      await session.click(NARROW_CLICK_POINTS.modeSegment,);
      await session.waitState({
        predicate: function modeChanged(state: UiState,): boolean {
          return state.mode !== before.mode;
        },
        what: 'the mode changing from the same click on the player page (the control for the clicks above)',
        after: before.seq,
      },);
      return 'no click reached the player under Search; the mode segment answers the same click on the player page';
    },
  },);
}
