/**
 Guard against credentials written into Git configuration.

 A secret inside `lfs.url`, a remote URL, a `lfs.<url>.locksverify` key name or an `http.extraHeader` value is copied
 by every copy of `.git`, echoed by `git-lfs` in its error messages and crash logs, and printed by any command that
 lists configuration. Credentials belong in a Git credential helper
 (`doc/troubleshooting/lfs-credential-in-url.md`).

 Nothing here prints or returns a credential: findings carry the configuration key with URL userinfo replaced, never the
 value.

 @module
 */

/**
 Text that replaces the userinfo of a URL wherever this module shows a URL.
 */
export const USERINFO_MASK = '<credentials>';

/**
 Schemes whose URLs never legitimately carry userinfo: any user name before the host is already a credential
 (`https://<token>@host`). Other schemes (`ssh://git@host`) need a password to count.
 */
const WEB_SCHEMES: ReadonlySet<string> = new Set([
  'http',
  'https',
  'ftp',
  'ftps',
  'ws',
  'wss',
],);

/**
 Characters that end the authority part of a URL.
 */
const AUTHORITY_TERMINATORS = '/?#\\ \t\r\n"\'<>`';

/**
 Separator between a URL scheme and its authority.
 */
const SEPARATOR = '://';

/**
 Longest scheme this scanner reads before `://`.
 */
const SCHEME_LIMIT = 32;

/**
 Why a URL counts as carrying a credential.
 */
export type UrlCredentialKind =
  /**
   Userinfo holds a password (`user:password@host`).
   */
  | 'password'
  /**
   Userinfo on a web scheme holds a user name alone, which for hosted services is the token (`token@host`).
   */
    | 'user-on-web-scheme';

/**
 One URL with userinfo found in text.
 */
export type UrlCredentialFinding = {
  /**
   Why the URL counts as carrying a credential.
   */
  readonly kind: UrlCredentialKind;
  /**
   Offset in the scanned text of the first character after `://`, where the userinfo starts.
   */
  readonly userinfoStart: number;
  /**
   Offset of the `@` that ends the userinfo.
   */
  readonly userinfoEnd: number;
};

/**
 Whether a character can be part of a URL scheme.

 @param character - single character to classify

 @returns `true` for a letter, a digit, `+`, `.` or `-`
 */
function isSchemeCharacter(character: string,): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || ((character >= '0') && (character <= '9'))
    || (character === '+')
    || (character === '.')
    || (character === '-');
}

/**
 Scheme written directly before an offset, lowercase.

 @param text - scanned text

 @param end - offset of the `:` that follows the scheme

 @returns the scheme, or an empty string when no scheme precedes `end`
 */
function schemeBefore({
  text,
  end,
}: {
  readonly text: string;
  readonly end: number;
},): string {
  for (let start = end;; start -= 1) {
    if ((start === 0) || ((end - start) >= SCHEME_LIMIT)
      || (!isSchemeCharacter(text.charAt(start - 1,),))) {
      return text.slice(
        start,
        end,
      )
        .toLowerCase();
    }
  }
}

/**
 Offset of the first character at or after `from` that ends a URL authority, or the text length.

 @param text - scanned text

 @param from - offset of the first authority character

 @returns offset one past the last authority character
 */
function authorityEndFrom({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
},): number {
  for (let end = from;; end += 1) {
    if ((end >= text.length) || AUTHORITY_TERMINATORS.includes(text.charAt(end,),)) {
      return end;
    }
  }
}

/**
 What examining one `://` found.
 */
type SeparatorExamination = {
  /**
   The credential-carrying URL at the separator, when there is one.
   */
  readonly finding?: UrlCredentialFinding;
  /**
   Offset to resume searching for the next `://`.
   */
  readonly resume: number;
};

/**
 Examine the URL whose `://` sits at `separator`.

 @param text - scanned text

 @param separator - offset of the `://`

 @returns the finding, if the URL carries a credential, and where to resume searching
 */
