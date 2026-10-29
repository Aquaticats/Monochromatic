/**
 In-memory file registry for browser environments.
 
 Server-side callers (Node/Bun) never need this: the FS module reads
 files from disk. Browser callers populate this Map with CSS file
 contents (keyed by absolute path) before invoking build/mixin functions.
 */
export const fsRegistry: Map<string, string> = new Map<string, string>();
