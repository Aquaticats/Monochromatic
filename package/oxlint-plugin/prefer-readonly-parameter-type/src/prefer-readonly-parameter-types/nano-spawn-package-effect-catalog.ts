/**
 * Audited nano-spawn package effects.
 *
 * @module
 */

import type { IntrinsicEffectEntry, } from './intrinsic-effect-catalog.ts';

/**
 * Exact nano-spawn package provenance.
 */
const NANO_SPAWN_PROVENANCE = {
  kind: 'package',
  packageName: 'nano-spawn',
  major: 2,
} as const;

/**
 * nano-spawn implementation audit identity.
 */
const NANO_SPAWN_EVIDENCE = 'nano-spawn 2.1.0 source/index.js sha256 f0e98f616b0725411637c96982c9c01559119f29a290965b51dcd0e59c722583, source/options.js sha256 b8b0e1c4cf652c88bdec37fe61f950cfbda8304a13f8e200e6e465148a6cdd34, and source/spawn.js sha256 f46b8b556c02718229f41891b648a46de570bd0a6459a1f44f007ae4c33fc57c';

/**
 * Audited nano-spawn effects used by subprocess-owning packages.
 */
export const NANO_SPAWN_PACKAGE_EFFECTS: readonly IntrinsicEffectEntry[] = [{
  provenance: NANO_SPAWN_PROVENANCE,
  ownerType: 'globalThis',
  member: 'default',
  targets: [{
    kind: 'argument',
    index: 2,
  },],
  opaqueTargets: [
    {
      kind: 'argument',
      index: 1,
    },
    {
      kind: 'argument',
      index: 2,
    },
  ],
  evidence: `${NANO_SPAWN_EVIDENCE}; spawn reads iterable arguments and option properties, copies their containers, then forwards retained AbortSignal and stream capabilities to node:child_process.spawn`,
},];
