import type {
  RegionDefectTally,
  ScreenedDefectClaim,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { IssueProbeReading, } from '@monochromatic-dev/translation-repair-assembly/ts';

//region Probe attribution
// The shape a probe reading has once it has been READ BACK from an artifact,
// which is deliberately smaller than the shape that was written.
//
// A screened claim carries who said it, what the deterministic check made of
// it, and the wording it quoted. The quotes are UNLICENSED corpus text. The
// reader that feeds `score-probe` exists to produce counts that are safe to
// paste into a verdict or a message, so it parses attribution and drops every
// quote field.
//
// Before these types existed the reader satisfied `ScreenedDefectClaim` by
// filling `evidence`, `omittedText`, `reason`, `category` and `severity` with
// the empty string. That is a claim shaped exactly like a complete one, and
// nothing in the type said otherwise: a caller reading `claim.evidence` got
// `''` and had no way to tell "this reader does not parse quotes" from "the
// prober quoted nothing". The first reading is always right and the second is
// impossible, since the screen cannot admit a claim with no anchor as
// corroborated.
//
// So the reduced shape is named instead of faked. A full claim remains
// assignable to the reduced one, which is what lets the same telemetry
// functions summarize readings straight from the pipeline and readings lifted
// off disk.

/**
 Who filed a screened claim and what the differential check made of it.

 The two fields the majority rule needs, and the only two that carry no
 corpus text.

 @example
 ```ts
 const attribution: ProbeClaimAttribution = { modelId: 'hf:vendor/model', admissibility: 'corroborated', };
 ```
 */
export type ProbeClaimAttribution = Pick<
  ScreenedDefectClaim,
  'modelId' | 'admissibility'
  >;

/**
 A region tally whose claims carry attribution only.

 Every count is unchanged, because counts are what the summary reports and
 they are derived by the screen from the same claims. Only the claim list is
 narrowed.

 @example
 ```ts
 const tally: TelemetryRegionTally = { envelopeId: 'envelope/1', claims: [], corroborated: 0, ... };
 ```
 */
export type TelemetryRegionTally =
  & Omit<RegionDefectTally, 'claims'>
    & {
    /**
     Screened claims of this region, attribution only.
     */
      readonly claims: readonly ProbeClaimAttribution[];
    };

/**
 A probe reading whose region claims carry attribution only.

 @example
 ```ts
 const reading: TelemetryProbeReading = { heardProbers: 3, configuredProbers: 3, regions: [], };
 ```
 */
export type TelemetryProbeReading =
  & Omit<IssueProbeReading, 'regions'>
    & {
    /**
     Region tallies as the reader lifts them.
     */
      readonly regions: readonly TelemetryRegionTally[];
    };

//endregion Probe attribution
