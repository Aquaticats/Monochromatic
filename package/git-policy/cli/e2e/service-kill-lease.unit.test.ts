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
 How the service's source must start the lease's line, after its indentation.
 A commented-out declaration starts with `//` instead, so it is never read as the lease.
 */
const LEASE_HEAD = 'pub const CLAIM_LEASE: Duration = Duration::from_secs(';

/**
 How the lease's line must end.
 */
const LEASE_TAIL = ');';

/**
 Whether a line of Rust source declares the claim lease, in any shape.

 @param line - one line, indentation removed

 @returns true for a line that starts a `CLAIM_LEASE` declaration, `pub` or not
 */
function declaresLease(line: string,): boolean {
  return line.startsWith('pub const CLAIM_LEASE:',) || line.startsWith('const CLAIM_LEASE:',);
}

/**
 The lease the service gives a claim, in milliseconds, read from the service's source.

 @returns the lease

 @throws Error when the source no longer declares the lease once, on one line, as
 `pub const CLAIM_LEASE: Duration = Duration::from_secs(<n>);`
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
   Every line that declares the lease, indentation removed.
   */
  const declarations = source
    .split('\n',)
    .map(function trimmed(line,) {
      return line.trim();
    },)
    .filter(declaresLease,);
  /**
   The one declaration, empty when there is none or more than one.
   */
  const declaration = declarations.length === 1 ? declarations[0] ?? '' : '';
  /**
   The seconds between the head and the tail, empty when the declaration has another shape.
   */
  const seconds = declaration.startsWith(LEASE_HEAD,) && declaration.endsWith(LEASE_TAIL,)
    ? declaration.slice(
      LEASE_HEAD.length,
      declaration.length - LEASE_TAIL.length,
    )
    : '';
  if ((seconds === '') || (String(Number(seconds,),) !== seconds))
    throw new Error(`src/native/landing/service_work.rs must declare CLAIM_LEASE once, on one line, as ${LEASE_HEAD}<n>${LEASE_TAIL} found ${String(declarations.length,)} declarations; update service-kill-lease.unit.test.ts and scenario-service-kill-fixture.ts`,);
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
