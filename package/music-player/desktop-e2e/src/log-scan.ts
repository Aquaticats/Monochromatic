/**
 Reading facts out of the app's log lines: shortcut uses and the nested Wayland display name.
 
 @example
 ```ts
 import { countShortcutUses, } from './log-scan.ts';
 
 countShortcutUses({ lines: ['INFO shortcut used action="reveal"',], action: 'reveal', },); // 1
 ```
 
 @packageDocumentation
 */

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
      .every(function isDigit(character,): boolean {
        return (character >= '0') && (character <= '9');
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
