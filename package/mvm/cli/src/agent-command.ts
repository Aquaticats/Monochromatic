/**
 One request to the QEMU guest agent of a libvirt domain, through
 `virsh qemu-agent-command`.

 Callers get the agent's `return` value, or one of two errors that separate
 "the agent answered with an error" from "no answer arrived". The second kind
 says nothing about what happened in the guest, so callers decide whether to
 ask again.

 @module
 */

import { MS_PER_SECOND, } from '@monochromatic-dev/module-const/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { CommandFailedError, } from './spawn-errors.ts';
import { virsh, } from './virsh.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Errors

/**
 The guest agent received the command and answered it with an error,
 for example an unknown process ID or a file that cannot be opened.
 Asking again gives the same answer.

 @example
 ```ts
 try {
   await agentCommand({ domain: 'mvm-dev', execute: 'guest-exec-status', parameters: { pid: 1 }, timeoutSeconds: 60 });
 }
 catch (error) {
   if (error instanceof GuestAgentReplyError) console.error(error.commandName);
 }
 ```
 */
export class GuestAgentReplyError extends Error {
  /**
   Name of the guest agent command that was answered with an error.
   */
  readonly commandName: string;

  /**
   @param cause - Underlying virsh failure, kept for its exit status and output

   @param commandName - Name of the guest agent command, kept so callers can tell which request failed

   @param domain - Prefixed libvirt domain name, named in the message

   @param stderr - Standard-error text virsh printed, quoted in the message
   */
  constructor({
    cause,
    commandName,
    domain,
    stderr,
  }: {
    readonly cause: unknown;
    readonly commandName: string;
    readonly domain: string;
    readonly stderr: string;
  },) {
    super(
      `The guest agent in ${domain} answered ${commandName} with an error: ${stderr.trim()}`,
      { cause, },
    );
    this.name = 'GuestAgentReplyError';
    this.commandName = commandName;
  }
}

/**
 No answer from the guest agent arrived: virsh failed for another reason,
 such as the agent not answering within the timeout, the agent not being
 connected, the domain not running, or libvirt being unreachable.
 Whether the guest carried out the command is unknown.

 @example
 ```ts
 try {
   await agentCommand({ domain: 'mvm-dev', execute: 'guest-ping', timeoutSeconds: 5 });
 }
 catch (error) {
   if (error instanceof GuestAgentUnreachableError) console.error('no answer');
 }
 ```
 */
export class GuestAgentUnreachableError extends Error {
  /**
   @param cause - Underlying virsh failure, kept for its exit status and output

   @param commandName - Name of the guest agent command, named in the message

   @param domain - Prefixed libvirt domain name, named in the message

   @param stderr - Standard-error text virsh printed, quoted in the message

   @param timeoutSeconds - Seconds libvirt was told to wait, named in the message
   */
  constructor({
    cause,
    commandName,
    domain,
    stderr,
    timeoutSeconds,
  }: {
    readonly cause: unknown;
    readonly commandName: string;
    readonly domain: string;
    readonly stderr: string;
    readonly timeoutSeconds: number;
  },) {
    super(
      `No answer to ${commandName} came back from the guest agent in ${domain} (libvirt waited up to ${
        String(timeoutSeconds,)
      } seconds). virsh reported: ${stderr.trim()}`,
      { cause, },
    );
    this.name = 'GuestAgentUnreachableError';
  }
}

/**
 The guest agent answered, but the answer is larger than libvirt hands over,
 so its content is lost. libvirt allows one answer 4194304 characters; binary
 content travels base64-encoded, which leaves about 3 MiB of it.
 Whatever the command did in the guest has happened.

 @example
 ```ts
 try {
   await agentCommand({ domain: 'mvm-dev', execute: 'guest-exec-status', parameters: { pid: 1 }, timeoutSeconds: 60 });
 }
 catch (error) {
   if (error instanceof GuestAgentAnswerTooLargeError) console.error(error.commandName);
 }
 ```
 */
export class GuestAgentAnswerTooLargeError extends Error {
  /**
   Name of the guest agent command whose answer was too large.
   */
  readonly commandName: string;

  /**
   @param cause - Underlying virsh failure, kept for its exit status and output

   @param commandName - Name of the guest agent command, kept so callers can tell which request failed

   @param domain - Prefixed libvirt domain name, named in the message

   @param stderr - Standard-error text virsh printed, quoted in the message
   */
  constructor({
    cause,
    commandName,
    domain,
    stderr,
  }: {
    readonly cause: unknown;
    readonly commandName: string;
    readonly domain: string;
    readonly stderr: string;
  },) {
    super(
      `The guest agent in ${domain} answered ${commandName}, but the answer is larger than libvirt hands over (4194304 characters), so its content is lost. virsh reported: ${stderr.trim()}`,
      { cause, },
    );
    this.name = 'GuestAgentAnswerTooLargeError';
    this.commandName = commandName;
  }
}