function examineSeparator({
  text,
  separator,
}: {
  readonly text: string;
  readonly separator: number;
},): SeparatorExamination {
  /**
   Offset of the first authority character.
   */
  const from = separator + SEPARATOR.length;
  /**
   Scheme before the separator.
   */
  const scheme = schemeBefore({
    text,
    end: separator,
  },);
  if (scheme === '') {
    return { resume: from, };
  }
  /**
   Offset of the first terminator after the separator, or the text length.
   */
  const authorityEnd = authorityEndFrom({
    text,
    from,
  },);
  /**
   Offset of the last `@` in the authority; userinfo ends there.
   */
  const at = text.lastIndexOf(
    '@',
    authorityEnd - 1,
  );
  if ((at <= from) || (at >= authorityEnd)) {
    return { resume: authorityEnd, };
  }
  /**
   Whether the userinfo holds a password.
   */
  const hasPassword = text.slice(
    from,
    at,
  )
    .includes(':',);
  if (hasPassword) {
    return {
      finding: {
        kind: 'password',
        userinfoStart: from,
        userinfoEnd: at,
      },
      resume: authorityEnd,
    };
  }
  if (!WEB_SCHEMES.has(scheme,)) {
    return { resume: authorityEnd, };
  }
  return {
    finding: {
      kind: 'user-on-web-scheme',
      userinfoStart: from,
      userinfoEnd: at,
    },
    resume: authorityEnd,
  };
}

/**
 Every URL in `text` whose authority carries userinfo that counts as a credential.

 One forward pass: each `://` is examined once and its authority is read up to the next terminator, which can never
 reach a later `://`, so the work is linear in the text length.

 @param text - configuration value, key name or file content to inspect

 @returns findings in text order; empty when no URL carries a credential

 @example
 ```ts
 findUrlCredentials({ text: 'https://lfs:s3cret@host.example' });
 // [{ kind: 'password', userinfoStart: 8, userinfoEnd: 17 }]
 ```
 */
export function findUrlCredentials({ text, }: { readonly text: string; },): readonly UrlCredentialFinding[] {
  /**
   Findings gathered so far.
   */
  const findings: UrlCredentialFinding[] = [];
  for (let separator = text.indexOf(SEPARATOR,); separator !== (-1);) {
    /**
     What this separator's URL holds and where to look next.
     */
    const {
      finding,
      resume,
    } = examineSeparator({
      text,
      separator,
    },);
    if (finding !== undefined) {
      findings.push(finding,);
    }
    separator = text.indexOf(
      SEPARATOR,
      resume,
    );
  }
  return findings;
}

/**
 Text with the userinfo of every credential-carrying URL replaced by {@link USERINFO_MASK}.

 @param text - text that may hold URLs with credentials

 @returns the same text with each credential removed, safe to print

 @example
 ```ts
 redactUrlCredentials({ text: 'see https://lfs:s3cret@host.example/x' });
 // 'see https://<credentials>@host.example/x'
 ```
 */
export function redactUrlCredentials({ text, }: { readonly text: string; },): string {
  /**
   Credential-carrying URLs in text order.
   */
  const findings = findUrlCredentials({ text, },);
  /**
   Unmasked stretch before each mask, then the mask.
   */
  const pieces = findings.flatMap(function stretchThenMask(
    finding: UrlCredentialFinding,
    position: number,
  ): readonly string[] {
    return [
      text.slice(
        findings[position - 1]
          ?.userinfoEnd
          ?? 0,
        finding.userinfoStart,
      ),
      USERINFO_MASK,
    ];
  },);
  return pieces.join('',) + text.slice(findings.at(-1,)
    ?.userinfoEnd
    ?? 0,);
}

/**
 One Git configuration entry, as `git config --list --null --show-scope --show-origin` reports it.
 */
export type GitConfigEntry = {
  /**
   Scope Git reports: `system`, `global`, `local`, `worktree` or `command`.
   */
  readonly scope: string;
  /**
   File (or `command line:`) the entry came from.
   */
  readonly origin: string;
  /**
   Full key, including any subsection (`lfs.https://host.locksverify`).
   */
  readonly key: string;
  /**
   Value; never copied into a finding.
   */
  readonly value: string;
};

/**
 Why a configuration entry counts as carrying a credential.
 */
export type GitConfigFindingReason =
  | 'credential-in-value'
    | 'credential-in-key-name'
    | 'authorization-header-in-value';

/**
 One configuration entry that carries a credential, described without the credential.
 */
export type GitConfigFinding = {
  /**
   Scope of the entry.
   */
  readonly scope: string;
  /**
   File the entry came from.
   */
  readonly origin: string;
  /**
   Key with URL userinfo replaced, safe to print.
   */
  readonly redactedKey: string;
  /**
   Which part of the entry holds the credential.
   */
  readonly reason: GitConfigFindingReason;
};

