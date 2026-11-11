/**
 Measured listing membership fixtures shared by this package's unit suites:
 the serviceable and media id sets the 2026-11-11 endpoint measurement
 recorded. Test-only data, kept beside the source so both the mapping and the
 settings-seam suites pin the same goldens. @module
 */

//region Public API

/**
 Every serviceable id the 2026-11-11 measured listing carried: the thirteen
 chat ids chat probes answered plus the `auto` router alias. Sorted here so
 membership comparisons need no endpoint order. Recorded with the media
 classification in
 `doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md`.
 */
export const MEASURED_SERVICEABLE_IDS = [
  'auto',
  'deepseek-v4-flash-0731',
  'deepseek-v4-pro',
  'deepseek-v4-pro-0813',
  'deepseek-v4.1-flash',
  'deepseek-v4.1-flash-1017',
  'glm-5.2',
  'glm-5.3',
  'qwen3.6-flash',
  'qwen3.7-max',
  'qwen3.7-plus',
  'qwen3.8-flash',
  'qwen3.8-max',
  'qwen4-max-preview',
] as const;

/**
 The seven ids the same measured listing carried whose chat probes were
 refused (image, video, tts, asr, and realtime models). None may be served.
 */
export const MEASURED_MEDIA_IDS = [
  'qwen-audio-3.0-asr-flash',
  'qwen-audio-3.0-realtime-plus',
  'qwen-audio-3.0-tts-plus',
  'qwen-image-3.0-pro',
  'wan2.7-image',
  'wan2.7-image-pro',
  'wan3.0-video',
] as const;

//endregion Public API
