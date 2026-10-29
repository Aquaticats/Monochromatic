import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  isIpLiteral,
  parseAllowedFromFiles,
  parseEndpointPort,
  parsePositiveInt,
  stripComment,
} from './config-parse-values.ts';
import type { ParseAcc, } from './config-parse-types.ts';
import type { WireguardConfig, } from './config-types.ts';
import { ConfigError, } from './errors.ts';
import { assertUnreservedExemptMark, } from './exempt-mark-reservation.ts';
import { trimLinear, } from './text.ts';

/**
 Module logger for config parsing.
 */
const l = tagged({ tag: 'config-parse', },);

/**
 Splits one `key = value` line at its first `=`, trimming both sides linearly.
 
 @param stripped - Comment-stripped config line.
 
 @returns Trimmed key and trimmed value.
 
 @example
 ```ts
 splitKeyValue({ stripped: 'Address = 10.0.0.1/32' });
 ```
 */
function splitKeyValue(
  { stripped, }: { readonly stripped: string; },
): {
  readonly key: string;
  readonly value: string
} {
  /**
   Index of the first key/value separator.
   */
  const equals = stripped.indexOf('=',);
  if (equals === (-1)) {
    return {
      key: stripped,
      value: '',
    };
  }
  return {
    key: trimLinear({ value: stripped.slice(
      0,
      equals,
    ), },),
    value: trimLinear({ value: stripped.slice(equals + 1,), },),
  };
}

/**
 Reports the lowercased section name when a trimmed line is a section header.
 
 @param stripped - Comment-stripped, whitespace-trimmed config line.
 
 @returns Lowercased section name, or an empty string when not a header.
 
 @example
 ```ts
 sectionName({ stripped: '[Interface]' });
 ```
 */
function sectionName({ stripped, }: { readonly stripped: string; },): string {
  if ((!stripped.startsWith('[',)) || (!stripped.endsWith(']',)))
    return '';
  return trimLinear({ value: stripped.slice(
    1,
    -1,
  ), },)
    .toLowerCase();
}

/**
 Routes one `[Interface]` key/value pair into the accumulator.
 
 @param acc - Accumulator being built.
 
 @param key - Trimmed lowercased key.
 
 @param value - Trimmed value.
 
 @param unstripped - Value before comment-stripping, used by hooks.
 
 @returns True when the key was consumed as an interface setting.
 
 @example
 ```ts
 consumeInterfaceKey({ acc, key: 'dns', value: '1.1.1.1', unstripped: '1.1.1.1' });
 ```
 */
function consumeInterfaceKey(
  {
    acc,
    key,
    value,
    unstripped,
  }: {
    readonly acc: ParseAcc;
    readonly key: string;
    readonly value: string;
    readonly unstripped: string;
  },
): boolean {
  if (key === 'address') {
    for (const part of value.split(',',)) {
      /**
       One whitespace-trimmed comma-separated address token.
       */
      const token = trimLinear({ value: part, },);
      if (token !== '')
        acc.addresses
          .push(token,);
    }
    return true;
  }
  if (key === 'dns') {
    for (const part of value.split(',',)) {
      /**
       One whitespace-trimmed comma-separated DNS token.
       */
      const token = trimLinear({ value: part, },);
      if (token === '')
        continue;
      if (isIpLiteral({ token, },))
        acc.dns
          .push(token,);
      else
        acc.dnsSearch
          .push(token,);
    }
    return true;
  }
  if (key === 'table') {
    acc.table = value;
    return true;
  }
  if (key === 'exemptmark') {
    /**
     Positive mark, still unchecked against masks other subsystems match on.
     */
    const mark = parsePositiveInt({
      key,
      value,
    },);
    assertUnreservedExemptMark({
      key: 'ExemptMark',
      mark,
    },);
    acc.exemptMark = mark;
    return true;
  }
  if (key === 'preup') {
    acc.preUp
      .push(unstripped,);
    return true;
  }
  if (key === 'postup') {
    acc.postUp
      .push(unstripped,);
    return true;
  }
  if (key === 'predown') {
    acc.preDown
      .push(unstripped,);
    return true;
  }
  if (key === 'postdown') {
    acc.postDown
      .push(unstripped,);
    return true;
  }
  return false;
}

/**
 Consumes peer-scoped AllowedIPs key when it belongs to wg-quicker.
 
 @param acc - Accumulator carrying current peer state.
 
 @param key - Lowercased key.
 
 @param value - Comment-stripped value.
 
 @returns Whether source-file directive was consumed instead of forwarded.
 
 @throws {@link ConfigError} when directive is outside peer,
 duplicated,
 or conflicts with literal `AllowedIPs`.
 
 @example
 ```ts
 consumePeerAllowedIpsKey({ acc, key: 'allowedips', value: '0.0.0.0/0' });
 ```
 */
function consumePeerAllowedIpsKey(
  {
    acc,
    key,
    value,
  }: {
    readonly acc: ParseAcc;
    readonly key: string;
    readonly value: string;
  },
): boolean {
  if (key === 'allowedips') {
    if (acc.inPeer && acc.peerHasAllowedFromFiles) {
      throw new ConfigError(
        `Peer ${String(acc.peerIndex + 1,)} cannot contain both AllowedIPs and AllowedIPsFromFiles.`,
      );
    }
    if (acc.inPeer)
      acc.peerHasAllowedIps = true;
    return false;
  }
  if (key !== 'allowedipsfromfiles')
    return false;
  if (!acc.inPeer)
    throw new ConfigError('AllowedIPsFromFiles must occur inside a [Peer] section.',);
  if (acc.peerHasAllowedIps) {
    throw new ConfigError(
      `Peer ${String(acc.peerIndex + 1,)} cannot contain both AllowedIPs and AllowedIPsFromFiles.`,
    );
  }
  if (acc.peerHasAllowedFromFiles) {
    throw new ConfigError(
      `Peer ${String(acc.peerIndex + 1,)} cannot contain more than one AllowedIPsFromFiles directive.`,
    );
  }
  /**
   Validated paths associated with current peer and insertion point.
   */
  const paths = parseAllowedFromFiles({ value, },);
  acc.allowedFromFiles
    .push({
      ...paths,
      peerIndex: acc.peerIndex,
      wgLineIndex: acc.wgLines
        .length,
    },);
  acc.peerHasAllowedFromFiles = true;
  return true;
}

