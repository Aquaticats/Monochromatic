/**
 The library surface of the queue of runs waiting for a free place: the queue folder and its moves, the order each entry carries, the
 shapes an entry takes, the claim one filling holds on an entry, the set-aside record of an entry whose start failed, the start of one
 entry, one pass of filling, the places asked for, the filling's stall notice, the reviews the rate limit ended, and the changeover from
 the queue this package kept as one file.

 `index.ts` re-exports it, and the commands are `fill-free-places-from-a-queue.ts`, `put-an-entry-into-a-queue.ts`,
 `take-an-entry-out-of-a-queue.ts`, `move-an-entry-of-a-queue.ts`, `list-the-queue-of-runs.ts`, `list-the-entries-a-queue-set-aside.ts`,
 `put-a-set-aside-entry-back-into-a-queue.ts`, `put-ended-reviews-back-at-the-head-of-a-queue.ts` and
 `convert-a-queue-file-into-a-queue-folder.ts`.

 @module
 */

export {
  type AClaimOnAnEntry,
  type QueueClaim,
  type QueueRecord,
  BEING_TAKEN_START,
  BEING_WRITTEN_START,
  CLAIMED_FOLDER,
  NO_QUEUE_ID,
  ORDER_FIELD,
  QUEUE_BOOKKEEPING_FIELDS,
  QUEUE_ID_FIELD,
  QueueFileError,
  aFreshQueueId,
  anEntryFileName,
  anEntryNameInAFileName,
  claimFileOf,
  claimedFolderOf,
  entryFileOf,
  isQueueRecord,
  orderOf,
  queueIdOf,
} from './queue-of-runs.ts';
export {
  NOT_AN_ORDER,
  anOrderBetween,
  anOrderIn,
  isAnOrder,
  ordersOfAPlace,
} from './order-of-an-entry-in-a-queue.ts';
export {
  LONGEST_ESCAPED_NAME,
  NO_NAME_IN_AN_ESCAPED_FORM,
  QueueNameError,
  aNameInAnEscapedName,
  aUnitNameOf,
  anEscapedName,
  anEntryNameInAUnitName,
} from './names-a-queue-gives-its-things.ts';
export {
  aClaimOf,
  aNameTakenAside,
  aPlaceFor,
} from './places-and-names-of-a-queue-folder.ts';
export {
  aClaimIn,
  readTheClaimsOnAQueue,
} from './reading-the-claims-beside-a-queue.ts';
export {
  readTheQueueReportingProblems,
  readTheQueueThatMayNotBeThere,
} from './reading-a-queue-a-kill-left.ts';
export { namesInAFolderThatMayNotBeThere, } from './reading-a-folder-for-tests.ts';
export {
  NO_ENTRY_OF_THAT_NAME,
  aFileLaidDown,
  readQueueOfRuns,
  readTheEntryAt,
  readTheEntryNamed,
  writeAnEntryOfAQueue,
  writeTheQueueOfRuns,
} from './reading-and-writing-the-folder-a-queue-is.ts';
export {
  type ClaimedAnEntry,
  claimAnEntryOfAQueue,
  completeTheClaim,
  giveBackTheClaim,
  namesOfAnAttemptIn,
  startOfAnAttemptTookEffect,
  theClaimOn,
} from './claiming-an-entry-of-a-queue.ts';
export {
  AnotherFillingHoldsTheEntryError,
  discardTheEntryTaken,
  moveAQueueEntryTo,
  putAQueueEntryIn,
  takeAQueueEntryOut,
} from './putting-and-taking-entries-of-a-queue.ts';
export {
  aClaimGivenBackOnlyIfOurs,
  aClaimRemovedOnlyIfItSays,
  anEntryRemovedOnlyIfTheAttempt,
  putBackUnlessTheNameIsTaken,
  theIdentityQueued,
} from './a-claim-taken-and-given-back.ts';
export {
  type HolderOfAClaim,
  processIsThere,
  startedAtOf,
  theHolderOfAClaim,
} from './where-a-claims-holder-is-now.ts';
export {
  HALF_WRITTEN_FILE_AGE_MILLISECONDS,
  halfWrittenFilesSwept,
  settleTheQueueAfterAKill,
} from './settling-the-queue-after-a-kill.ts';
export {
  BEING_CARRIED_ENDING,
  CARRIED_ENDING,
  CONVERTED_QUEUE_ENDING,
  type ConvertedAQueue,
  convertAQueueFileIntoAFolder,
  entriesOfAQueueFile,
} from './converting-a-queue-file-into-a-folder.ts';
export {
  REVIEW_OF_A_RANGE_ENTRY_FIELDS,
  type ReviewOfARangeEntry,
  type ShapeOfAQueueEntry,
  changeOfAReviewRun,
  poolAPlaceAskedForWouldHold,
  poolOfAQueueEntry,
  resumesAQueueEntry,
  shapeOfAQueueEntry,
} from './entry-of-a-queue.ts';
export {
  RATE_LIMIT_REVIEW_PROMPT,
  RATE_LIMIT_REVIEW_PROMPT_FILE_NAME,
  REVIEWS_PUT_BACK_LOOKBACK_SECONDS,
  type EndedReviewsPutBack,
  rateLimitReviewPromptFileIn,
  putBackSince,
  putEndedReviewsBack,
} from './putting-ended-reviews-back.ts';
export {
  putASetAsideEntryBack,
} from './putting-a-set-aside-entry-back.ts';
export {
  NO_SET_ASIDE_RECORD,
  SET_ASIDE_FILE_SUFFIX,
  type SetAsideRecord,
  SetAsideRecordError,
  isASetAsideRecordFile,
  removeSetAsideRecord,
  setAsideAnEntryOfAQueue,
  setAsideFileOf,
  setAsideRecordIn,
  setAsideRecordIsThere,
  setAsideRecordOf,
  setAsideRecordsIn,
  writeSetAsideRecord,
} from './set-aside-entry.ts';
export {
  type FailureOfAStart,
  type KindOfAStartFailure,
  type StartOfAnEntry,
  type StartOfAnEntryRequest,
  type StartOfAnEntrySeams,
  gitRefusalIsTheMachines,
  reviewRunEntryOf,
  startAnEntryOfAQueue,
} from './starting-an-entry-of-a-queue.ts';
export {
  attemptsOfEachReview,
  newestAttemptOfEachReview,
} from './attempts-of-a-review-run.ts';
export {
  type BatchUnitStart,
  type BatchUnitStartRequest,
  batchUnitRequestOf,
  startABatchFile,
  startABatchInItsOwnUnit,
} from './starting-a-batch-in-its-own-unit.ts';
export {
  FILLING_STAGGER_MILLISECONDS,
  FILLING_STARTS_A_ROUND,
  type FillingOutcome,
  type FillingSeams,
  fillFreePlacesFromAQueue,
  mayStartNow,
} from './filling-free-places-from-a-queue.ts';
export {
  batchListOfAUnit,
  placesAskedFor,
} from './asked-for-places.ts';
export {
  FILLING_ROUND_SECONDS,
  STALLED_FILLING_BOUND_MILLISECONDS,
  STALLED_FILLING_BOUND_SECONDS,
  type StalledFillingLook,
  noticeIfTheFillingStalls,
  stalledFillingNoticeOf,
} from './stalled-filling-notice.ts';
export {
  DESKTOP_NOTICE_APP_NAME,
  DESKTOP_NOTICE_TIMEOUT_MILLISECONDS,
  type DesktopNotice,
  type DesktopNoticeOutcome,
  noticeOnTheDesktop,
} from './desktop-notice.ts';
export {
  DEFAULT_FILLING_SECONDS,
  FILLING_USAGE,
  type FillingOptions,
  FillingOptionsError,
  parseFillingOptions,
} from './parse-filling-options.ts';
export {
  ENTRY_OPTION,
  POSITION_OPTION,
  QUEUE_OPTION,
  type QueueOptionWords,
  entryIn,
  nameIn,
  positionIn,
  queueFolderIn,
  queueRefusalOf,
  reviewsFolderIn,
  runsFolderIn,
  wholeNumberFromOne,
} from './parse-queue-of-runs-options.ts';
