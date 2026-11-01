/**
 Which checks and which sessions a run includes, given the optional `E2E_ONLY` filter.
 
 The suite runs in several app sessions (the wide window, the narrow window, the folder dialog, the Auto-play on select
 setting over a restart, the Text size setting over a restart, quitting, and one per
 color scheme and accent for the screenshots). A filter names a check id, or a group prefix; this module decides which
 sessions a filter needs and whether one check passes the filter.
 
 @example
 ```ts
 import { sessionsFor, } from './selection.ts';
 
 sessionsFor({ only: 'folded', },); // ['narrow']
 ```
 
 @packageDocumentation
 */

/**
 The sessions of a run, in the order they run.
 */
export type SessionName = 'auto-play' | 'dialog' | 'narrow' | 'quit' | 'states' | 'text-size' | 'visual' | 'wide';

/**
 The group prefixes and exact ids that bring a session into a filtered run.
 */
export type SessionRule = {
  readonly exact: readonly string[];
  readonly name: SessionName;
  readonly prefixes: readonly string[];
};

/**
 The rule of every session. An empty filter includes all of them.
 */
export const SESSION_RULES: readonly SessionRule[] = [
  {
    name: 'wide',
    prefixes: [
      'start',
      'focus',
      'playback',
      'browse',
      'folder-list',
      'track-list',
      'row',
      'search',
      'double-shift',
      'settings',
      'badges',
      'play',
      'mode',
      'seek',
      'audio',
      'track',
      'tooltip',
      'menu',
      'click-through',
      'lists',
      'layout',
    ],
    exact: [],
  },
  {
    name: 'narrow',
    prefixes: [
      'folded',
      'editor-page',
      'click-through',
    ],
    exact: [],
  },
  // The dialog session also runs for its one check by exact id, while the prefix `browse` selects the wide session only.
  {
    name: 'dialog',
    prefixes: ['dialog',],
    exact: ['browse/ctrl-shift-o',],
  },
  {
    name: 'auto-play',
    prefixes: ['auto-play',],
    exact: [],
  },
  {
    name: 'text-size',
    prefixes: ['text-size',],
    exact: [],
  },
  {
    name: 'quit',
    prefixes: ['quit',],
    exact: [],
  },
  {
    name: 'states',
    prefixes: ['states',],
    exact: [],
  },
  {
    name: 'visual',
    prefixes: ['visual',],
    exact: [],
  },
];

/**
 Lists the sessions a filter needs.
 
 @param only - the filter; empty for none
 
 @returns the names of the sessions to run, in run order
 
 @example
 ```ts
 sessionsFor({ only: '', },); // all six
 ```
 */
export function sessionsFor(
  {
    only,
  }: {
    readonly only: string;
  },
): readonly SessionName[] {
  return SESSION_RULES
    .filter(function isNeeded(rule,): boolean {
      return (only === '')
        || rule.exact
          .includes(only,)
        || rule.prefixes
          .some(function startsWithPrefix(prefix,): boolean {
            return only.startsWith(prefix,);
          },);
    },)
    .map(function nameOf(rule,): SessionName {
      return rule.name;
    },);
}

/**
 Decides whether one check passes the filter: its id or its group contains the filter text.
 
 @param only - the filter; empty for none
 
 @param id - the check id
 
 @param group - the group the check belongs to
 
 @returns whether the check runs
 
 @example
 ```ts
 isSelected({ only: 'seek', id: 'seek/alt-left', group: 'playback', },); // true
 ```
 */
export function isSelected(
  {
    only,
    id,
    group,
  }: {
    readonly group: string;
    readonly id: string;
    readonly only: string;
  },
): boolean {
  return (only === '') || id.includes(only,)
    || group.includes(only,);
}
