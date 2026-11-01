/**
 Loaded Pi project-context capture for Advisor requests.
 
 @module
 */

/**
 Context file loaded into Pi's main-agent system prompt.
 
 @example
 ```typescript
 const contextFile: AdvisorProjectContextFile = {
   path: '/project/AGENTS.md',
   content: 'Use mise.',
 };
 ```
 */
export type AdvisorProjectContextFile = {
  /**
   Absolute context-file path reported by Pi.
   */
  readonly path: string;
  /**
   Complete loaded context-file content.
   */
  readonly content: string;
};

/**
 Session-local project-context snapshot handle.
 */
export type AdvisorProjectContextState = {
  /**
   Read canonical active project-context JSON.
   */
  readonly get: () => string;
  /**
   Replace active snapshot from authoritative Pi prompt options.
   */
  readonly replace: (contextFiles: readonly AdvisorProjectContextFile[]) => void;
  /**
   Clear active snapshot at session boundary.
   */
  readonly clear: () => void;
};

/**
 Serialize owned context-file fields as JSON in Pi load order.
 
 JSON encoding keeps context-file delimiters and instruction-like text inside
 one explicit data grammar before insertion into Advisor system prompt.
 
 @param contextFiles - Pi-loaded context files
 
 @returns canonical request data, or empty string when no files loaded
 
 @example
 ```typescript
 serializeAdvisorProjectContext([
   { path: '/project/AGENTS.md', content: 'Use mise.' },
 ]);
 ```
 */
export function serializeAdvisorProjectContext(
  contextFiles: readonly AdvisorProjectContextFile[],
): string {
  /**
   Isolated path/content records preserving Pi load order.
   */
  const isolatedContextFiles = contextFiles.map(function isolateContextFile(
    contextFile,
  ): AdvisorProjectContextFile {
    return {
      content: contextFile.content,
      path: contextFile.path,
    };
  },);
  return isolatedContextFiles.length === 0
    ? ''
    : JSON.stringify(isolatedContextFiles,);
}

/**
 Create session-local project-context snapshot state.
 
 @returns readonly state operations over closure-private snapshot
 
 @example
 ```typescript
 const state = createAdvisorProjectContextState();
 state.replace([{ path: '/project/AGENTS.md', content: 'Use mise.' }]);
 ```
 */
export function createAdvisorProjectContextState(): AdvisorProjectContextState {
  /**
   Canonical current project-context snapshot cell.
   */
  const projectContext = { value: '', };
  return {
    get: function get() {
      return projectContext.value;
    },
    replace: function replace(
      contextFiles: readonly AdvisorProjectContextFile[],
    ) {
      projectContext.value = serializeAdvisorProjectContext(contextFiles,);
    },
    clear: function clear() {
      projectContext.value = '';
    },
  };
}
