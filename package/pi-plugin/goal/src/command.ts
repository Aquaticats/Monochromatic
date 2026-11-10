/**
 Strict `/goal` command parser.
 
 @module
 */

import {
  GOAL_USAGE,
  MAX_OBJECTIVE_LENGTH,
} from './constants.ts';
import type { ParsedGoalCommand, } from './types.ts';

//region Reserved command surface

/**
 Removed command prefixes reserved for direct rejection.
 */
export const REMOVED_COMMAND_PREFIXES = [
  'status',
  'edit',
  'pause',
  'resume',
  '--tokens',
] as const;

//endregion Reserved command surface

//region Parsing

/**
 Index `findIndex` returns when no inspected character matched.
 */
const NO_BOUNDARY_FOUND = -1;

/**
 Return first whitespace-delimited token without regular expression parsing.
 
 @param text - normalized command text
 
 @returns first token
 
 @example
 ```ts
 firstToken('status now');
 ```
 */
function firstToken(text: string,): string {
  /**
   Command text split into UTF-16 code units, so an index into this list is
   also an offset into {@link text} itself.
   */
  const characters = text.split('',);
  /**
   Offset of the first whitespace character, or {@link NO_BOUNDARY_FOUND} when
   the text holds none.
   */
  const boundary = characters.findIndex(function isWhitespace(character: string,): boolean {
    return character.trim() === '';
  },);
  if (boundary === NO_BOUNDARY_FOUND)
    return text;
  return text.slice(
    0,
    boundary,
  );
}

/**
 Parse exact public `/goal` command surface.
 
 @param args - command arguments after `/goal`
 
 @returns start, clear, or direct rejection
 
 @example
 ```ts
 parseGoalCommand('finish migration');
 ```
 */
function parseGoalCommand(args: string,): ParsedGoalCommand {
  /**
   Surrounding-whitespace-normalized arguments.
   */
  const normalized = args.trim();
  if (normalized === '') {
    return {
      kind: 'rejected',
      diagnostic: GOAL_USAGE,
    };
  }
  /**
   First command token used to reserve removed operations.
   */
  const prefix = firstToken(normalized,);
  if (prefix === 'clear') {
    if (normalized === 'clear')
      return { kind: 'clear', };
    return {
      kind: 'rejected',
      diagnostic: GOAL_USAGE,
    };
  }
  if (REMOVED_COMMAND_PREFIXES.some(function prefixWasRemoved(candidate,) {
    return candidate === prefix;
  },)) {
    return {
      kind: 'rejected',
      diagnostic: GOAL_USAGE,
    };
  }
  if (normalized.length > MAX_OBJECTIVE_LENGTH) {
    return {
      kind: 'rejected',
      diagnostic: `Goal objective is too long (${normalized.length}/${MAX_OBJECTIVE_LENGTH} characters). Put long instructions in a file and reference it from /goal instead.`,
    };
  }
  return {
    kind: 'start',
    objective: normalized,
  };
}

//endregion Parsing

export { parseGoalCommand, };
