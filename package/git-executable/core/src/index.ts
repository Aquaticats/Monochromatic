/**
 Real-Git executable resolution for infrastructure that must bypass workspace policy wrapper.
 */

export { RealGitNotFoundError, } from './error.ts';
export { resolveRealGit, } from './resolve-real-git.ts';
export type { ResolveRealGitOptions, } from './types.ts';
