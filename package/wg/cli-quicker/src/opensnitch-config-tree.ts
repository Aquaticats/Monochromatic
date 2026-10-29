import { OpenSnitchConfigError, } from './errors.ts';
import {
  acceptedUdpPorts,
  createManagedRule,
  isManagedRule,
  managedPrefix,
} from './opensnitch-rule.ts';

/**
 OpenSnitch system-firewall schema version supported by this integration.
 */
const SUPPORTED_VERSION = 1;

/**
 JSON object with unknown fields retained during round-trip.
 */
export type JsonRecord = Readonly<Record<string, unknown>>;

/**
 Reconciled OpenSnitch document and whether disk write is necessary.
 */
export type OpenSnitchConfigMutation = {
  /**
   Document preserving unknown and legacy fields.
   */
  readonly document: JsonRecord;

  /**
   Whether managed rule set differs from requested ports.
   */
  readonly changed: boolean;

  /**
   Sorted distinct destination ports represented by managed rules.
   */
  readonly managedPorts: readonly number[];

  /**
   Formerly managed exact ports with no remaining accepting rule.
   */
  readonly forbiddenPorts: readonly number[];
};

/**
 Reports non-null JSON object.
 
 @param value - Unknown JSON value.
 
 @returns Whether value is object record.
 
 @example
 ```ts
 isRecord({ Version: 1 });
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
 Parses untrusted JSON with OpenSnitch-specific syntax diagnostic.
 
 @param text - Full JSON document.
 
 @param path - Source path used in diagnostics.
 
 @returns Parsed unknown JSON value.
 
 @throws {@link OpenSnitchConfigError} when JSON syntax is invalid.
 
 @example
 ```ts
 parseJson({ text: '{"Enabled":true}', path: '/etc/opensnitchd/system-fw.json' });
 ```
 */
function parseJson(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    throw new OpenSnitchConfigError(
      `OpenSnitch system-firewall config is not valid JSON: ${path}`,
      { cause: error, },
    );
  }
}

/**
 Parses OpenSnitch system-firewall JSON without discarding unknown fields.
 
 @param text - Full JSON document.
 
 @param path - Source path used in diagnostics.
 
 @returns Parsed object document.
 
 @throws {@link OpenSnitchConfigError} when JSON or root shape is invalid.
 
 @example
 ```ts
 parseOpenSnitchConfig({ text: '{"Enabled":true}', path: '/etc/opensnitchd/system-fw.json' });
 ```
 */
export function parseOpenSnitchConfig(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): JsonRecord {
  /**
   Untrusted parsed JSON root.
   */
  const parsed = parseJson({
    text,
    path,
  },);
  if (!isRecord(parsed,))
    throw new OpenSnitchConfigError(`OpenSnitch system-firewall config has invalid root: ${path}`,);
  return parsed;
}

/**
 Finds exactly one nftables output-mangle chain in version 1 document.
 
 @param document - Parsed OpenSnitch system-firewall document.
 
 @param path - Source path used in diagnostics.
 
 @param requireEnabled - Whether disabled system firewall must reject installation.
 
 @returns Target chain object and its parent entry.
 
 @throws {@link OpenSnitchConfigError} when schema is unsupported or ambiguous.
 
 @example
 ```ts
 findTargetChain({ document, path: '/tmp/system-fw.json', requireEnabled: true });
 ```
 */
function findTargetChain(
  {
    document,
    path,
    requireEnabled,
  }: {
    readonly document: Readonly<JsonRecord>;
    readonly path: string;
    readonly requireEnabled: boolean;
  },
): {
  readonly chain: JsonRecord;
  readonly parent: JsonRecord;
  readonly parentChains: readonly unknown[];
  readonly rules: readonly unknown[];
  readonly systemRules: readonly unknown[];
} {
  if (document.Version !== SUPPORTED_VERSION) {
    throw new OpenSnitchConfigError(
      `OpenSnitch system-firewall config version is unsupported at ${path}; expected 1.`,
    );
  }
  if (requireEnabled && (document.Enabled !== true)) {
    throw new OpenSnitchConfigError(
      `OpenSnitch system firewall is disabled at ${path}; enable it before bringing tunnel up.`,
    );
  }
  if (!Array.isArray(document.SystemRules,))
    throw new OpenSnitchConfigError(`OpenSnitch SystemRules is missing at ${path}.`,);
  /**
   Validated top-level system-rule entries.
   */
  const systemRules = document.SystemRules;
  /**
   Matching chain-parent pairs across system-rule entries.
   */
  const matches = systemRules
    .flatMap(function matchingChains(entry,): readonly {
      readonly chain: JsonRecord;
      readonly parent: JsonRecord;
      readonly parentChains: readonly unknown[];
    }[] {
      if ((!isRecord(entry,)) || (!Array.isArray(entry.Chains,)))
        return [];
      /**
       Validated chains array from current parent entry.
       */
      const parentChains = entry.Chains;
      return parentChains
        .filter(function isTarget(chain,): chain is JsonRecord {
          return isRecord(chain,)
            && (chain.Name === 'mangle_output')
            && (chain.Table === 'opensnitch')
            && (chain.Family === 'inet');
        },)
        .map(function pair(chain: Readonly<JsonRecord>,) {
          return {
            chain,
            parent: entry,
            parentChains,
          };
        },);
    },);
  if (matches.length !== 1) {
    throw new OpenSnitchConfigError(
      `OpenSnitch config at ${path} must contain exactly one inet opensnitch mangle_output chain.`,
    );
  }
  /**
   Sole validated target-chain match.
   */
  const [match,] = matches;
  if (match === undefined)
    throw new OpenSnitchConfigError(`OpenSnitch mangle_output chain is missing at ${path}.`,);
  /**
   Rules candidate from sole matched chain.
   */
  const rules = match
    .chain
    .Rules;
  if (!Array.isArray(rules,))
    throw new OpenSnitchConfigError(`OpenSnitch mangle_output Rules is invalid at ${path}.`,);
  return {
    ...match,
    rules,
    systemRules,
  };
}

