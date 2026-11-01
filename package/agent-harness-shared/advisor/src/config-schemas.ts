/**
 Valibot schemas for Advisor extension configuration.
 
 @module
 */

import * as v from 'valibot';
import { MAX_ADVISOR_TIMER_MS, } from './constants.ts';

//region Types

/**
 Raw configuration file shape before defaults are applied.
 */
export type AdvisorConfigFile = {
  /**
   Whether Advisor starts enabled.
   */
  readonly enabled?: boolean;
  /**
   Total Advisor operation deadline in milliseconds.
   */
  readonly timeoutMs?: number;
  /**
   Explicit overlap opt-in, independent of serial failure fallback.
   */
  readonly hedgingEnabled?: boolean;
  /**
   Required launch delay when overlap is enabled.
   */
  readonly hedgeDelayMs?: number;
  /**
   Post-success collection allowance.
   */
  readonly collectionGraceMs?: number;
  /**
   Maximum serialized context characters.
   */
  readonly maxContextChars?: number;
  /**
   Maximum Advisor output tokens.
   */
  readonly maxAdvisorOutputTokens?: number;
  /**
   Whether prior Advisor results stay in context.
   */
  readonly includePriorAdvisorResults?: boolean;
  /**
   Project-specific Advisor prompt suffix.
   */
  readonly systemPrompt?: string;
  /**
   Initial reviewer for default calls: user config only, never a schema default.
   */
  readonly defaultReviewerModel?: string;
};

/**
 Settings file subset needed for scoped-model reconstruction.
 */
export type AdvisorSettingsFile = {
  /**
   Pi model-cycle patterns.
   */
  readonly enabledModels?: readonly string[];
};

//endregion Types

//region Schemas

/**
 String schema used inside settings arrays.
 */
const StringSchema: v.GenericSchema<string> = v.string();

/**
 Positive number schema used for budget and timeout fields.
 */
const PositiveNumberSchema: v.GenericSchema<number> = v.pipe(
  v.number(),
  v.minValue(1,),
);

/**
 Finite integral durations for new scheduling boundaries.
 */
const SchedulingDurationSchema = v.pipe(
  v.number(),
  v.finite(),
  v.integer(),
  v.minValue(1,),
  v.maxValue(MAX_ADVISOR_TIMER_MS,),
);

/**
 Advisor configuration file schema.
 */
export const AdvisorConfigFileSchema: v.GenericSchema<AdvisorConfigFile> = v.object({
  enabled: v.exactOptional(v.boolean(),),
  timeoutMs: v.exactOptional(SchedulingDurationSchema,),
  hedgingEnabled: v.exactOptional(v.boolean(),),
  hedgeDelayMs: v.exactOptional(SchedulingDurationSchema,),
  collectionGraceMs: v.exactOptional(SchedulingDurationSchema,),
  maxContextChars: v.exactOptional(PositiveNumberSchema,),
  maxAdvisorOutputTokens: v.exactOptional(PositiveNumberSchema,),
  includePriorAdvisorResults: v.exactOptional(v.boolean(),),
  systemPrompt: v.exactOptional(v.string(),),
  defaultReviewerModel: v.exactOptional(v.string(),),
},);

/**
 Pi settings subset schema.
 */
export const AdvisorSettingsFileSchema: v.GenericSchema<AdvisorSettingsFile> = v.object({
  enabledModels: v.exactOptional(
    v.array(StringSchema,),
  ),
},);

//endregion Schemas
