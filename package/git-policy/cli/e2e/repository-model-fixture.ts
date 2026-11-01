/**
 Scenario repository types shared by setup and workload modules.

 @module
 */

import type { HookEvent, } from './hook-program-fixture.ts';
import type { ScenarioService, } from './service-fixture.ts';

//region Types

/**
 How the scenario's repository is set up.
 */
export type RepositoryOptions = Readonly<{
  /**
   Whether commands run in the main worktree or a linked worktree.
   */
  worktree: 'linked' | 'main';
  /**
   Hook source:
   config-based hooks (Git 2.54.0 or newer),
   or none.
   Traditional hooks are unsupported by the wrapper,
   so no scenario installs them.
   */
  hooks: 'config' | 'none';
  /**
   Installed hook events.
   */
  hookEvents: readonly HookEvent[];
  /**
   Extra hook behavior.
   */
  hookMode?: 'lint-staged';
  /**
   Whether every commit is SSH-signed.
   */
  signing: boolean;
  /**
   Whether the scenario's message editor appends a trailer that every landed message must carry.
   */
  editorTrailer?: boolean;
  /**
   Whether every landed commit must have exactly one settled push record in the service's work directory.
   */
  afterCommitRecords?: boolean;
  /**
   Files committed and pushed before the scenario starts.
   */
  seedFiles: readonly Readonly<{
    path: string;
    bytes: Buffer
  }>[];
}>;

/**
 Ready scenario repository.
 */
export type ScenarioRepository = Readonly<{
  /**
   Scenario root directory.
   */
  root: string;
  /**
   Directory every scenario command runs in.
   */
  worktree: string;
  /**
   Git common directory.
   */
  commonDir: string;
  /**
   Local bare remote.
   */
  remote: string;
  /**
   Branch checked out in `worktree`.
   */
  branch: string;
  /**
   Absolute real Git executable.
   */
  realGit: string;
  /**
   Hook marker directory.
   */
  markerDir: string;
  /**
   Hook log directory.
   */
  logDir: string;
  /**
   Holding editor program.
   */
  editorProgram: string;
  /**
   Scripted editor program that marks and holds two points of a commit's preparation.
   */
  scriptedEditorProgram: string;
  /**
   The scenario's commit service.
   */
  service: ScenarioService;
  /**
   Environment whose PATH puts the wrapper first.
   */
  wrapperEnv: NodeJS.ProcessEnv;
  /**
   Environment without the wrapper on PATH,
   for processes that bypass cli-git.
   */
  realEnv: NodeJS.ProcessEnv;
  /**
   Options the repository was built with.
   */
  options: RepositoryOptions;
}>;

//endregion Types
