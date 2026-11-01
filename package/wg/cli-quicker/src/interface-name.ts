import { ConfigError, } from './errors.ts';

/**
 Extension stripped from a config basename to derive the interface name.
 */
const CONF_EXTENSION = '.conf';

/**
 Longest interface name Linux accepts (`IFNAMSIZ` is 16 with the terminating NUL).
 */
const MAX_INTERFACE_NAME_LENGTH = 15;

/**
 Characters an interface name may hold.
 None of them is special to a shell or to `nft`,
 and none opens a path.
 */
const NAME_CHARACTERS = new Set('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_=+.-',);

/**
 Names that hold only allowed characters and still name no interface:
 the kernel refuses them, and as a path part they mean the directory itself or its parent.
 */
const DOT_NAMES: ReadonlySet<string> = new Set([
  '.',
  '..',
],);

/**
 Lowest code point of the printable ASCII characters (the space).
 */
const FIRST_PRINTABLE = 0x20;

/**
 Highest code point of the printable ASCII characters (the tilde).
 */
const LAST_PRINTABLE = 0x7E;

/**
 Spells the characters of a text that are not printable ASCII as `\u{...}` escapes,
 so that a refused name in an error message cannot move the cursor, reorder the line or hide behind a look-alike.

 @param text - Text to make showable.

 @returns The text with every character outside printable ASCII written as an escape.

 @example
 ```ts
 showable({ text: 'a\u009bb' }); // 'a\\u{9b}b'
 ```
 */
function showable({ text, }: { readonly text: string; },): string {
  return Array.from(
    text,
    function spell(character,): string {
      /**
       Code point of the character.
       */
      const code = character.codePointAt(0,) ?? 0;
      return (code < FIRST_PRINTABLE) || (code > LAST_PRINTABLE)
        ? `\\u{${code.toString(16,)}}`
        : character;
    },
  )
    .join('',);
}

/**
 Derives the interface name from the argument of the command line:
 the basename of the config path without `.conf`,
 or the bare name itself.

 @param arg - Interface name or config path.

 @returns The interface name the argument stands for,
 not yet checked.

 @example
 ```ts
 interfaceNameOfArgument({ arg: '/etc/wireguard/wg0.conf' }); // 'wg0'
 ```
 */
export function interfaceNameOfArgument({ arg, }: { readonly arg: string; },): string {
  /**
   Last path component of the argument.
   */
  const base = arg.slice(arg.lastIndexOf('/',) + 1,);
  return base.endsWith(CONF_EXTENSION,)
    ? base.slice(
      0,
      -CONF_EXTENSION.length,
    )
    : base;
}

/**
 Tells whether a name can be the name of an interface that wg-quicker puts into commands.
 It is 1 to 15 characters of letters, digits and `_ = + . -`,
 does not begin with `-` (a command would read it as an option),
 and is not `.` or `..`.

 @param interfaceName - Candidate name.

 @returns Whether it is acceptable.

 @example
 ```ts
 isValidInterfaceName({ interfaceName: 'fr-par-fr1' }); // true
 ```
 */
export function isValidInterfaceName({ interfaceName, }: { readonly interfaceName: string; },): boolean {
  if ((interfaceName.length === 0) || (interfaceName.length > MAX_INTERFACE_NAME_LENGTH))
    return false;
  if (interfaceName.startsWith('-',) || DOT_NAMES.has(interfaceName,))
    return false;
  return Array.from(interfaceName,)
    .every(function isAllowed(character,): boolean {
      return NAME_CHARACTERS.has(character,);
    },);
}

/**
 Refuses a name that cannot be the name of an interface,
 before any command, path or nft table is built from it.

 @param interfaceName - Name to check.

 @throws {@link ConfigError} when the name is not acceptable,
 naming it and the rule.

 @example
 ```ts
 assertInterfaceName({ interfaceName: 'wg0' });
 ```
 */
export function assertInterfaceName({ interfaceName, }: { readonly interfaceName: string; },): void {
  if (isValidInterfaceName({ interfaceName, },))
    return;
  throw new ConfigError(
    `${showable({ text: JSON.stringify(interfaceName,), },)} is not a usable interface name: `
      + 'use 1 to 15 characters from letters, digits and _ = + . -, not starting with - and not . or .. '
      + '(the name becomes part of commands, file names and nft table names, and Linux limits it to 15 characters)',
  );
}
