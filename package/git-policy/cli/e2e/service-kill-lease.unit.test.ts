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

/**
 Milliseconds in a second, for the lease the service spells in seconds.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Upper bound of a scenario's workload (`SCENARIO_TIMEOUT_MS` in `scenario-model-fixture.ts`) that the recovery wait must stay under.
 */
const SCENARIO_TIMEOUT_MS = 180_000;

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
   The lease declaration.
   */
  const declaration = /pub const CLAIM_LEASE: Duration = Duration::from_secs\((?<seconds>\d+)\);/v
    .exec(source,);
  /**
   The lease in seconds.
   */
  const seconds = declaration?.groups?.['seconds'];
  if (seconds === undefined)
    throw new Error('src/native/landing/service_work.rs no longer declares CLAIM_LEASE as Duration::from_secs(<n>); update service-kill-lease.unit.test.ts and scenario-service-kill-fixture.ts',);
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
