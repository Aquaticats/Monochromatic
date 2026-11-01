/**
 Options that make a commit open the suite's scripted editor.

 The wrapper prepares a commit that asks for an editor by running real Git in a scratch repository, and Git runs the
 editor there, before anything is sent to the service. The editor is therefore the suite's instrument for holding or
 killing a commit during preparation (`hook-program-fixture.ts`, `SCRIPTED_EDITOR_PROGRAM`).

 @module
 */

import type { ScenarioRepository, } from './repository-fixture.ts';

/**
 Arguments and environment of a commit that opens the scripted editor.

 @param repository - scenario repository

 @param trailer - whether the editor appends its trailer to the message

 @returns the `commit` option and the editor environment for `startAttempt`

 @example
 ```ts
 await startAttempt({ ...context, label: 'victim', paths, mode: 'explicit', ...editorOptions({ repository }) });
 ```
 */
export function editorOptions({
  repository,
  trailer = false,
}: Readonly<{
  repository: ScenarioRepository;
  trailer?: boolean;
}>,): Readonly<{
  extraArgs: readonly string[];
  env: Readonly<Record<string, string>>;
}> {
  return {
    extraArgs: ['-e',],
    env: {
      GIT_EDITOR: repository.scriptedEditorProgram,
      ...(trailer ? { E2E_EDITOR_TRAILER: '1', } : {}),
    },
  };
}