/**
 Fields in one record of the NUL-delimited listing: scope, origin, and key with value.
 */
const RECORD_FIELDS = 3;

/**
 Suffix of the key that sets extra HTTP headers, compared lowercase because Git reads variable names case-insensitively.
 */
const EXTRA_HEADER_SUFFIX = '.extraheader';

/**
 Header names that carry a credential.
 */
const CREDENTIAL_HEADER_NAMES: readonly string[] = [
  'authorization:',
  'proxy-authorization:',
];

/**
 Entries that carry a credential in their key name, their value, or an `Authorization` header value.

 @param entries - configuration entries to inspect

 @returns one finding per offending entry and reason; empty when the configuration holds no credential

 @example
 ```ts
 findGitConfigCredentials({
   entries: [{ scope: 'local', origin: 'file:.git/config', key: 'lfs.url', value: 'https://lfs:s3cret@host.example', },],
 });
 // [{ scope: 'local', origin: 'file:.git/config', redactedKey: 'lfs.url', reason: 'credential-in-value' }]
 ```
 */
export function findGitConfigCredentials({ entries, }: {
  readonly entries: readonly GitConfigEntry[];
},): readonly GitConfigFinding[] {
  /**
   Findings gathered so far.
   */
  const findings: GitConfigFinding[] = [];
  for (const entry of entries) {
    /**
     Key printed in findings, with any userinfo in its subsection removed.
     */
    const redactedKey = redactUrlCredentials({ text: entry.key, },);
    /**
     Fields every finding of this entry shares.
     */
    const common = {
      scope: entry.scope,
      origin: entry.origin,
      redactedKey,
    } as const;
    if (findUrlCredentials({ text: entry.key, },)
      .length
      > 0) {
      findings.push({
        ...common,
        reason: 'credential-in-key-name',
      },);
    }
    if (findUrlCredentials({ text: entry.value, },)
      .length
      > 0) {
      findings.push({
        ...common,
        reason: 'credential-in-value',
      },);
    }
    if (entry.key
      .toLowerCase()
      .endsWith(EXTRA_HEADER_SUFFIX,)) {
      /**
       Value lowercase for the header-name comparison.
       */
      const header = entry.value
        .trimStart()
        .toLowerCase();
      if (CREDENTIAL_HEADER_NAMES.some(function startsWithName(name: string,): boolean {
        return header.startsWith(name,);
      },)) {
        findings.push({
          ...common,
          reason: 'authorization-header-in-value',
        },);
      }
    }
  }
  return findings;
}

/**
 Entries parsed from the NUL-delimited output of `git config --list --null --show-scope --show-origin`.

 With `--null`, each record is `scope NUL origin NUL key LF value NUL` (measured with git 2.56.0), and a key without a
 value has no LF.

 @param output - raw command output

 @returns the entries in output order

 @throws {@link SyntaxError} when the output ends inside a record

 @example
 ```ts
 parseGitConfigList({ output: 'local\0file:.git/config\0lfs.url\nhttps://host.example\0' });
 // [{ scope: 'local', origin: 'file:.git/config', key: 'lfs.url', value: 'https://host.example' }]
 ```
 */
export function parseGitConfigList({ output, }: { readonly output: string; },): readonly GitConfigEntry[] {
  /**
   Entries parsed so far.
   */
  const entries: GitConfigEntry[] = [];
  /**
   NUL-separated fields, three per record, plus the empty field after the final NUL.
   */
  const fields = output.split('\0',);
  if (fields.pop() !== '') {
    throw new SyntaxError('git config output does not end with a NUL',);
  }
  if ((fields.length % RECORD_FIELDS) !== 0) {
    throw new SyntaxError('git config output ends inside a record',);
  }
  for (let at = 0; at < fields.length; at += RECORD_FIELDS) {
    /**
     Key and value field.
     */
    const record = String(fields[at + 2],);
    /**
     Offset of the line feed between key and value.
     */
    const lineFeed = record.indexOf('\n',);
    entries.push({
      scope: String(fields[at],),
      origin: String(fields[at + 1],),
      key: lineFeed === (-1) ? record : record.slice(
        0,
        lineFeed,
      ),
      value: lineFeed === (-1) ? '' : record.slice(lineFeed + 1,),
    },);
  }
  return entries;
}
