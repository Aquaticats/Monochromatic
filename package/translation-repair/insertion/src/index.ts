//region Public index
// The names `@monochromatic-dev/translation-repair-insertion` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  bandOf,
  countSettledPerBand,
  MEDIUM_PAGE_BYTES,
  rankWithinBands,
  type SizedEntry,
  SMALL_PAGE_BYTES,
} from './corpus-run/band-order.ts';
export {
  entryInfoFindingsOf,
  EntryInfoParseError,
  type EntryInfoRecord,
  readEntryInfoRecord,
} from './corpus-run/entry-info-record.ts';
export {
  CARRIED_FOLDED_FINDING,
  foldCarriedInsertions,
} from './corpus-run/insertion-carried-fold.ts';
export { CARRIED_SHIFTED_FINDING, } from './corpus-run/insertion-carried-shift.ts';
export {
  admitContainerDeficit,
  CONTAINER_DEFICIT_ADMITTED_FINDING,
  CONTAINER_DEFICIT_UNREAD_FINDING,
  SPLIT_IN_CONTAINER_DEFICIT_FINDING,
} from './corpus-run/insertion-container-deficit.ts';
export {
  admitContainerHalves,
  CONTAINER_HALF_ADMITTED_FINDING,
} from './corpus-run/insertion-container-halves.ts';
export {
  classifyInsertionCoverage,
  type InsertionCandidate,
  type InsertionCoverageRow,
} from './corpus-run/insertion-coverage-model.ts';
export {
  admitReferencedDefinitions,
  DEFINITION_ADMITTED_FINDING,
} from './corpus-run/insertion-referenced-definitions.ts';
export {
  isMarkupOnly,
  markupFraction,
} from './corpus-run/markup-slice.ts';
export { poolGeneration, } from './corpus-run/pool-generation.ts';
export {
  LISTING_TIMEOUT_MS,
  readProviderListing,
} from './corpus-run/provider-listing.ts';
export {
  readRunnerClosure,
  type RunnerClosure,
} from './corpus-run/runner-closure.ts';

//endregion Public index
