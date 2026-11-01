/** Exhausted-credit evidence and per-operation provider exclusion. @module */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { assertAdvisorProviderAvailable, createAdvisorOperationLedger, isAdvisorCreditExhaustion, nextAdvisorCandidate, NO_ADVISOR_CANDIDATE, } from '../dist/final/node/index.mjs';

/** Same registered provider owns both Hyper endpoints. */
const candidates = [
  { model: 'hyper/first', provider: 'hyper', },
  { model: 'hyper/second', provider: 'hyper', },
  { model: 'other/third', provider: 'other', },
];

await describe({ name: '', children: [
  ...[
    `402: {"message":"You're out of credits. Add more at https://hyper.charm.land","type":"billing_error","code":null}`,
    'INSUFFICIENT_QUOTA', 'insufficient credits', 'credit balance is too low', 'INSUFFICIENT_G1_CREDITS_BALANCE',
  ].map(diagnostic => it({ name: `recognizes ${diagnostic}`, fn: async (): Promise<void> => {
    expect(isAdvisorCreditExhaustion(diagnostic,),).toBe(true,);
  }, },),),
  ...['402 status code (no body)', 'billing_error', 'payment authorization denied', '429 rate limit', 'quota exceeded', 'network error', '',]
    .map(diagnostic => it({ name: `does not invent exhausted credits for ${diagnostic || 'empty diagnostic'}`, fn: async (): Promise<void> => {
      expect(isAdvisorCreditExhaustion(diagnostic,),).toBe(false,);
    }, },),),
  it({ name: 'excludes every endpoint of the exhausted provider but starts fresh next call', fn: async (): Promise<void> => {
    /** First operation learns exhausted credits from an actual failure. */
    const ledger = createAdvisorOperationLedger({ startedAtMs: 0, deadlineAtMs: 100, },);
    ledger.blockProvider('hyper',);
    expect(nextAdvisorCandidate({ candidates, attemptedModels: [], runningProviders: [], blockedProviders: ledger.snapshot().blockedProviders, },),)
      .toEqual(candidates[2],);
    expect(() => assertAdvisorProviderAvailable({ provider: 'hyper', blockedProviders: ledger.snapshot().blockedProviders, },),)
      .toThrow('blocked for this call',);
    assertAdvisorProviderAvailable({ provider: 'other', blockedProviders: ledger.snapshot().blockedProviders, },);
    /** Another operation must not inherit the block. */
    const fresh = createAdvisorOperationLedger({ startedAtMs: 0, deadlineAtMs: 100, },);
    expect(nextAdvisorCandidate({ candidates, attemptedModels: [], runningProviders: [], blockedProviders: fresh.snapshot().blockedProviders, },),)
      .toEqual(candidates[0],);
  }, },),
  it({ name: 'prefers another provider but permits same-provider alternatives when necessary', fn: async (): Promise<void> => {
    expect(nextAdvisorCandidate({ candidates, attemptedModels: ['hyper/first',], blockedProviders: [], runningProviders: ['hyper',], },),)
      .toEqual(candidates[2],);
    expect(nextAdvisorCandidate({ candidates, attemptedModels: ['hyper/first', 'other/third',], blockedProviders: [], runningProviders: ['hyper',], },),)
      .toEqual(candidates[1],);
    expect(nextAdvisorCandidate({ candidates, attemptedModels: ['hyper/first', 'hyper/second', 'other/third',], blockedProviders: [], runningProviders: [], },),)
      .toBe(NO_ADVISOR_CANDIDATE,);
    expect(nextAdvisorCandidate({ candidates: [], attemptedModels: [], blockedProviders: [], runningProviders: [], },),)
      .toBe(NO_ADVISOR_CANDIDATE,);
  }, },),
], },);
