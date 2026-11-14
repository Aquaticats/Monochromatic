/**
 The library surface of the queue of runs waiting for a free place: the queue file and its moves, the shapes an entry takes, the
 set-aside record of an entry whose start failed, the start of one entry, one pass of filling, the places asked for, the filling's
 stall notice, and the reviews the rate limit ended.

 `index.ts` re-exports it, and the commands are `fill-free-places-from-a-queue.ts`, `put-an-entry-into-a-queue.ts`,
 `take-an-entry-out-of-a-queue.ts`, `move-an-entry-of-a-queue.ts`, `list-the-entries-a-queue-set-aside.ts`,
 `put-a-set-aside-entry-back-into-a-queue.ts` and `put-ended-reviews-back-at-the-head-of-a-queue.ts`.

 @module
 */

export {
  type ChangeOfAQueue,
  type QueueRecord,
  QueueFileError,
  changeTheQueueOfRuns,
  moveAQueueEntryTo,
  putAQueueEntryIn,
  readQueueOfRuns,
  takeAQueueEntryOut,
  writeQueueOfRuns,
  writtenQueueOfRuns,
} from './queue-of-runs.ts';
export {
  QUEUE_LOCK_POLL_MILLISECONDS,
  QUEUE_LOCK_STALE_MILLISECONDS,
  QUEUE_LOCK_WAIT_MILLISECONDS,
  QueueLockHeldError,
  aPathWrittenBeside,
  acquireTheQueueLock,
  lockFileOfAQueue,
} from './queue-lock.ts';
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
  newestAttemptOfEachReview,
  putBackSince,
  putEndedReviewsBack,
} from './putting-ended-reviews-back.ts';
export {
  NO_SET_ASIDE_RECORD,
  SET_ASIDE_FILE_SUFFIX,
  type SetAsideRecord,
  SetAsideRecordError,
  isASetAsideRecordFile,
  putASetAsideEntryBack,
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
  QUEUE_FILE_OPTION,
  type QueueOptionWords,
  entryIn,
  nameIn,
  positionIn,
  queueFileIn,
  queueRefusalOf,
  reviewsFolderIn,
  runsFolderIn,
  wholeNumberFromOne,
} from './parse-queue-of-runs-options.ts';
