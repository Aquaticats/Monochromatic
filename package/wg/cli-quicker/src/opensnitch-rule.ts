import { randomUUID, } from 'node:crypto';

import type { JsonRecord, } from './opensnitch-config-tree.ts';

/**
 Stable prefix identifying rules owned by wg-quicker.
 */
const MANAGED_DESCRIPTION_PREFIX = 'wg-quicker managed endpoint';

/**
 Highest valid UDP port number.
 */
const MAX_UDP_PORT = 65_535;

/**
 Reports non-null JSON object.
 
 @param value - Unknown JSON value.
 
 @returns Whether value is object record.
 
 @example
 ```ts
 isRecord({ Enabled: true });
 ```
 */
function isRecord(value: unknown,): value is JsonRecord {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Creates interface-specific managed-rule description prefix.
 
 @param interfaceName - WireGuard interface owning rules.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @returns Prefix with unambiguous interface and namespace delimiters.
 
 @example
 ```ts
 managedPrefix({ interfaceName: 'wg0', networkNamespaceKey: 'abc123' });
 ```
 */
export function managedPrefix(
  {
    interfaceName,
    networkNamespaceKey,
  }: {
    readonly interfaceName: string;
    readonly networkNamespaceKey: string;
  },
): string {
  if (networkNamespaceKey === '')
    return `${MANAGED_DESCRIPTION_PREFIX} [${interfaceName}] UDP destination port `;
  return `${MANAGED_DESCRIPTION_PREFIX} [${interfaceName}] [netns:${networkNamespaceKey}] UDP destination port `;
}

/**
 Reports whether rule belongs to one interface's wg-quicker lifecycle.
 
 @param value - Unknown rule value.
 
 @param prefix - Exact interface-specific ownership prefix.
 
 @returns Whether description identifies managed rule.
 
 @example
 ```ts
 isManagedRule({ value: { Description: 'wg-quicker managed endpoint [wg0] UDP destination port 1' }, prefix });
 ```
 */
export function isManagedRule(
  {
    value,
    prefix,
  }: {
    readonly value: unknown;
    readonly prefix: string;
  },
): boolean {
  if (!isRecord(value,))
    return false;
  if ((typeof value.Description) !== 'string')
    return false;
  return value
    .Description
    .startsWith(prefix,);
}

/**
 Creates one OpenSnitch nftables accept rule for endpoint UDP port.
 
 @param interfaceName - WireGuard interface owning rule.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @param port - UDP destination port accepted before NFQUEUE.
 
 @returns OpenSnitch version 1 rule object.
 
 @example
 ```ts
 createManagedRule({ interfaceName: 'wg0', networkNamespaceKey: 'abc123', port: 51820 });
 ```
 */
export function createManagedRule(
  {
    interfaceName,
    networkNamespaceKey,
    port,
  }: {
    readonly interfaceName: string;
    readonly networkNamespaceKey: string;
    readonly port: number;
  },
): JsonRecord {
  return {
    UUID: randomUUID(),
    Enabled: true,
    Position: '0',
    Description: `${managedPrefix({
      interfaceName,
      networkNamespaceKey,
    },)}${String(port,)}`,
    Parameters: '',
    Expressions: [
      {
        Statement: {
          Op: '',
          Name: 'udp',
          Values: [
            {
              Key: 'dport',
              Value: String(port,),
            },
          ],
        },
      },
    ],
    Target: 'accept',
    TargetParameters: '',
  };
}

/**
 Extracts exact destination port from one UDP statement value.
 
 @param value - Unknown expression value.
 
 @returns Exact valid port as singleton array,
 or empty array for unsupported value.
 
 @example
 ```ts
 exactUdpPort({ Key: 'dport', Value: '51820' });
 ```
 */
function exactUdpPort(value: unknown,): readonly number[] {
  if (!isRecord(value,))
    return [];
  if ((value.Key !== 'dport') || ((typeof value.Value) !== 'string'))
    return [];
  /**
   Numeric exact-port candidate.
   */
  const port = Number(value.Value,);
  if (!Number.isSafeInteger(port,))
    return [];
  if (port <= 0)
    return [];
  if (port > MAX_UDP_PORT)
    return [];
  return [port,];
}

/**
 Extracts exact UDP destination ports accepted by one enabled rule.
 
 Complex port sets and ranges are intentionally not inferred;
 only exact values can prove whether one generated rule disappeared.
 
 @param value - Unknown OpenSnitch rule.
 
 @returns Exact accepted UDP ports.
 
 @example
 ```ts
 acceptedUdpPorts({ value: managedRule });
 ```
 */
export function acceptedUdpPorts(
  { value, }: { readonly value: unknown; },
): readonly number[] {
  if (!isRecord(value,))
    return [];
  if ((value.Enabled !== true) || (value.Target !== 'accept'))
    return [];
  if (!Array.isArray(value.Expressions,))
    return [];
  return value.Expressions
    .flatMap(function expressionPorts(expression,): readonly number[] {
      if (!isRecord(expression,))
        return [];
      /**
       Statement candidate from current expression.
       */
      const { Statement: statement, } = expression;
      if ((!isRecord(statement,)) || (statement.Name !== 'udp'))
        return [];
      if (!Array.isArray(statement.Values,))
        return [];
      return statement
        .Values
        .flatMap(exactUdpPort,);
    },);
}
