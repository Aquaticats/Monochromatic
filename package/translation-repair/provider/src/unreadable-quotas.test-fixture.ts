/**
 A Synthetic quota read that cannot be understood, as XingZ6020's was on 2026-10-23: the `/quotas` body carried no
 `weeklyTokenLimit` object, so the budgets count the meter unreadable and the provider spendable.

 Fixtures are cat-themed invention; no HTTP is made.

 @module
 */

/**
 Reads Synthetic's quotas and fails the way an unexpected body does.

 @returns A rejected read

 @example
 ```ts
 const budgets = createProviderBudgets({ maxHoldMs: 86_400_000, synthetic: { quotas: unreadableQuotas, }, },);
 ```
 */
export function unreadableQuotas(): Promise<never> {
  return Promise.reject(new Error('Synthetic /quotas body violated expectations',),);
}
