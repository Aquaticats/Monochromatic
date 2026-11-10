/**
 Public surface of the shared fuzz tooling: what a sidecar takes instead of copying it.

 Today that is the run budget (issue 1419). Issue 613 moves the V8 coverage harness of the thirteen `*.fuzz` sidecars
 here as well, generation by generation, so each sidecar keeps only the configuration that is its own: the subject
 package it resolves, its baseline path, its exclusions and its environment variable names.
 */

export * from './run-budget.ts';
