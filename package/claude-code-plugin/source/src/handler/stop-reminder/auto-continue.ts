/**
 Unconditional stop-blocking detector.
 
 The hedging, dismissal, and trailing-question detectors all answer "is this
 response defective?". This one answers a different question: "did the agent
 stop while work remained?", and it answers it without consulting the response
 text at all.
 
 The distinction matters because every text-conditioned rule can be satisfied
 by changing the text. A rule keyed on the phrase `Next:` is satisfied by
 deleting that sentence, which converts an informative stop into a silent one.
 This detector cannot be satisfied that way because it reads nothing.
 
 Termination is shared. Claude Code caps consecutive blocks through
 `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`, but only reliably when the agent is idle:
 runs producing no tool calls were overridden after nine blocks, while a run
 working on every continuation reached thirty-one unchecked. This repository
 bounds the busy case in `continuation-depth.ts` and releases the pointless
 cases in `continuation-progress.ts` and `continuation-tasks.ts`. See
 `doc/troubleshooting/claude-code-opus-5-premature-turn-end.md`.

 Off unless {@link AUTO_CONTINUE_ENV} opts in.

 @module
 */

/**
 Opt-in switch. Set to any value in {@link ENABLING_VALUES} to force
 continuation without editing settings or code.
 */
const AUTO_CONTINUE_ENV = 'MONOCHROMATIC_STOP_AUTO_CONTINUE' as const;

/**
 Setting value standing for an unset opt-in switch.
 
 The empty string is the read-side spelling of absence for environment
 variables, so it is the domain's own "not configured" value rather than a
 stand-in for one.
 */
const UNSET_SETTING = '' as const;

/**
 Values that enable forcing continuation.
 
 An unset variable means disabled, so the guard is opt-in. It was opt-out while
 `claude-opus-5` ended turns on announced-but-undone work; `claude-opus-5-5`
 reduced it without removing it, so the reason became a distraction when it fired on every stop;
 the Luna check in `luna-situation.ts` now decides when it applies.
 */
const ENABLING_VALUES: ReadonlySet<string> = new Set([
  '1',
  'on',
  'true',
  'yes',
],);

/**
 Reports whether forced continuation is active, given one already-read setting.
 
 Pure, so tests exercise every branch without touching process state.
 
 @param rawSetting - value read from {@link AUTO_CONTINUE_ENV}, or {@link UNSET_SETTING}
 
 @returns `true` only when `rawSetting` names one of {@link ENABLING_VALUES}
 
 @example
 ```ts
 autoContinueEnabled('on'); // true
 autoContinueEnabled(''); // false
 ```
 */
function autoContinueEnabled(rawSetting: string,): boolean {
  return ENABLING_VALUES.has(
    rawSetting
      .trim()
      .toLowerCase(),
  );
}

/**
 Reads the opt-in switch from this process and reports whether blocking applies.
 
 Separated from {@link autoContinueEnabled} so the decision stays pure and only
 this wrapper touches the environment.
 
 @returns `true` only when the environment enables forced continuation
 
 @example
 ```ts
 if (autoContinueActive()) reasons.push(...autoContinueReason());
 ```
 */
function autoContinueActive(): boolean {
  return autoContinueEnabled(process.env[AUTO_CONTINUE_ENV] ?? UNSET_SETTING,);
}

/**
 Reminder lines emitted when a stop is refused.
 
 Phrased as "you stopped while work remained" rather than "you used a
 forbidden phrase", because naming a phrase teaches suppression of the phrase.
 The instruction to keep status prose exists for the same reason: the failure
 this guard targets is announcing work instead of doing it, and a silent stop
 is strictly worse than an announced one.
 
 The `AskUserQuestion` route matters because it is the only exit that actually
 gets the agent what a stop was reaching for. That tool waits for the user's
 answer, so an agent genuinely blocked on a decision can pause without ending
 its turn, and the hook has nothing to refuse. Stopping, by contrast, ends the
 work and waits to be restarted by hand, which is the cost this whole
 mechanism exists to remove.
 
 @returns reminder lines, joined by callers into one block reason
 
 @example
 ```ts
 autoContinueReason().join(' ');
 ```
 */
function autoContinueReason(): readonly string[] {
  return [
    'You are stopping while tracked work may remain.',
    'Resume the next item now rather than reporting that you will resume it.',
    'If your response named a next action, perform that action in this turn.',
    'Keep writing status and next-step prose exactly as you would have;',
    'do not delete, shorten, or rephrase it to avoid this check,',
    'because a silent stop is worse than an announced one.',
    'If nothing can proceed without a decision from the user,',
    'ask them with the AskUserQuestion tool instead of stopping.',
    'That tool waits for their answer, which is what you actually need;',
    'a stopped turn only ends your work and waits to be restarted.',
    'If the blocker is an external event rather than a decision,',
    'name the concrete blocker and what will clear it.',
  ];
}

export {
  AUTO_CONTINUE_ENV,
  autoContinueActive,
  autoContinueEnabled,
  autoContinueReason,
  ENABLING_VALUES,
  UNSET_SETTING,
};