/**
 The guest agent's answer did not have the shape its protocol reference gives for the command.

 @example
 ```ts
 throw new GuestAgentProtocolError({ commandName: 'guest-exec', received: '{"return":{}}' });
 ```
 */
export class GuestAgentProtocolError extends Error {
  /**
   @param cause - Parse failure behind the error, when the answer was not JSON at all

   @param commandName - Name of the guest agent command, named in the message

   @param received - Answer text or value description, quoted in the message
   */
  constructor({
    cause,
    commandName,
    received,
  }: {
    readonly cause?: unknown;
    readonly commandName: string;
    readonly received: string;
  },) {
    super(
      `The guest agent answered ${commandName} with an unexpected value: ${received}`,
      { cause, },
    );
    this.name = 'GuestAgentProtocolError';
  }
}

//endregion Errors

//region JSON for a command-line argument

/**
 Highest character code written as itself; everything greater becomes a `\uXXXX` escape.
 */
const LAST_PLAIN_CODE = 0x7E;

/**
 Highest code point a single `\uXXXX` escape can express; greater ones take a surrogate pair.
 */
const LAST_SINGLE_ESCAPE_CODE = 0xFF_FF;

/**
 First code point outside the range a single escape can express.
 */
const SUPPLEMENTARY_BASE = 0x1_00_00;

/**
 First high (leading) surrogate.
 */
const HIGH_SURROGATE_BASE = 0xD8_00;

/**
 First low (trailing) surrogate.
 */
const LOW_SURROGATE_BASE = 0xDC_00;

/**
 Number of code-point bits each surrogate of a pair carries.
 */
const SURROGATE_BITS = 10;

/**
 Mask selecting the bits a low surrogate carries.
 */
const LOW_SURROGATE_MASK = 0x3_FF;

/**
 Base of the hexadecimal digits in a `\uXXXX` escape.
 */
const HEX_RADIX = 16;

/**
 Number of hexadecimal digits in a `\uXXXX` escape.
 */
const ESCAPE_DIGITS = 4;

/**
 Writes one `\uXXXX` escape.

 @param code - UTF-16 code unit to escape

 @returns Six-character JSON escape

 @example
 ```ts
 unitEscape(0xE9); // => '\\u00e9'
 ```
 */
function unitEscape(code: number,): string {
  return `\\u${
    code
      .toString(HEX_RADIX,)
      .padStart(
        ESCAPE_DIGITS,
        '0',
      )
  }`;
}

/**
 Writes one character of JSON text using ASCII only.

 @param character - One Unicode character of the JSON text

 @returns The character itself when ASCII, else its escape or surrogate-pair escapes

 @example
 ```ts
 asciiCharacter('a'); // => 'a'
 asciiCharacter('é'); // => '\\u00e9'
 ```
 */
function asciiCharacter(character: string,): string {
  /**
   Code point of the character; iteration by character never yields an empty string.
   */
  const code = character.codePointAt(0,) ?? 0;
  if (code <= LAST_PLAIN_CODE) {
    return character;
  }
  if (code <= LAST_SINGLE_ESCAPE_CODE) {
    return unitEscape(code,);
  }
  /**
   Offset into the supplementary range, split across the two surrogates.
   */
  const offset = code - SUPPLEMENTARY_BASE;
  return `${unitEscape(HIGH_SURROGATE_BASE + (offset >> SURROGATE_BITS),)}${
    unitEscape(LOW_SURROGATE_BASE + (offset & LOW_SURROGATE_MASK),)
  }`;
}

/**
 Serializes a value as JSON using ASCII characters only.
 Guest paths and commands may hold any Unicode text; the JSON travels as one
 command-line argument of virsh, and ASCII keeps it independent of the
 locale virsh runs in. Single linear pass over the serialized text.
 Exported from the package entry so the built-artifact tests reach it.

 @param value - Value to serialize

 @returns JSON text whose non-ASCII characters are `\uXXXX` escapes

 @internal

 @example
 ```ts
 asciiJson({ path: '/tmp/é', }); // => '{"path":"/tmp/\\u00e9"}'
 ```
 */
export function asciiJson(value: unknown,): string {
  // Non-ASCII characters can only occur inside the strings of JSON text, where an escape means the same character.
  return Array.from(
    JSON.stringify(value,),
    asciiCharacter,
  )
    .join('',);
}

