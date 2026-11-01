//region Public index
// The names `@monochromatic-dev/translation-repair-reference` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  citedReferenceBlock,
  lookupCitedReference,
  referenceLineOf,
} from './cited-reference-lookup.ts';
export {
  citedReferenceUrlsOf,
  MAX_CITED_REFERENCES,
} from './cited-reference-scan.ts';
export {
  type CachedLookup,
  CacheFileUnreadableError,
  isLookupHit,
  isLookupRecord,
  LOOKUP_CACHE_DIR_VAR,
  lookupCacheDir,
  lookupCachePath,
  type LookupHit,
  type LookupRecord,
  packageCacheDir,
  readCachedLookup,
  writeCachedLookup,
} from './lookup-cache.ts';
export {
  screenAttestedAdditions,
  type ScreenedClaims,
} from './reference-attest-claims.ts';
export {
  buildReferenceAttestConfirmMessages,
  type ConfirmationBallot,
  confirmedDetails,
  isReferenceAttestConfirmWire,
  REFERENCE_ATTEST_CONFIRM_RESPONSE_FORMAT,
  type ReferenceAttestConfirmWire,
} from './reference-attest-confirm-wire.ts';
export {
  type AttestedDetail,
  attestedDetailLines,
  attestedDetailsOverlapping,
  compacted,
  mergedAttestations,
  quoteIsIn,
  type VerifiedAttestation,
} from './reference-attest-match.ts';
export {
  attestCitedReferences,
  type ReferenceAttestation,
} from './reference-attest-stage.ts';
export {
  type AttestationVerdict,
  attestationVerdictLine,
  attestationVerdicts,
  keptAttestations,
} from './reference-attest-verdict.ts';
export {
  type AttestationItemWire,
  buildReferenceAttestMessages,
  isReferenceAttestWire,
  REFERENCE_ATTEST_RESPONSE_FORMAT,
  type ReferenceAttestWire,
} from './reference-attest-wire.ts';
export {
  type CachedReference,
  isReferenceRecord,
  readCachedReference,
  REFERENCE_CACHE_SUBDIR,
  referenceCacheDir,
  referenceCachePath,
  type ReferenceRecord,
  writeCachedReference,
} from './reference-cache.ts';
export {
  type NumberedReferenceLine,
  numberedReferenceLines,
  REFERENCE_UNFETCHED,
  REFERENCE_UNREADABLE,
  referenceLineHead,
  ReferenceLineHeadError,
  referenceLinePageText,
  referencePageTexts,
} from './reference-line-head.ts';
export { nextSchemeStart, } from './scheme-start-scan.ts';
export {
  lookupLinesOf,
  lookupWorkTitle,
  workTitleLookupLines,
} from './work-title-lookup.ts';
export {
  lookupQueryFor,
  workTitlesOf,
} from './work-title-scan.ts';
export {
  EXA_API_KEY_VAR,
  EXA_SEARCH_URL,
  HIGHLIGHT_CHARACTERS,
  hitsOf,
  RESULTS_PER_TITLE,
  searchWorkTitle,
  WorkTitleLookupError,
} from './work-title-search.ts';

//endregion Public index
