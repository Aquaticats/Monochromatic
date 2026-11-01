/**
 Helpers shared by the scenarios that `tunnel-traffic-block.integration.test.ts` runs inside disposable user and
 network namespaces:
 they read the packet counters of the block's rules from `nft -j list table` and report which rule counted a packet.

 @module
 */

import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  run,
  runAllowingFailure,
} from '../dist/final/node/runner.mjs';
import { trafficBlockTable, } from '../dist/final/node/tunnel-traffic-block.mjs';
import type { TempDir, } from './tempdir.ts';

/**
 Rule kinds the scenarios tell apart.
 */
export type RuleKind = 'mark' | 'hosts4' | 'hosts6' | 'drop';

/**
 Chains of the block.
 */
export type Chain = 'output' | 'forward';

/**
 What one probe saw.
 */
export type Probed = {
  /**
   Kinds of rule whose counters grew while the packet was sent.
   */
  readonly counted: readonly RuleKind[];

  /**
   Exit status of the sender.
   */
  readonly status: number;

  /**
   Standard output of the sender.
   */
  readonly output: string;
};

/**
 One expression of a rule in the JSON of `nft -j list table`,
 reduced to the parts the scenarios read.
 */
type Expression = {
  readonly match?: {
    readonly left?: {
      readonly meta?: { readonly key?: string; };
      readonly payload?: { readonly protocol?: string; };
    };
  };
  readonly counter?: { readonly packets?: number; };
  readonly drop?: unknown;
};

/**
 One rule of the JSON listing.
 */
type ListedRule = {
  readonly chain?: string;
  readonly expr?: readonly Expression[];
};

/**
 The JSON listing of a table,
 reduced to its rules.
 */
type Listing = {
  readonly nftables: readonly { readonly rule?: ListedRule; }[];
};

/**
 One rule's kind and the packets it counted.
 */
type Counted = {
  readonly kind: RuleKind;
  readonly packets: number;
};

/**
 Narrows parsed JSON to the listing the scenarios read.

 @param value - Parsed JSON.

 @returns Whether it has the list of `nftables` entries.

 @example
 ```ts
 isListing({ nftables: [] }); // true
 ```
 */
function isListing(value: unknown,): value is Listing {
  return ((typeof value) === 'object') && (value !== null)
    && ('nftables' in value)
    && Array.isArray(value.nftables,);
}

/**
 Makes a scratch directory in the test scratch root,
 removed when the scope exits,
 because the real one may belong to another user inside the namespace.

 @returns A disposable directory.

 @example
 ```ts
 await using dir = await scratchDirectory();
 ```
 */
export async function scratchDirectory(): Promise<TempDir> {
  /**
   Path of the new directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'wg-quicker-scenario-',
  ),);
  return {
    path,
    [Symbol.asyncDispose]: async function removeScratchDirectory(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Which kinds a listed rule is.

 @param rule - Rule of the listing.

 @returns The kinds it has,
 which is none for a rule the scenarios do not know.

 @example
 ```ts
 kindsOf({ rule: { expr: [{ drop: null }] } }); // ['drop']
 ```
 */
function kindsOf({ rule, }: { readonly rule: ListedRule; },): readonly RuleKind[] {
  /**
   Expressions of the rule.
   */
  const expressions = rule.expr ?? [];
  if (expressions.some(function dropsPacket(expression,): boolean {
    return 'drop' in expression;
  },))
    return ['drop',];
  if (expressions.some(function matchesMark(expression,): boolean {
    return expression.match
      ?.left
      ?.meta
      ?.key
      === 'mark';
  },))
    return ['mark',];
  if (expressions.some(function matchesV4(expression,): boolean {
    return expression.match
      ?.left
      ?.payload
      ?.protocol
      === 'ip';
  },))
    return ['hosts4',];
  if (expressions.some(function matchesV6(expression,): boolean {
    return expression.match
      ?.left
      ?.payload
      ?.protocol
      === 'ip6';
  },))
    return ['hosts6',];
  return [];
}

/**
 Reads the packet counters of the rules of one chain.

 @param interfaceName - Interface the block is made for.

 @param chain - Chain of the block.

 @returns The kinds in the order of the rules,
 each with the packets it counted.

 @example
 ```ts
 await listCounters({ interfaceName: 'wgclear', chain: 'output' });
 ```
 */
export async function listCounters(
  {
    interfaceName,
    chain,
  }: {
    readonly interfaceName: string;
    readonly chain: Chain
  },
): Promise<readonly Counted[]> {
  /**
   Listing of the table as JSON.
   */
  const parsed: unknown = JSON.parse((await run({
    command: 'nft',
    args: [
      '-j',
      'list',
      'table',
      'inet',
      trafficBlockTable({ interfaceName, },),
    ],
  },)).stdout,);
  if (!isListing(parsed,))
    throw new Error('nft did not list the table as JSON with an nftables array',);
  return parsed.nftables
    .flatMap(function ruleOf(entry,): readonly ListedRule[] {
      return (entry.rule === undefined) || (entry.rule
        .chain
        !== chain) ? [] : [entry.rule,];
    },)
    .flatMap(function counted(rule,): readonly Counted[] {
      return kindsOf({ rule, },)
        .map(function withPackets(kind,): Counted {
          return {
            kind,
            packets: rule.expr
              ?.find(function hasCounter(expression,): boolean {
                return expression.counter !== undefined;
              },)
              ?.counter
              ?.packets
              ?? 0,
          };
        },);
    },);
}

/**
 Runs a command and reports which rules of a chain counted a packet while it ran.

 @param interfaceName - Interface the block is made for.

 @param chain - Chain whose counters are read.

 @param command - Executable that sends one packet.

 @param args - Arguments.

 @returns The kinds whose counters grew,
 and the exit status and output of the command.

 @example
 ```ts
 await probe({ interfaceName: 'wgclear', chain: 'output', command: 'ping', args: ['-c1', '10.99.0.7'] });
 ```
 */
export async function probe(
  {
    interfaceName,
    chain,
    command,
    args,
  }: {
    readonly interfaceName: string;
    readonly chain: Chain;
    readonly command: string;
    readonly args: readonly string[];
  },
): Promise<Probed> {
  /**
   Counters before.
   */
  const before = await listCounters({
    interfaceName,
    chain,
  },);
  /**
   The command's outcome.
   */
  const result = await runAllowingFailure({
    command,
    args,
  },);
  /**
   Counters after.
   */
  const after = await listCounters({
    interfaceName,
    chain,
  },);
  return {
    counted: after
      .filter(function grew(
        rule,
        index,
      ): boolean {
        return rule.packets > (before[index]
          ?.packets
          ?? 0);
      },)
      .map(function kindOfGrown(rule,): RuleKind {
        return rule.kind;
      },),
    status: result.exitCode,
    output: result.stdout,
  };
}

/**
 Kinds of the rules of one chain,
 in order.

 @param interfaceName - Interface the block is made for.

 @param chain - Chain of the block.

 @returns The kinds in the order of the rules.

 @example
 ```ts
 await ruleOrder({ interfaceName: 'wgclear', chain: 'forward' });
 ```
 */
export async function ruleOrder(
  {
    interfaceName,
    chain,
  }: {
    readonly interfaceName: string;
    readonly chain: Chain
  },
): Promise<readonly RuleKind[]> {
  return (await listCounters({
    interfaceName,
    chain,
  },)).map(function kindOfRule(rule,): RuleKind {
    return rule.kind;
  },);
}
