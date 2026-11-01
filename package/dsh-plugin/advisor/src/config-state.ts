/**
 Unified config loading and per-session Advisor state for the DSH plugin. @module
 */

import {
  loadMergedConfig,
  type AdvisorConfig,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

//region Types

/**
 Mutable per-session Advisor state shared by the tool and the command.

 The session toggle mirrors pi's `/advisor on` and `/advisor off`;
 transcript-degradation text is recorded here so `/advisor status` can report
 a Q4 fallback after a failed derived-history acquisition.
 */
export type DshAdvisorSessionState = {
  /**
   Whether Advisor calls may run in this session.
   */
  readonly getEnabled: () => boolean;
  /**
   Set the session toggle.
   */
  readonly setEnabled: (enabled: boolean) => void;
  /**
   Record why the derived history was unavailable, or clear with an empty
   string after a healthy acquisition.
   */
  readonly recordTranscriptDegradation: (reason: string) => void;
  /**
   Recorded degradation reason; empty string means the last acquisition was
   healthy.
   */
  readonly getTranscriptDegradationReason: () => string;
  /**
   Adopt the disposer that lifts a `/advisor off` tool restriction, disposing
   any previously adopted restriction first.
   */
  readonly adoptToolRestriction: (dispose: () => void) => void;
  /**
   Dispose any adopted tool restriction, restoring the tool.
   */
  readonly clearToolRestriction: () => void;
};

//endregion Types

//region Public API

/**
 Load the unified Advisor config from `pi-advisor.json` files.

 Global `~/.pi/agent/extensions/pi-advisor.json` merges under project
 `.pi/extensions/pi-advisor.json`; behavioral settings live only in these
 files, never in the Schemastery plugin row.

 @param options - lookup directories for global and project config

 @returns merged runtime configuration

 @throws when a present config file is invalid or violates the schema

 @example
 ```ts
 const config = await loadDshAdvisorConfig({ cwd: process.cwd() });
 ```
 */
export async function loadDshAdvisorConfig(
  options: {
    readonly cwd: string;
    readonly home?: string;
  },
): Promise<AdvisorConfig> {
  return await loadMergedConfig(options,);
}

/**
 Create one session's Advisor toggle and degradation state.

 @param initialEnabled - initial toggle value, defaulting to the config
 `enabled`

 @returns mutable session state handle

 @example
 ```ts
 const state = createDshAdvisorSessionState({ initialEnabled: config.enabled });
 ```
 */
export function createDshAdvisorSessionState(
  {
    initialEnabled,
  }: {
    readonly initialEnabled: boolean;
  },
): DshAdvisorSessionState {
  /**
   Shared mutable state behind the immutable handle.
   */
  const state = {
    enabled: initialEnabled,
    transcriptDegradationReason: '',
  };
  /**
   Disposers of adopted tool restrictions, drained on every replace or clear.
   */
  const restrictionDisposers: (() => void)[] = [];
  return {
    getEnabled: function getEnabled(): boolean {
      return state.enabled;
    },
    setEnabled: function setEnabled(enabled: boolean): void {
      state.enabled = enabled;
    },
    recordTranscriptDegradation: function recordTranscriptDegradation(
      reason: string,
    ): void {
      state.transcriptDegradationReason = reason;
    },
    getTranscriptDegradationReason: function getTranscriptDegradationReason(): string {
      return state.transcriptDegradationReason;
    },
    adoptToolRestriction: function adoptToolRestriction(dispose: () => void): void {
      for (const pending of restrictionDisposers.splice(0,))
        pending();
      restrictionDisposers.push(dispose,);
    },
    clearToolRestriction: function clearToolRestriction(): void {
      for (const pending of restrictionDisposers.splice(0,))
        pending();
    },
  };
}

//endregion Public API
