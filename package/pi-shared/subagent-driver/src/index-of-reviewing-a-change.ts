/**
 The library surface of the review commands: who reviews a change and what a review of one is called, what a review reads, the
 briefs a review and its fix are given, and the starter that starts one review per finished branch. `index.ts` re-exports it.

 @module
 */

export {
  type GitQuestion,
  ReviewGitError,
  type ReviewMaterial,
  type ReviewReading,
  ReviewReadingError,
  codeFilesAmong,
  materialOfAReview,
  outputOfGit,
  rangeOfABranch,
  readingIn,
} from './material-of-a-review.ts';
export {
  CHANGE_DIFF_FILE_NAME,
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  DEFAULT_RUNS_FOLDER_IN_HOME,
  REPRODUCTIONS_FOLDER_NAME,
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  REVIEW_BRANCH_PREFIX,
  REVIEW_BRIEF_FILE_NAME,
  REVIEW_FILE_NAME,
  REVIEW_READING_FILE_NAME,
  REVIEW_RUN_PREFIX,
  branchOfAReview,
  briefFileOfAReview,
  changeNameOfBranch,
  diffFileOfAReview,
  reproductionsFolderOfAReview,
  readingFileOfAReview,
  reviewFileOfAReview,
  reviewFolderOf,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
export {
  TemplatePlaceholderError,
  filledTemplate,
  placeholdersIn,
} from './template-with-placeholders.ts';
export {
  NO_SYNTHETIC_HOST_LOAD,
  RULES_FILE_IN_THE_CHECKOUT,
  rulesFileIn,
} from './rules-every-pi-brief-carries.ts';
export {
  FOCUS_OF_A_CHANGE_REVIEW,
  REVIEW_BRIEF_PLACEHOLDERS,
  briefOfAReview,
  placeholdersInTheReviewBrief,
  reviewBriefValues,
  subjectOfAReview,
} from './review-brief.ts';
export {
  FIX_BRIEF_PLACEHOLDERS,
  FIX_BRIEF_WALL_SECONDS,
  FIX_RUN_PREFIX,
  branchOfAFixFor,
  briefOfAFix,
  fixBriefValues,
  fixRunNameOf,
  placeholdersInTheFixBrief,
} from './fix-brief.ts';
export {
  LONGEST_MEASURED_REVIEW_TURNS,
  REVIEW_START_DEADLINE_SECONDS,
  REVIEW_TURN_CAP,
  REVIEW_WALL_SECONDS,
  type BriefOfOneReview,
  type ReviewStartingSeams,
  type StartedReviews,
  type StartOfOneReview,
  retryWordsOfAReview,
  startReviews,
} from './reviews-of-finished-branches.ts';
export { writeReviewBrief, } from './writing-a-review-brief.ts';
export {
  keepFailedOutput,
  reportOnStandardError,
} from './kept-start-failure.ts';
export {
  FIX_BRIEFS_USAGE,
  FixBriefOptionsError,
  type FixBriefOptions,
  parseFixBriefOptions,
} from './parse-fix-brief-options.ts';
export {
  REVIEW_BRIEF_USAGE,
  ReviewBriefOptionsError,
  type ReviewBriefOptions,
  parseReviewBriefOptions,
} from './parse-review-brief-options.ts';
export {
  START_REVIEWS_USAGE,
  type StartReviewsOptions,
  StartReviewsOptionsError,
  parseStartReviewsOptions,
} from './parse-start-reviews-options.ts';
export {
  retryingStartEntry,
  startThroughTheRetryingStart,
} from './start-through-the-retrying-start.ts';
