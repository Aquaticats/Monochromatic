import {
  isIPv4,
  isIPv6,
} from 'node:net';

import { normalizeAddress, } from './host-address.ts';
import { run, } from './runner.ts';
import type { BypassProto, } from './tunnel-bypass-types.ts';

/**
 What the tunnel itself needs to stay reachable:
 the DNS servers it sends queries to,
 and the endpoints of its peers.

 A host rule for one of these addresses would send the tunnel's own control traffic around the tunnel,
 or move its DNS onto the physical network.
 */
export type TunnelDependencies = {
  /**
   DNS servers set on the tunnel link,
   as printed by `resolvectl`.
   */
  readonly dnsServers: readonly string[];

  /**
   Peer endpoint addresses,
   as printed by `wg show`.
   */
  readonly endpoints: readonly string[];
};

/**
 Reads the tunnel's dependencies.
 */
export type TunnelDependencyReader = (call: { readonly interfaceName: string; },) => Promise<TunnelDependencies>;

/**
 Reduces one token of `resolvectl dns` output to the bare address:
 drops the `#server-name` of DNS-over-TLS,
 the `%scope` of a link-local address,
 and a port.

 @param token - One server as printed.

 @returns The address text.

 @example
 ```ts
 bareAddress({ token: '1.1.1.1#cloudflare-dns.com' }); // '1.1.1.1'
 ```
 */
function bareAddress({ token, }: { readonly token: string; },): string {
  /**
   Token without the server name.
   */
  const withoutName = token.split('#',)[0] ?? '';
  if (withoutName.startsWith('[',)) {
    /**
     Position of the bracket that closes the address.
     */
    const close = withoutName.indexOf(']',);
    return close === (-1) ? '' : (withoutName.slice(
      1,
      close,
    )
      .split('%',)[0]
      ?? '');
  }
  /**
   Text around the only colon of a dotted IPv4 address with a port.
   */
  const [host, port, ...extra] = withoutName.split(':',);
  if ((port !== undefined) && (extra.length === 0)
    && isIPv4(host ?? '',))
    return host ?? '';
  return withoutName.split('%',)[0] ?? '';
}

/**
 Reports whether a line starts with white space,
 as the lines `resolvectl` wraps onto do.

 @param line - One output line.

 @returns Whether the line is indented.

 @example
 ```ts
 isIndented({ line: '        2620:fe::fe' }); // true
 ```
 */
function isIndented({ line, }: { readonly line: string; },): boolean {
  return line.startsWith(' ',) || line.startsWith('\t',);
}

/**
 Reports whether a line is the label line of a link,
 such as `Link 11 (wg0): 1.1.1.1`.

 @param line - One output line.

 @returns Whether the line carries a label.

 @example
 ```ts
 isLabelLine({ line: 'Link 11 (wg0): 1.1.1.1' }); // true
 ```
 */
function isLabelLine({ line, }: { readonly line: string; },): boolean {
  return (!isIndented({ line, },)) && (line.includes(': ',)
    || line.trimEnd()
      .endsWith(':',));
}

/**
 Picks the text of one line that holds servers.

 `resolvectl` wraps a long server list onto indented lines,
 which belong to the label line before them.

 @param line - The line.

 @param lines - All lines of the output.

 @param index - Position of the line.

 @returns The server text,
 empty when the line holds none.

 @example
 ```ts
 serversText({ line: 'Link 11 (wg0): 1.1.1.1', lines: ['Link 11 (wg0): 1.1.1.1'], index: 0 }); // '1.1.1.1'
 ```
 */
function serversText(
  {
    line,
    lines,
    index,
  }: {
    readonly line: string;
    readonly lines: readonly string[];
    readonly index: number;
  },
): string {
  if (isLabelLine({ line, },)) {
    /**
     Position of the colon that ends the label.
     */
    const colon = line.indexOf(': ',);
    return colon === (-1) ? '' : line.slice(colon + 2,);
  }
  if (!isIndented({ line, },))
    return '';
  for (let before = index - 1; before >= 0; before -= 1) {
    /**
     Earlier line that decides what this indented line belongs to.
     */
    const candidate = lines[before] ?? '';
    if (!isIndented({ line: candidate, },))
      return isLabelLine({ line: candidate, },) ? line : '';
  }
  return '';
}

