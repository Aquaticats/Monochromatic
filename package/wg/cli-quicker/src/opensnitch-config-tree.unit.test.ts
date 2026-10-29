import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  parseOpenSnitchConfig,
  reconcileOpenSnitchConfig,
  renderOpenSnitchConfig,
} from '../dist/final/node/opensnitch-config-tree.mjs';

/**
 Creates minimal OpenSnitch 1.8 nftables system-firewall document.
 
 @param rules - Initial mangle-output rules.
 
 @returns Document carrying unrelated legacy and future fields.
 
 @example
 ```ts
 fixtureDocument({ rules: [] });
 ```
 */
function fixtureDocument(
  { rules, }: { readonly rules: readonly unknown[]; },
): Record<string, unknown> {
  return {
    Enabled: true,
    Version: 1,
    FutureTopLevel: { retained: true, },
    SystemRules: [
      {
        Rule: {
          Table: 'mangle',
          Chain: 'OUTPUT',
          Parameters: '-p icmp',
        },
        Chains: [],
      },
      {
        FutureEntry: 'retained',
        Chains: [
          {
            Name: 'filter_input',
            Table: 'opensnitch',
            Family: 'inet',
            Rules: [],
          },
          {
            Name: 'mangle_output',
            Table: 'opensnitch',
            Family: 'inet',
            FutureChain: 7,
            Rules: [...rules,],
          },
        ],
      },
    ],
  };
}

/**
 Finds target-chain rules in reconciled fixture document.
 
 @param document - OpenSnitch document.
 
 @returns Target rules.
 
 @example
 ```ts
 targetRules({ document });
 ```
 */
