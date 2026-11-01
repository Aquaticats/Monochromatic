/**
 * Declares mutation absent from implementation.
 *
 * @param controller - Capability overload claims to mutate.
 *
 * @mutates controller - Claims cancellation transition for overload.
 */
export function inconsistentReadonlyOverload(controller: AbortController,): void;

/**
 * Implements overload without mutation.
 *
 * @param controller - Capability inspected without transition.
 */
export function inconsistentReadonlyOverload(controller: AbortController,): void {
  void controller.signal;
}

/**
 * Declares a step-taking signature with no contract at all.
 *
 * @param apply - Holder whose step is invoked.
 *
 * @returns step result.
 */
export function invokedStepOverload(
  apply: { readonly step: (value: number,) => number; },
): number;

/**
 * Implements by invoking the supplied step and writing nothing.
 *
 * The shape `package/pipe/core` has. Nothing here mutates: the body destructures a
 * callable out of its parameter and calls it, which records an invoked capability
 * against parameter zero. A bodyless overload can seed `mutated` from an authored
 * contract and can never seed `invoked`, so the two sides of the overload comparison
 * are drawn from different vocabularies and no contract can close the gap.
 *
 * @param apply - Holder whose step is invoked.
 *
 * @returns step result.
 */
export function invokedStepOverload(
  apply: { readonly step: (value: number,) => number; },
): number {
  /**
   * Step lifted out of its holder, so the callee is an identifier.
   */
  const { step, } = apply;
  return step(1,);
}

/**
 * Invokes a supplied step with no overloads at all.
 *
 * The control for `invokedStepOverload`. Same body, same invoked capability, one
 * declaration. Nothing is reported, which locates the disagreement in the comparison
 * rather than in the effect: the invoked capability is recorded either way, and only
 * the overload check treats it as a contract that a signature failed to state.
 *
 * @param apply - Holder whose step is invoked.
 *
 * @returns step result.
 */
export function invokedStepPlain(
  apply: { readonly step: (value: number,) => number; },
): number {
  /**
   * Step lifted out of its holder, so the callee is an identifier.
   */
  const { step, } = apply;
  return step(1,);
}