/**
 Reads the DNS servers out of `resolvectl dns <interface>` output.

 @param text - Command output,
 such as `Link 11 (wg0): 1.1.1.1 9.9.9.9`,
 possibly wrapped onto indented lines.

 @returns Server addresses,
 none when the link has no DNS server.

 @example
 ```ts
 parseResolvectlDns({ text: 'Link 11 (wg0): 1.1.1.1 9.9.9.9' }); // ['1.1.1.1', '9.9.9.9']
 ```
 */
export function parseResolvectlDns({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Lines of the output.
   */
  const lines = text.split('\n',);
  return lines.flatMap(function serversOfLine(
    line,
    index,
  ): readonly string[] {
    /**
     Text that holds servers:
     the part after the label,
     or a wrapped continuation of a label line.
     */
    const servers = serversText({
      line,
      lines,
      index,
    },);
    return servers.split(' ',)
      .map(function bare(token,): string {
        return bareAddress({ token, },);
      },)
      .filter(function isAddress(address,): boolean {
        return isIPv4(address,) || isIPv6(address,);
      },);
  },);
}

/**
 Reads the peer endpoint addresses out of `wg show <interface> endpoints` output.

 @param text - Command output,
 one `<public key><TAB><address>:<port>` line per peer,
 or `(none)` for a peer with no endpoint.

 @returns Endpoint addresses.

 @example
 ```ts
 parseWgEndpoints({ text: 'abc=\t203.0.113.1:51820' }); // ['203.0.113.1']
 ```
 */
export function parseWgEndpoints({ text, }: { readonly text: string; },): readonly string[] {
  return text.split('\n',)
    .flatMap(function endpointOfLine(line,): readonly string[] {
      /**
       Endpoint column.
       */
      const column = line.split('\t',)[1]
        ?.trim()
        ?? '';
      if ((column === '') || (column === '(none)'))
        return [];
      /**
       Address without the port.
       */
      const address = bareAddress({ token: column, },);
      return isIPv4(address,) || isIPv6(address,) ? [address,] : [];
    },);
}

/**
 Reads the tunnel's dependencies from the running system.

 @param interfaceName - Tunnel interface name.

 @returns DNS servers and peer endpoints.

 @throws When `resolvectl` or `wg` cannot be run,
 so the caller can withhold every host rule instead of guessing.

 @example
 ```ts
 await readTunnelDependencies({ interfaceName: 'wg0' });
 ```
 */
export async function readTunnelDependencies(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<TunnelDependencies> {
  /**
   Output of the two commands.
   */
  const [dns, endpoints,] = await Promise.all([
    run({
      command: 'resolvectl',
      args: [
        'dns',
        interfaceName,
      ],
    },),
    run({
      command: 'wg',
      args: [
        'show',
        interfaceName,
        'endpoints',
      ],
    },),
  ],);
  return {
    dnsServers: parseResolvectlDns({ text: dns.stdout, },),
    endpoints: parseWgEndpoints({ text: endpoints.stdout, },),
  };
}

/**
 Identity keys of the addresses the tunnel depends on,
 in the form `<family>|<normalized address>` that host rules use.

 @param dependencies - DNS servers and endpoints.

 @returns The keys.

 @example
 ```ts
 protectedKeys({ dependencies: { dnsServers: ['1.1.1.1'], endpoints: [] } });
 ```
 */
export function protectedKeys({ dependencies, }: { readonly dependencies: TunnelDependencies; },): ReadonlySet<string> {
  return new Set([
    ...dependencies.dnsServers,
    ...dependencies.endpoints,
  ].flatMap(function keyOf(text,): readonly string[] {
    /**
     Family of the address.
     */
    const family: BypassProto = isIPv4(text,) ? '-4' : '-6';
    /**
     Normalized address text.
     */
    const address = normalizeAddress({
      family,
      text,
    },);
    return address === '' ? [] : [`${family}|${address}`,];
  },),);
}
