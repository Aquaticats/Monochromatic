/**
 Reading facts out of the app's log lines: shortcut uses, the nested Wayland display name and how far a start got.
 
 @example
 ```ts
 import { countShortcutUses, } from './log-scan.ts';
 
 countShortcutUses({ lines: ['INFO shortcut used action="reveal"',], action: 'reveal', },); // 1
 ```
 
 @packageDocumentation
 */

import { isDigit, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

/**
 Text the app logs each time a shortcut ran.
 */
const SHORTCUT_USED = 'shortcut used';

/**
 Text before the nested compositor's display name in its start-up log.
 */
const SOCKET_MARKER = 'Created new socket';

/**
 Text between the marker and the display name.
 */
const NAME_OPENING = 'name=Some("';

/**
 Text after the display name.
 */
const NAME_CLOSING = '")';

/**
 Display names start with this word.
 */
const DISPLAY_PREFIX = 'wayland-';

/**
 Counts the lines in which the app reported using a shortcut action.
 
 @param lines - log lines without color sequences
 
 @param action - the action name to count; every action when absent
 
 @returns how many lines match
 
 @example
 ```ts
 countShortcutUses({ lines: [], },); // 0
 ```
 */
export function countShortcutUses(
  {
    lines,
    action,
  }: {
    readonly action?: string;
    readonly lines: readonly string[];
  },
): number {
  return lines.filter(function isUse(line,): boolean {
    return line.includes(SHORTCUT_USED,) && ((action === undefined) || line.includes(`action="${action}"`,));
  },)
    .length;
}

/**
 Tells whether a text is a Wayland display name: the prefix and at least one digit, nothing else.
 
 @param name - candidate name
 
 @returns whether the name has the form `wayland-N`
 
 @example
 ```ts
 isDisplayName('wayland-1',); // true
 ```
 */
export function isDisplayName(name: string,): boolean {
  if (!name.startsWith(DISPLAY_PREFIX,))
    return false;

  /**
   The characters after the prefix.
   */
  const digits = name.slice(DISPLAY_PREFIX.length,);
  return (digits !== '')
    && Array.from(digits,)
      .every(function checkDigit(character,): boolean {
        return isDigit(character,);
      },);
}

/**
 Finds the nested Wayland display name in the compositor's start-up lines: the line `Created new socket
 name=Some("wayland-N")`.
 
 @param lines - log lines without color sequences
 
 @returns the first display name found, or the empty string
 
 @example
 ```ts
 waylandDisplayOf(['Created new socket name=Some("wayland-1")',],); // 'wayland-1'
 ```
 */
export function waylandDisplayOf(lines: readonly string[],): string {
  for (const line of lines) {
    /**
     Where the marker starts, or -1.
     */
    const at = line.indexOf(SOCKET_MARKER,);
    if (at === (-1))
      continue;

    /**
     Text after the marker; at least one space or tab separates it from the name.
     */
    const afterMarker = line.slice(at + SOCKET_MARKER.length,);

    /**
     The same text without the leading white space.
     */
    const afterSpace = afterMarker.trimStart();
    if ((afterSpace.length === afterMarker.length) || (!afterSpace.startsWith(NAME_OPENING,)))
      continue;

    /**
     Text after the opening of the name.
     */
    const rest = afterSpace.slice(NAME_OPENING.length,);

    /**
     The name, up to its closing.
     */
    const name = rest.slice(
      0,
      Math.max(
        rest.indexOf(NAME_CLOSING,),
        0,
      ),
    );
    if (rest.includes(NAME_CLOSING,) && isDisplayName(name,))
      return name;
  }
  return '';
}

/**
 How far a session start got, from the unit's log, each stage with what it means in plain words, in the order a start
 passes them.

 Measured on 2026-11-07: a start that stalls stops at one of these, and each is a different place to look. Silence means
 the unit has not run the compositor yet (or its first page-in is still going); a compositor with no socket is opening
 its render device; a socket with no spawned client is past device setup; a spawned app with no line of its own is still
 loading; an app that logs but writes no state line has not drawn its first page.
 */
const START_STAGE_TEXT = {
  'compositor-silent': 'the compositor wrote nothing yet (the unit has not run it, or it is still loading its files)',
  'compositor-starting': 'the compositor is running but has not opened its control socket (it is opening the render device)',
  'socket-listening': 'the control socket is open but the compositor has not started the app',
  'app-spawned': 'the compositor started the app, which wrote no line of its own yet (the app is still loading)',
  'app-running': 'the app is logging but has written no ui_state line (it has not drawn its first page)',
  'state-line': 'the app wrote ui_state lines (the start finished)',
} as const;

/**
 A point a start passes.
 */
export type StartStage = keyof typeof START_STAGE_TEXT;

/**
 Text of the compositor's line that says its control socket is open.
 */
const SOCKET_LISTENING = 'control socket listening';

/**
 Text of the compositor's line that says it started the hosted app.
 */
const CLIENT_SPAWNED = 'spawned hosted client';

/**
 Text every line of the app's own logger carries: its crate name as the logger target.
 */
const APP_LOGGER = 'music_player::';

/**
 Text of the app's state lines.
 */
const STATE_LINE = 'ui_state {';

/**
 Finds the furthest point a start reached.

 @param lines - log lines without color sequences

 @returns the stage of the last point the log shows

 @example
 ```ts
 startStageOf({ lines: [], },); // 'compositor-silent'
 ```
 */
export function startStageOf({ lines, }: { readonly lines: readonly string[]; },): StartStage {
  /**
   Whether any line contains the text.

   @param text - what to look for

   @returns whether a line has it
   */
  function shows(text: string,): boolean {
    return lines.some(function hasText(line,): boolean {
      return line.includes(text,);
    },);
  }
  if (shows(STATE_LINE,))
    return 'state-line';
  if (shows(APP_LOGGER,))
    return 'app-running';
  if (shows(CLIENT_SPAWNED,))
    return 'app-spawned';
  if (shows(SOCKET_LISTENING,))
    return 'socket-listening';
  if (lines.some(function isNotBlank(line,): boolean {
    return line.trim() !== '';
  },))
    return 'compositor-starting';
  return 'compositor-silent';
}

/**
 Says in plain words where a start stopped.

 @param stage - the stage `startStageOf` found

 @returns a sentence fragment naming the last point reached and what to look at next

 @example
 ```ts
 describeStartStage('app-spawned',); // 'the compositor started the app, which wrote no line of its own yet (still loading)'
 ```
 */
export function describeStartStage(stage: StartStage,): string {
  return START_STAGE_TEXT[stage];
}
