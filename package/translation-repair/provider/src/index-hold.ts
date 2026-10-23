//region Hold public index
// The names the router's holds give its siblings and its own tests, each from
// the module that declares it; split out of `index.ts` when the credential
// holds put that file over the line budget. It also carries the reason and the log line of a model that lost every
// provider, which the same router raises beside them.

export {
  callHoldingRefusedCredential,
  type CredentialHolds,
  createCredentialHolds,
  credentialRefusalLine,
  foldCredentialHolds,
  holdOnCredentialRefusal,
  reachPastCredentialHolds,
} from './credential-hold.ts';
export {
  EVERY_PROVIDER_COUNTS,
  everyWritingProviderDry,
  idleWetProviders,
  type WritingReach,
} from './budget-routing.ts';
export { durationWordsOf, } from './duration-words.ts';
export {
  type BoundOwner,
  pastPassWordsOf,
  returnFitsWithin,
} from './neuralwatt-return-fit.ts';
export {
  keyUnblockTextOf,
  namedDrynessOf,
  openingClauseOf,
  returnClauseOf,
  returnTimeOf,
} from './neuralwatt-return-words.ts';
export {
  dryCauseClausesOf,
  holdReasonOf,
} from './hold-reason.ts';
export {
  modelDarkReason,
  refuseDarkModel,
} from './model-dark-reason.ts';
export { isCredentialRefusal, } from './provider-budget-refusal.ts';
export { reachPastHoldsOrRefusal, } from './reach-past-holds.ts';

//endregion Hold public index