/**
 Processes a single raw config line into the accumulator.
 
 @param acc - Accumulator being built.
 
 @param rawLine - One unmodified config line.
 
 @example
 ```ts
 processLine({ acc, rawLine: 'Address = 10.0.0.1/32' });
 ```
 */
function processLine(
  {
    acc,
    rawLine,
  }: {
    readonly acc: ParseAcc;
    readonly rawLine: string;
  },
): void {
  /**
   Comment-stripped, whitespace-trimmed view of the line.
   */
  const stripped = stripComment({ line: rawLine, },);
  /**
   Lowercased section name when the line opens a section.
   */
  const section = sectionName({ stripped, },);
  if (section !== '') {
    acc.inInterface = section === 'interface';
    acc.inPeer = section === 'peer';
    if (acc.inPeer) {
      acc.peerIndex += 1;
      acc.peerHasAllowedIps = false;
      acc.peerHasAllowedFromFiles = false;
    }
    // The [Interface] header must reach wg addconf so PrivateKey stays valid.
    acc.wgLines
      .push(rawLine,);
    return;
  }
  if (stripped === '') {
    acc.wgLines
      .push(rawLine,);
    return;
  }
  /**
   Trimmed key and value split from the comment-stripped line.
   */
  const {
    key,
    value,
  } = splitKeyValue({ stripped, },);
  /**
   Lowercased key for case-insensitive matching.
   */
  const lower = key.toLowerCase();
  if (consumePeerAllowedIpsKey({
    acc,
    key: lower,
    value,
  },))
    return;
  if (acc.inPeer && (lower === 'endpoint'))
    acc.endpointPorts
      .push(parseEndpointPort({ value, },),);
  if (acc.inInterface) {
    /**
     Index of the first key/value separator in the raw line.
     */
    const equals = rawLine.indexOf('=',);
    /**
     Value text before comment-stripping, preserving hook commands.
     */
    const unstripped = trimLinear({ value: rawLine.slice(equals + 1,), },);
    if (lower === 'mtu') {
      // MTU is consumed for `ip link set` and not forwarded to wg, which rejects it.
      acc.mtu = parsePositiveInt({
        key,
        value,
      },);
      return;
    }
    if (consumeInterfaceKey({
      acc,
      key: lower,
      value,
      unstripped,
    },))
      return;
  }
  acc.wgLines
    .push(rawLine,);
}


/**
 Parses config text into the interface settings wg-quicker consumes and the raw
 peer block passed to `wg addconf`.
 
 Only `[Interface]` routing/DNS/hook keys are interpreted; `PrivateKey`, the
 `[Interface]` header, and every other line are forwarded verbatim, so a very
 large `AllowedIPs` value is stored but never pattern-matched.
 
 @param interfaceName - Interface name used for logging context.
 
 @param text - Full config file text.
 
 @returns Parsed interface settings plus raw peer config.
 
 @example
 ```ts
 parseConfigText({ interfaceName: 'wg0', text: '[Interface]\nPrivateKey = k\nAddress = 10.0.0.1/32\n' });
 ```
 */
export function parseConfigText(
  {
    interfaceName,
    text,
  }: {
    readonly interfaceName: string;
    readonly text: string;
  },
): WireguardConfig {
  /**
   Function-scoped logger for the parse.
   */
  const fl = tagged({
    tag: parseConfigText.name,
    l,
  },);
  /**
   Accumulator built by the line walk, contained so no root-level `let` leaks.
   */
  const acc: ParseAcc = {
    addresses: [],
    dns: [],
    dnsSearch: [],
    preUp: [],
    postUp: [],
    preDown: [],
    postDown: [],
    allowedFromFiles: [],
    endpointPorts: [],
    wgLines: [],
    inInterface: false,
    inPeer: false,
    peerIndex: -1,
    peerHasAllowedIps: false,
    peerHasAllowedFromFiles: false,
  };
  for (const rawLine of text.split('\n',))
    processLine({
      acc,
      rawLine,
    },);
  fl.debug(
    `parsed ${String(acc.addresses
      .length,)} address(es), ${String(acc.dns
        .length,)} dns, `
      + `${String(acc.wgLines
        .length,)} forwarded line(s) for ${interfaceName}`,
  );
  return {
    interfaceName,
    addresses: acc.addresses,
    dns: acc.dns,
    dnsSearch: acc.dnsSearch,
    ...(acc.mtu === undefined ? {} : { mtu: acc.mtu, }),
    ...(acc.table === undefined ? {} : { table: acc.table, }),
    ...(acc.exemptMark === undefined ? {} : { exemptMark: acc.exemptMark, }),
    endpointPorts: [...new Set(acc.endpointPorts,),],
    allowedFromFiles: acc.allowedFromFiles,
    preUp: acc.preUp,
    postUp: acc.postUp,
    preDown: acc.preDown,
    postDown: acc.postDown,
    wgConfig: `${acc.wgLines
      .join('\n',)}\n`,
  };
}