//endregion JSON for a command-line argument

//region Request

/**
 Text libvirt puts in front of an error the guest agent itself returned.
 Present only when the agent answered; virsh runs in the `C` locale, so the text is not translated.
 */
const AGENT_ANSWERED_WITH_ERROR = 'unable to execute QEMU agent command';

/**
 Text libvirt prints when the agent's answer exceeds what its remote protocol carries in one string.
 Seen on 2026-10-05 with libvirt 12.4.0 for a 3 MiB file read, whose base64 answer is 4194304 characters.
 */
const ANSWER_TOO_LARGE = 'Unable to encode message payload';

/**
 Seconds virsh gets on top of the agent timeout before the call counts as stuck.
 */
const VIRSH_MARGIN_SECONDS = 30;

/**
 Sends one command to the guest agent of a domain and returns its `return` value.

 @param domain - Prefixed libvirt domain name the agent runs in

 @param execute - Guest agent command name, as in the QEMU guest agent protocol reference

 @param parameters - Arguments of the command; omitted for commands that take none

 @param timeoutSeconds - Seconds libvirt waits for the agent's answer before giving up

 @returns Value under the answer's `return` key

 @throws {@link GuestAgentReplyError} when the agent answered with an error

 @throws {@link GuestAgentUnreachableError} when no answer arrived

 @throws {@link GuestAgentAnswerTooLargeError} when the answer is larger than libvirt hands over

 @throws {@link GuestAgentProtocolError} when the answer is not a JSON object with a `return` key

 @example
 ```ts
 await agentCommand({ domain: 'mvm-dev', execute: 'guest-ping', timeoutSeconds: 5 });
 ```
 */
export async function agentCommand({
  domain,
  execute,
  parameters,
  timeoutSeconds,
}: {
  readonly domain: string;
  readonly execute: string;
  readonly parameters?: Readonly<Record<string, unknown>>;
  readonly timeoutSeconds: number;
},): Promise<unknown> {
  /**
   Logger scoped to this request so failures are attributable.
   */
  const rl = tagged({
    tag: agentCommand.name,
    l,
  },);
  /**
   Answer text printed by virsh, or the classified failure.
   */
  const answer = await (async function send(): Promise<string> {
    try {
      return await virsh({
        args: [
          'qemu-agent-command',
          '--timeout',
          String(timeoutSeconds,),
          domain,
          asciiJson({
            execute,
            ...(parameters === undefined ? {} : { arguments: parameters, }),
          },),
        ],
        deadlineMs: (timeoutSeconds + VIRSH_MARGIN_SECONDS) * MS_PER_SECOND,
      },);
    }
    catch (error) {
      // Only a virsh run that failed says something about the agent; a missing executable or an unreachable libvirt is the caller's to see as is.
      if (!(error instanceof CommandFailedError))
        throw error;

      /**
       Standard-error text of the failed virsh call; decides which error the caller sees.
       */
      const { stderr, } = error;
      if (stderr.includes(AGENT_ANSWERED_WITH_ERROR,)) {
        rl.debug(`${execute} in ${domain} answered with an error: ${stderr.trim()}`,);
        throw new GuestAgentReplyError({
          cause: error,
          commandName: execute,
          domain,
          stderr,
        },);
      }
      if (stderr.includes(ANSWER_TOO_LARGE,)) {
        rl.debug(`${execute} in ${domain} answered with more than libvirt hands over: ${stderr.trim()}`,);
        throw new GuestAgentAnswerTooLargeError({
          cause: error,
          commandName: execute,
          domain,
          stderr,
        },);
      }
      rl.debug(`${execute} in ${domain} got no answer: ${stderr.trim()}`,);
      throw new GuestAgentUnreachableError({
        cause: error,
        commandName: execute,
        domain,
        stderr,
        timeoutSeconds,
      },);
    }
  })();
  /**
   Parsed answer; the protocol wraps every result in an object with a `return` key.
   */
  const parsed = (function parse(): unknown {
    try {
      return JSON.parse(answer,);
    }
    catch (error) {
      rl.debug(`${execute} in ${domain} answered with text that is not JSON`,);
      throw new GuestAgentProtocolError({
        cause: error,
        commandName: execute,
        received: answer,
      },);
    }
  })();
  if (((typeof parsed) !== 'object') || (parsed === null)
    || (!('return' in parsed))) {
    throw new GuestAgentProtocolError({
      commandName: execute,
      received: answer,
    },);
  }
  return parsed.return;
}

//endregion Request
