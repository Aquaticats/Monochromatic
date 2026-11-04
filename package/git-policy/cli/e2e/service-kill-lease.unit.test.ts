import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CLAIM_LEASE_MS,
  CLAIM_RECOVERY_BUDGET_MS,
} from './scenario-service-kill-fixture.ts';
import { SCENARIO_TIMEOUT_MS, } from './scenario-model-fixture.ts';

/**
 Milliseconds in a second, for the lease the service spells in seconds.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 The lease the service gives a claim, in milliseconds, read from the service's source.

 @returns the lease

 @throws Error when the source no longer spells the lease as `CLAIM_LEASE: Duration = Duration::from_secs(<n>)`
 */
async function serviceClaimLease(): Promise<number> {
  /**
   The service's after-commit work source.
   */
  const source = await readFile(
    join(
      import.meta.dirname,
      '..',
      'src',
      'native',
      'landing',
      'service_work.rs',
    ),
    'utf8',
  );
  /**
   The source without its comments, so a commented-out declaration cannot be read as the lease.
   */
  const code = source.replaceAll(
    /\/\*[\s\S]*?\*\/|\/\/[^\n]*/gv,
    '',
  );
  /**
   Every declaration of the lease.
   */
  const declarations = [...code.matchAll(/\b(?:pub\s+)?const\s+CLAIM_LEASE\b[^;]*;/gv,),];
  /**
   The lease in seconds.
   */
  const seconds = /^(?:pub\s+)?const\s+CLAIM_LEASE:\s*Duration\s*=\s*Duration::from_secs\((?<seconds>\d+)\);$/v
    .exec(declarations[0]?.[0] ?? '',)?.groups?.['seconds'];
  if ((declarations.length !== 1) || (seconds === undefined))
    throw new Error(`src/native/landing/service_work.rs must declare CLAIM_LEASE once, as pub const CLAIM_LEASE: Duration = Duration::from_secs(<n>); found ${String(declarations.length,)} declarations; update service-kill-lease.unit.test.ts and scenario-service-kill-fixture.ts`,);
  return Number(seconds,) * MILLISECONDS_PER_SECOND;
}

await describe({
  name: 'the kill scenarios\' claim recovery wait',
  children: [
    it({
      name: 'equals the lease the service gives a claim',
      fn: async () => {
        expect(CLAIM_LEASE_MS,).toBe(await serviceClaimLease(),);
      },
    },),
    it({
      name: 'outlasts the lease and fits in the scenario timeout',
      fn: async () => {
        expect(CLAIM_RECOVERY_BUDGET_MS,).toBeGreaterThan(await serviceClaimLease(),);
        expect(CLAIM_RECOVERY_BUDGET_MS,).toBeLessThan(SCENARIO_TIMEOUT_MS,);
      },
    },),
  ],
},);