function targetRules(
  { document, }: { readonly document: Readonly<Record<string, unknown>>; },
): readonly Record<string, unknown>[] {
  const systemRules = document.SystemRules as readonly Record<string, unknown>[];
  const chains = systemRules.flatMap(function readChains(entry,): readonly Record<string, unknown>[] {
    return (entry.Chains ?? []) as readonly Record<string, unknown>[];
  },);
  const target = chains.find(function isTarget(chain,): boolean {
    return chain.Name === 'mangle_output';
  },);
  return (target?.Rules ?? []) as readonly Record<string, unknown>[];
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: reconcileOpenSnitchConfig.name,
      children: [
        it({
          name: 'adds sorted distinct managed rules while preserving unknown and legacy fields',
          fn: async () => {
            const userRule = {
              Enabled: true,
              Description: 'user rule',
              FutureRule: 'retained',
            };
            const result = reconcileOpenSnitchConfig({
              document: fixtureDocument({ rules: [userRule,], }),
              interfaceName: 'wg0',
              endpointPorts: [51_820, 2_049, 51_820,],
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            expect(result.changed,).toBe(true,);
            expect(result.managedPorts,).toEqual([2_049, 51_820,],);
            expect(result.document.FutureTopLevel,).toEqual({ retained: true, },);
            const rules = targetRules({ document: result.document, },);
            expect(rules[0],).toBe(userRule,);
            expect(rules,).toHaveLength(3,);
            expect(rules[1],).toMatchObject({
              Enabled: true,
              Position: '0',
              Description: 'wg-quicker managed endpoint [wg0] UDP destination port 2049',
              Target: 'accept',
            },);
            expect(rules[2],).toMatchObject({
              Description: 'wg-quicker managed endpoint [wg0] UDP destination port 51820',
            },);
            expect(String(rules[1]?.UUID,),).toHaveLength(36,);
          },
        },),

        it({
          name: 'replaces stale rules only for requested interface',
          fn: async () => {
            const wg0Stale = {
              Description: 'wg-quicker managed endpoint [wg0] UDP destination port 1',
            };
            const wg1Rule = {
              Description: 'wg-quicker managed endpoint [wg1] UDP destination port 2',
            };
            const result = reconcileOpenSnitchConfig({
              document: fixtureDocument({ rules: [wg0Stale, wg1Rule,], }),
              interfaceName: 'wg0',
              endpointPorts: [3,],
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            const rules = targetRules({ document: result.document, },);
            expect(rules,).toHaveLength(2,);
            expect(rules[0],).toBe(wg1Rule,);
            expect(rules[1]?.Description,).toBe(
              'wg-quicker managed endpoint [wg0] UDP destination port 3',
            );
          },
        },),

        it({
          name: 'reports replaced exact port that must disappear from live chain',
          fn: async () => {
            const installed = reconcileOpenSnitchConfig({
              document: fixtureDocument({ rules: [], }),
              interfaceName: 'wg0',
              endpointPorts: [2_049,],
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            const replaced = reconcileOpenSnitchConfig({
              document: installed.document,
              interfaceName: 'wg0',
              endpointPorts: [51_820,],
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            expect(replaced.forbiddenPorts,).toEqual([2_049,],);
          },
        },),

        it({
          name: 'retains same interface rules owned by another network namespace',
          fn: async () => {
            const first = reconcileOpenSnitchConfig({
              document: fixtureDocument({ rules: [], }),
              interfaceName: 'wg0',
              endpointPorts: [2_049,],
              networkNamespaceKey: 'namespace-a',
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            const second = reconcileOpenSnitchConfig({
              document: first.document,
              interfaceName: 'wg0',
              endpointPorts: [51_820,],
              networkNamespaceKey: 'namespace-b',
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            /**
             Serialized document proving both namespace owners coexist.
             */
            const serialized = JSON.stringify(second.document,);
            expect(serialized,).toContain('[netns:namespace-a]',);
            expect(serialized,).toContain('[netns:namespace-b]',);
            const removedSecond = reconcileOpenSnitchConfig({
              document: second.document,
              interfaceName: 'wg0',
              endpointPorts: [],
              networkNamespaceKey: 'namespace-b',
              path: '/tmp/system-fw.json',
              requireEnabled: false,
            },);
            expect(JSON.stringify(removedSecond.document,),).toContain('[netns:namespace-a]',);
            expect(JSON.stringify(removedSecond.document,),).not.toContain('[netns:namespace-b]',);
          },
        },),

        it({
          name: 'recovers persisted port after JSON rule was already removed',
          fn: async () => {
            const recovered = reconcileOpenSnitchConfig({
              document: fixtureDocument({ rules: [], }),
              interfaceName: 'wg0',
              endpointPorts: [],
              previousManagedPorts: [2_049,],
              path: '/tmp/system-fw.json',
              requireEnabled: false,
            },);
            expect(recovered.changed,).toBe(false,);
            expect(recovered.forbiddenPorts,).toEqual([2_049,],);
          },
        },),

        it({
          name: 'retains shared exact port when another interface owns allowance',
          fn: async () => {
            const first = reconcileOpenSnitchConfig({
              document: fixtureDocument({ rules: [], }),
              interfaceName: 'wg0',
              endpointPorts: [2_049,],
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            const shared = reconcileOpenSnitchConfig({
              document: first.document,
              interfaceName: 'wg1',
              endpointPorts: [2_049,],
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            const removed = reconcileOpenSnitchConfig({
              document: shared.document,
              interfaceName: 'wg0',
              endpointPorts: [],
              path: '/tmp/system-fw.json',
              requireEnabled: false,
            },);
            expect(removed.forbiddenPorts,).toEqual([],);
          },
        },),

        it({
          name: 'removes managed rules while disabled during teardown',
          fn: async () => {
            const document = fixtureDocument({
              rules: [{ Description: 'wg-quicker managed endpoint [wg0] UDP destination port 2049', },],
            },);
            document.Enabled = false;
            const result = reconcileOpenSnitchConfig({
              document,
              interfaceName: 'wg0',
              endpointPorts: [],
              path: '/tmp/system-fw.json',
              requireEnabled: false,
            },);
            expect(targetRules({ document: result.document, }),).toEqual([],);
          },
        },),

        it({
          name: 'returns unchanged document when no managed rules or ports exist',
          fn: async () => {
            const document = fixtureDocument({ rules: [], });
            const result = reconcileOpenSnitchConfig({
              document,
              interfaceName: 'wg0',
              endpointPorts: [],
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },);
            expect(result.changed,).toBe(false,);
            expect(result.document,).toBe(document,);
            expect(result.forbiddenPorts,).toEqual([],);
          },
        },),

        it({
          name: 'rejects disabled firewall during installation',
          fn: async () => {
            const document = fixtureDocument({ rules: [], });
            document.Enabled = false;
            expect(() => reconcileOpenSnitchConfig({
              document,
              interfaceName: 'wg0',
              endpointPorts: [51_820,],
              path: '/tmp/system-fw.json',
              requireEnabled: true,
            },),).toThrow('disabled',);
          },
        },),

        ...[
          {
            name: 'unsupported version',
            mutate: function unsupportedVersion(document: Record<string, unknown>,): void {
              document.Version = 2;
            },
            diagnostic: 'version is unsupported',
          },
          {
            name: 'missing system rules',
            mutate: function missingSystemRules(document: Record<string, unknown>,): void {
              delete document.SystemRules;
            },
            diagnostic: 'SystemRules is missing',
          },
          {
            name: 'missing target chain',
            mutate: function missingTargetChain(document: Record<string, unknown>,): void {
              document.SystemRules = [];
            },
            diagnostic: 'exactly one',
          },
          {
            name: 'duplicate target chain',
            mutate: function duplicateTargetChain(document: Record<string, unknown>,): void {
              document.SystemRules = [
                {
                  Chains: [
                    { Name: 'mangle_output', Table: 'opensnitch', Family: 'inet', Rules: [], },
                    { Name: 'mangle_output', Table: 'opensnitch', Family: 'inet', Rules: [], },
                  ],
                },
              ];
            },
            diagnostic: 'exactly one',
          },
          {
            name: 'invalid target rules',
            mutate: function invalidTargetRules(document: Record<string, unknown>,): void {
              document.SystemRules = [
                {
                  Chains: [
                    { Name: 'mangle_output', Table: 'opensnitch', Family: 'inet', Rules: 'invalid', },
                  ],
                },
              ];
            },
            diagnostic: 'Rules is invalid',
          },
        ].map(function invalidSchema(testCase,) {
          return it({
            name: `rejects ${testCase.name}`,
            fn: async () => {
              const document = fixtureDocument({ rules: [], });
              testCase.mutate(document,);
              expect(() => reconcileOpenSnitchConfig({
                document,
                interfaceName: 'wg0',
                endpointPorts: [51_820,],
                path: '/tmp/system-fw.json',
                requireEnabled: true,
              },),).toThrow(testCase.diagnostic,);
            },
          },);
        },),
      ],
    },),

    describe({
      name: parseOpenSnitchConfig.name,
      children: [
        it({
          name: 'parses object root',
          fn: async () => {
            expect(parseOpenSnitchConfig({
              text: '{"Version":1}',
              path: '/tmp/system-fw.json',
            },),).toEqual({ Version: 1, },);
          },
        },),
        it({
          name: 'rejects malformed JSON with source path',
          fn: async () => {
            expect(() => parseOpenSnitchConfig({
              text: '{',
              path: '/tmp/broken-system-fw.json',
            },),).toThrow('/tmp/broken-system-fw.json',);
          },
        },),
        it({
          name: 'rejects non-object root',
          fn: async () => {
            expect(() => parseOpenSnitchConfig({
              text: '[]',
              path: '/tmp/system-fw.json',
            },),).toThrow('invalid root',);
          },
        },),
      ],
    },),

    describe({
      name: renderOpenSnitchConfig.name,
      children: [
        it({
          name: 'renders indented JSON with trailing newline',
          fn: async () => {
            const rendered = renderOpenSnitchConfig({ document: { Version: 1, }, },);
            expect(rendered,).toBe('{\n  "Version": 1\n}\n',);
          },
        },),
      ],
    },),
  ],
},);