/**
 Reconciles interface-owned endpoint allowances in OpenSnitch version 1 tree.
 
 Empty `endpointPorts` removes every managed rule for interface.
 Unknown fields and unrelated rules retain identity and content.
 
 @param document - Parsed OpenSnitch system-firewall document.
 
 @param interfaceName - WireGuard interface owning managed rules.
 
 @param endpointPorts - Desired endpoint UDP ports.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @param path - Source path used in diagnostics.
 
 @param requireEnabled - Whether disabled firewall rejects operation.
 
 @param previousManagedPorts - Persisted ports needing negative recovery verification.
 
 @returns Reconciled document and change metadata.
 
 @throws {@link OpenSnitchConfigError} when schema cannot be changed safely.
 
 @example
 ```ts
 reconcileOpenSnitchConfig({
   document,
   interfaceName: 'wg0',
   endpointPorts: [51820],
   networkNamespaceKey: 'abc123',
   path: '/etc/opensnitchd/system-fw.json',
   requireEnabled: true,
 });
 ```
 */
export function reconcileOpenSnitchConfig(
  {
    document,
    interfaceName,
    endpointPorts,
    networkNamespaceKey = '',
    path,
    requireEnabled,
    previousManagedPorts = [],
  }: {
    readonly document: JsonRecord;
    readonly interfaceName: string;
    readonly endpointPorts: readonly number[];
    readonly networkNamespaceKey?: string;
    readonly path: string;
    readonly requireEnabled: boolean;
    readonly previousManagedPorts?: readonly number[];
  },
): OpenSnitchConfigMutation {
  /**
   Validated target path and arrays used for immutable replacement.
   */
  const {
    chain: targetChain,
    parent: targetParent,
    parentChains,
    rules: existingRules,
    systemRules,
  } = findTargetChain({
    document,
    path,
    requireEnabled,
  },);
  /**
   Desired sorted unique ports for deterministic config rendering.
   */
  const managedPorts = [...new Set(endpointPorts,),]
    .toSorted(function ascending(
      a,
      b,
    ): number {
      return a - b;
    },);
  /**
   Interface-owned description prefix.
   */
  const prefix = managedPrefix({
    interfaceName,
    networkNamespaceKey,
  },);
  /**
   Unrelated and other-interface rules retained in order.
   */
  const retainedRules = existingRules.filter(function retainRule(rule,): boolean {
    return !isManagedRule({
      value: rule,
      prefix,
    },);
  },);
  /**
   Exact ports accepted by removed interface-owned rules.
   */
  const removedManagedPorts = [
    ...previousManagedPorts,
    ...existingRules
      .filter(function ownedRule(rule,): boolean {
        return isManagedRule({
          value: rule,
          prefix,
        },);
      },)
      .flatMap(function removedPorts(rule,): readonly number[] {
        return acceptedUdpPorts({ value: rule, },);
      },),
  ];
  /**
   Exact ports still accepted by unrelated or other-interface rules.
   */
  const retainedAcceptedPorts = new Set(retainedRules.flatMap(function retainedPorts(
    rule,
  ): readonly number[] {
    return acceptedUdpPorts({ value: rule, },);
  },),);
  /**
   Fresh managed rules matching current endpoint ports.
   */
  const managedRules = managedPorts.map(function toRule(port,): JsonRecord {
    return createManagedRule({
      interfaceName,
      networkNamespaceKey,
      port,
    },);
  },);
  /**
   Removed ports expected to disappear from live chain.
   */
  const forbiddenPorts = [...new Set(removedManagedPorts,),]
    .filter(function noRemainingAllowance(port,): boolean {
      return (!managedPorts.includes(port,)) && (!retainedAcceptedPorts.has(port,));
    },)
    .toSorted(function ascending(
      a,
      b,
    ): number {
      return a - b;
    },);
  /**
   Existing managed descriptions used to avoid unnecessary write when removing none.
   */
  const existingManagedCount = existingRules.length - retainedRules.length;
  if ((existingManagedCount === 0) && (managedRules.length === 0)) {
    return {
      document,
      changed: false,
      managedPorts,
      forbiddenPorts,
    };
  }
  /**
   Replacement target chain preserving unknown chain fields.
   */
  const replacementChain: JsonRecord = {
    ...targetChain,
    Rules: [
      ...retainedRules,
      ...managedRules,
    ],
  };
  /**
   Replacement parent retaining every non-target chain.
   */
  const replacementParent: JsonRecord = {
    ...targetParent,
    Chains: parentChains.map(function replaceChain(chain,): unknown {
      return chain === targetChain ? replacementChain : chain;
    },),
  };
  return {
    document: {
      ...document,
      SystemRules: systemRules.map(function replaceParent(entry,): unknown {
        return entry === targetParent ? replacementParent : entry;
      },),
    },
    changed: true,
    managedPorts,
    forbiddenPorts,
  };
}

/**
 Renders validated OpenSnitch document with stable trailing newline.
 
 @param document - Reconciled JSON object.
 
 @returns Pretty-printed JSON suitable for OpenSnitch file watcher.
 
 @example
 ```ts
 renderOpenSnitchConfig({ document });
 ```
 */
export function renderOpenSnitchConfig(
  { document, }: { readonly document: Readonly<JsonRecord>; },
): string {
  /**
   Pretty-printed body before required trailing newline.
   */
  const serialized = JSON.stringify(
    document,
    null,
    2,
  );
  return `${serialized}\n`;
}
