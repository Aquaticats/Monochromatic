/**
 Library surface of the guard that keeps one run's `edit` and `write` calls out of the repository's main checkout: the folders it
 holds, the words the driver and Pi exchange about it, the extension Pi loads, and the decision it answers.

 Kept in a file of its own, as the other surfaces are, so no one list outgrows the line budget a file is allowed.

 @module
 */

export {
  armTheMainCheckoutGuard,
  type ArmingOfTheMainCheckoutGuard,
} from './main-checkout-guard-extension.ts';
export {
  armingOfTheMainCheckoutGuard,
  type ArmingOfTheMainCheckoutGuardOfARun,
} from './arming-the-main-checkout-guard-of-a-run.ts';
export {
  isAWorktreesOwnPath,
  type MainCheckoutGuard,
  mainCheckoutGuardOfARun,
  WORKTREES_FOLDER_PARTS,
} from './main-checkout-guard-of-a-run.ts';
export {
  guardArmedDescription,
  MAIN_CHECKOUT_GUARD_COMMAND,
  MAIN_CHECKOUT_GUARD_ENTRY_NAME,
  mainCheckoutGuardArguments,
} from './main-checkout-guard-in-pi.ts';
export {
  MAIN_CHECKOUT_GUARD_OFF,
  MAIN_CHECKOUT_GUARD_VARIABLE,
  MainCheckoutGuardUnreadableError,
  mainCheckoutGuardOfPiEnvironment,
  mainCheckoutGuardWordOf,
} from './main-checkout-guard-words.ts';
export {
  absolutePathOfAPiToolPath,
  type MainCheckoutRefusal,
  NOT_REFUSED,
  refusalOfAnEditOrWrite,
  refusalWords,
} from './refusal-of-a-write-to-the-main-checkout.ts';
export {
  isInside,
  resolveWithSymbolicLinks,
} from './path-with-symbolic-links-resolved.ts';
export {
  type CheckoutAndWorktree,
  checkoutAndWorktreeOf,
} from './checkout-of-a-command.ts';
