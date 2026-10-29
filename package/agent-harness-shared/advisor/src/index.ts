/**
 Host-neutral Advisor core for agent harness adapters.

 The package holds the scoped secondary-review pipeline: configuration,
 effective-scope resolution, model-budgeted serialized context, bounded
 completion recovery, operation accounting, and plain-text summaries. Host
 registration, slash commands, and rich TUI rendering stay in each harness
 adapter. The core reads the host only through three ports declared in
 `ports.ts`: model-resolution, transcript, and theme.

 @module
 */

export * from './constants.ts';
export type * from './types.ts';
export type * from './ports.ts';
export * from './advisor-completion-error.ts';
export * from './advisor-deadline.ts';
export * from './advisor-request.ts';
export * from './advisor-requested-model.ts';
export * from './advisor-client.ts';
export * from './advisor-completion.ts';
export * from './advisor-completion-client.ts';
export * from './config-schemas.ts';
export * from './config.ts';
export * from './context-user.ts';
export * from './context.ts';
export * from './dispatch-scope.ts';
export type * from './operation-types.ts';
export * from './operation-usage.ts';
export * from './operation-clock.ts';
export * from './operation-candidates.ts';
export * from './operation-ledger.ts';
export * from './operation-error.ts';
export * from './operation-attempt.ts';
export * from './operation-progress.ts';
export * from './operation.ts';
export * from './project-context.ts';
export * from './provider-credit.ts';
export * from './default-ranking.ts';
export * from './advisor-selection.ts';
export * from './tool-context-selection.ts';
export * from './run-preparation.ts';
export * from './run-advisor.ts';
export * from './tool-params.ts';
export * from './rendering-summary.ts';
export * from './rendering-operation.ts';
export * from './active-tool.ts';
