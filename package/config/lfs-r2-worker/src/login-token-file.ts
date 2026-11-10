/**
 Put one machine's upload token into the private credential file.

 The token arrives from standard input, so it never reaches a command line, a
 process listing, a Git config value or a URL. The file is created at mode `0600`
 in a directory only the account can enter, and Git writes it through
 `git credential approve` with every ambient helper cleared, so no other helper
 receives the pair on the way.

 @module
 */

import {
  chmod,
  mkdir,
  open,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

import { constantTimeEquals, } from './constant-time-equal.ts';
import {
  type CredentialHelper,
  type LfsHost,
  parseLfsHost,
  storeCredentialHelper,
} from './credential-helper.ts';
import { runGit, } from './git-command.ts';

/**
 Permission bits of a file only its owner may read.
 */
export const CREDENTIAL_FILE_MODE = 0o600;

/**
 Permission bits of the directory that holds it.
 */
export const CREDENTIAL_DIRECTORY_MODE = 0o700;

/**
 A token or client id that cannot be stored, and why.
 */
export class LoginTokenError extends Error {
  /**
   @param message - what is wrong with the value and what a token looks like
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'LoginTokenError';
  }
}

/**
 What the credential file now answers for the host.
 */
export type LoginToken = {
  /**
   Client id the file stores as the user name.
   */
  readonly clientId: string;
  /**
   Absolute path of the credential file the token went into.
   */
  readonly credentialFile: string;
  /**
   Config key that names the helper reading that file.
   */
  readonly key: string;
};

/**
 Parameters for {@link storeLoginToken}.
 */
export type StoreLoginTokenParams = {
  /**
   LFS host the credential is for, as an absolute URL.
   */
  readonly host: string;
  /**
   Client id the Worker's `clients` row carries.
   */
  readonly clientId: string;
  /**
   Absolute path of the private credential file.
   */
  readonly credentialFile: string;
  /**
   The machine's token, as read from standard input.
   */
  readonly token: string;
};

/**
 Index just past the last character that is not a line ending.

 @param value - value that may end with line endings

 @returns length the value has once its trailing line endings are dropped
 */
function withoutTrailingLineEndings(value: string,): number {
  /**
   Candidate end of the value, walked back over line endings.
   */
  let end = value.length;
  while ((end > 0) && ((value.charAt(end - 1,) === '\n') || (value.charAt(end - 1,) === '\r'))) {
    end -= 1;
  }
  return end;
}

/**
 Whether any character is whitespace, which a line-oriented credential field cannot
 hold.

 @param value - value to inspect

 @returns `true` when one character is whitespace of any kind
 */
function holdsWhitespace(value: string,): boolean {
  for (const character of value) {
    if (character.trim() === '') {
      return true;
    }
  }
  return false;
}

/**
 Refuse a value that cannot travel through a credential request. A client id or a
 token is one line with no whitespace, because the request is a line-oriented
 key-value block and a second line would be read as another field.

 @param value - client id or token to check

 @param what - name of the value, for the message

 @returns the value with its trailing line endings removed

 @throws {@link LoginTokenError} when the value is empty or holds whitespace
 */
function singleLine({
  value,
  what,
}: {
  readonly value: string;
  readonly what: string
},): string {
  /**
   Value with its trailing line endings removed, which is what a pasted token carries.
   */
  const trimmed = value.slice(
    0,
    withoutTrailingLineEndings(value,),
  );
  if (trimmed === '') {
    throw new LoginTokenError(`the ${what} on standard input is empty`,);
  }
  if (holdsWhitespace(trimmed,)) {
    throw new LoginTokenError(`the ${what} on standard input holds whitespace or a line break, so it is not one credential field`,);
  }
  return trimmed;
}

/**
 Build the key-value block one credential request carries.

 @param fields - fields the request names

 @returns the block, terminated by the blank line that ends a request

 @example
 ```ts
 credentialBlock([['protocol', 'https'], ['host', 'lfs.example']]);
 // 'protocol=https\nhost=lfs.example\n\n'
 ```
 */
function credentialBlock(fields: readonly (readonly [
  string,
  string
])[],): string {
  /**
   Lines of the block, each a `name=value` pair.
   */
  const lines = fields.map(function toLine([name, value,]: readonly [
    string,
    string,
  ],): string {
    return `${name}=${value}`;
  },);
  return `${lines.join('\n',)}\n\n`;
}

/**
 Read the fields a credential helper answered with.

 @param output - standard output of `git credential fill`

 @returns field values by name
 */
function credentialFields(output: string,): Readonly<Record<string, string>> {
  /**
   Lines that carry a field; a blank line ends the answer.
   */
  const lines = output.split('\n',)
    .filter(function hasField(line: string,): boolean {
      return line.includes('=',);
    },);
  return Object.fromEntries(lines.map(function toEntry(line: string,): readonly [
    string,
    string
  ] {
    /**
     Index of the `=` that separates the field name from its value.
     */
    const separator = line.indexOf('=',);
    return [
      line.slice(
        0,
        separator,
      ),
      line.slice(separator + 1,),
    ];
  },),);
}

/**
 Git options that run exactly one helper, the one this module means to write
 through.

 `credential.helper=` clears the accumulated helper list, so a global or
 repository helper cannot receive the pair as well and store it somewhere less
 private.

 @param helper - helper key and command to run

 @returns arguments placed before the `credential` subcommand
 */
function helperArguments({
  helper,
}: { readonly helper: CredentialHelper; },): readonly string[] {
  return [
    '-c',
    'credential.helper=',
    '-c',
    `${helper.key}=${helper.command}`,
  ];
}

/**
 Create the private file if it is absent and force its mode.

 @param credentialFile - absolute path of the credential file
 */
async function prepareCredentialFile(credentialFile: string,): Promise<void> {
  await mkdir(
    dirname(credentialFile,),
    {
      recursive: true,
      mode: CREDENTIAL_DIRECTORY_MODE,
    },
  );
  /**
   Handle of the file the token will be written to; closing it is enough, because
   `git credential approve` does the writing.
   */
  const handle = await open(
    credentialFile,
    'a',
    CREDENTIAL_FILE_MODE,
  );
  await handle.close();
  await chmod(
    credentialFile,
    CREDENTIAL_FILE_MODE,
  );
}

/**
 The two request fields that select a host.

 @param host - host the credential is for

 @returns `protocol` and `host` pairs, in the order a request spells them
 */
function hostFields({ host, }: { readonly host: LfsHost; },): readonly (readonly [
  string,
  string
])[] {
  return [
    [
      'protocol',
      host.protocol,
    ],
    [
      'host',
      host.host,
    ],
  ];
}

/**
 Store one machine's token in the private file, then prove the file answers it.

 The write goes through `git credential approve`, which is the same path a helper
 read follows, so the stored entry has exactly the shape Git and `git-lfs` look
 for. The read-back compares the password in constant time and reports no secret.

 @param host - LFS host the credential is for, as an absolute URL

 @param clientId - client id the Worker's `clients` row carries

 @param credentialFile - absolute path of the private credential file

 @param token - the machine's token, as read from standard input

 @returns the file and the config key that names the helper reading it

 @throws {@link LoginTokenError} when the client id or token is empty or holds whitespace, or
   when the file does not answer the token back

 @throws {@link GitCommandError} when Git rejects the request

 @example
 ```ts
 await storeLoginToken({ host, clientId: 'machine-one', credentialFile, token });
 ```
 */
export async function storeLoginToken({
  host,
  clientId,
  credentialFile,
  token,
}: StoreLoginTokenParams,): Promise<LoginToken> {
  /**
   Host split into the parts a credential request needs.
   */
  const lfsHost = parseLfsHost({ host, },);
  /**
   Client id as one credential field.
   */
  const storedClientId = singleLine({
    value: clientId,
    what: 'client id',
  },);
  /**
   Token as one credential field.
   */
  const storedToken = singleLine({
    value: token,
    what: 'token',
  },);
  /**
   Key and command of the helper that owns the private file.
   */
  const helper = storeCredentialHelper({
    host: lfsHost,
    credentialFile,
  },);
  await prepareCredentialFile(credentialFile,);
  await runGit({
    args: [
      ...helperArguments({ helper, },),
      'credential',
      'approve',
    ],
    cwd: dirname(credentialFile,),
    input: credentialBlock([
      ...hostFields({ host: lfsHost, },),
      [
        'username',
        storedClientId,
      ],
      [
        'password',
        storedToken,
      ],
    ],),
  },);
  /**
   What the file answers when Git asks it for this host, which is the same path
   `git-lfs` takes.
   */
  const answered = credentialFields(await runGit({
    args: [
      ...helperArguments({ helper, },),
      'credential',
      'fill',
    ],
    cwd: dirname(credentialFile,),
    input: credentialBlock(hostFields({ host: lfsHost, },),),
  },),);
  if (!constantTimeEquals({
    left: answered.password ?? '',
    right: storedToken,
  },)) {
    throw new LoginTokenError(`the credential file ${credentialFile} does not answer the token back for ${lfsHost.origin}`,);
  }
  if (answered.username !== storedClientId) {
    throw new LoginTokenError(`the credential file ${credentialFile} answers client id ${answered.username ?? '(none)'} instead of ${storedClientId}`,);
  }
  return {
    clientId: storedClientId,
    credentialFile,
    key: helper.key,
  };
}
