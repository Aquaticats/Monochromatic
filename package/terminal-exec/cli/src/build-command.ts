/**
 Constructs the final command array from a resolved terminal entry and user-provided options.
 Handles the trailing-`=` concatenation convention for terminal argument keys.
 
 @module
 */
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { ResolvedTerminal, } from './resolve.ts';

/**
 Logger root for terminal-exec after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);

/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'build-command',
  l: parentLogger,
},);

/**
 Options passed by the user to `xdg-terminal-exec`.
 */
export type UserOptions = {
  /**
   Value for `--app-id=VALUE`.
   */
  readonly appId: string;
  /**
   Value for `--title=VALUE`.
   */
  readonly title: string;
  /**
   Value for `--dir=VALUE`.
   */
  readonly dir: string;
  /**
   Whether `--hold` was passed.
   */
  readonly hold: boolean;
  /**
   Command and arguments to execute in the terminal.
   */
  readonly command: readonly string[];
};

/**
 Builds the token(s) for one terminal argument using the trailing-`=` convention.
 If the arg key ends with `=`, the value is concatenated as one token (`--title=My Title`).
 Otherwise, key and value become two separate tokens (`--title` `My Title`).
 
 @param argKey - Terminal argument key (e.g. `--title=` or `--title`).
 
 @param value - User-provided value.
 
 @returns Tokens to append for this argument.
 
 @example
 ```ts
 appendArg({ argKey: '--title=', value: 'My Shell' }); // ['--title=My Shell']
 appendArg({ argKey: '--title', value: 'My Shell' });  // ['--title', 'My Shell']
 ```
 */
function appendArg(
  {
    argKey,
    value,
  }: {
    readonly argKey: string;
    readonly value: string;
  },
): readonly string[] {
  if (argKey.endsWith('=',))
    return [`${argKey}${value}`,];
  return [
    argKey,
    value,
  ];
}

/**
 Builds the final command array from a resolved terminal and user options.
 
 Preserves configured Exec tokens, including explicit single-instance policy.
 Each applicable option block hands its key and value to {@link appendArg}
 to apply the trailing-`=` convention.
 
 @param terminal - Resolved terminal entry with Exec tokens and argument keys.
 
 @param options - User-provided options from CLI parsing.
 
 @returns Complete command array ready for `exec`.
 
 @example
 ```ts
 const cmd = buildCommand({
   terminal: { execTokens: ['/usr/bin/ghostty'], execArg: '-e', ... },
   options: { command: ['bash', '-l'], appId: '', title: '', dir: '', hold: false },
 })
 // ['/usr/bin/ghostty', '-e', 'bash', '-l']
 ```
 */
export function buildCommand({
  terminal,
  options,
}: {
  readonly terminal: ResolvedTerminal;
  readonly options: UserOptions;
},): readonly string[] {
  /**
   Immutable command: configured exec tokens followed by the tokens each applicable option block contributes.
   */
  const args: readonly string[] = [
    ...terminal.execTokens,
    ...(((options.appId
      .length
      > 0) && (terminal.appIdArg
        .length
        > 0))
      ? appendArg({
        argKey: terminal.appIdArg,
        value: options.appId,
      },)
      : []),
    ...(((options.title
      .length
      > 0) && (terminal.titleArg
        .length
        > 0))
      ? appendArg({
        argKey: terminal.titleArg,
        value: options.title,
      },)
      : []),
    ...(((options.dir
      .length
      > 0) && (terminal.dirArg
        .length
        > 0))
      ? appendArg({
        argKey: terminal.dirArg,
        value: options.dir,
      },)
      : []),
    ...((options.hold
      && (terminal.holdArg
        .length
        > 0))
      ? [terminal.holdArg,]
      : []),
    ...((options.command
      .length
      > 0)
      ? [
        terminal.execArg,
        ...options.command,
      ]
      : []),
  ];

  /**
   Info-level because the resolved command is user-facing diagnostic output.
   */
  l.info(`final command: ${JSON.stringify(args,)}`,);
  return args;
}
