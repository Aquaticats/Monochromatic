/**
 Virtual filesystem the native TypeScript process reads current sources through.
 
 @module
 */

import {
  serverFS,
  type FileSystemCallbacks,
} from 'typescript/fs';

import { normalizeSemanticFileName, } from './semantic-file-name.ts';

/**
 TypeScript filesystem callbacks answering from kept overlay text.
 
 Every operation the overlay cannot answer delegates to the server's operating-system filesystem through
 `serverFS.useOS`, the 7.1 spelling of the 7.0 `undefined` fallback. `readFile` returning `undefined` now means
 "absent", so a missed overlay lookup must send the delegation sentinel instead of `undefined`.
 */
export type SemanticOverlayFileSystem = FileSystemCallbacks;

/**
 Builds filesystem callbacks answering from overlay text, delegating everything else.
 
 Callbacks read through the live map rather than a copy, so text kept after this call is text the
 native process can still read. Paths arrive spelled however TypeScript spells them, so each is
 put into bridge source identity before lookup.
 
 @param overlays - Live overlay text keyed by normalized source path.
 
 @returns callbacks for overlay reads and presence, each falling back to the real filesystem.
 
 @example
 ```ts
 new API({ cwd: process.cwd(), fs: overlayFileSystem({ overlays },), });
 ```
 */
export function overlayFileSystem({
  overlays,
}: {
  readonly overlays: ReadonlyMap<string, string>;
},): SemanticOverlayFileSystem {
  return {
    /**
     Delegates directory listing wholesale; this overlay only ever holds file text.
     */
    directoryExists: serverFS.useOS,
    /**
     Delegates directory enumeration wholesale; this overlay only ever holds file text.
     */
    getAccessibleEntries: serverFS.useOS,
    /**
     Delegates path resolution wholesale; overlay paths are already canonical bridge identity.
     */
    realpath: serverFS.useOS,
    /**
     Delegates metadata wholesale; the bridge reads text through `readFile`, never stats.
     */
    stat: serverFS.useOS,
    /**
     Delegates writes wholesale; bridge overlay text is managed through the map, and 7.0 wrote through to the
     operating system the same way.
     */
    writeFile: serverFS.useOS,
    /**
     Delegates removals wholesale for the same reason writes do.
     */
    removeFile: serverFS.useOS,
    /**
     Reads virtual current-file content or delegates to TypeScript real filesystem.
     
     @param fileName - Path requested by native TypeScript process.
     
     @returns overlay content (including empty string) or real-filesystem delegation sentinel.
     */
    readFile: function readFileFromOverlayOrDelegate(
      fileName: string,
    ): string | typeof serverFS.useOS {
      /**
       Overlay text for the path, absent when the overlay does not hold it.
       */
      const overlayText = overlays.get(normalizeSemanticFileName(fileName,),);
      return overlayText ?? serverFS.useOS;
    },
    /**
     Reports positive overlay presence or delegates unknown paths to real filesystem.
     
     @param fileName - Path requested by native TypeScript process.
     
     @returns true for overlay file or real-filesystem delegation sentinel.
     */
    fileExists: function reportOverlayPresenceOrDelegate(
      fileName: string,
    ): true | typeof serverFS.useOS {
      return overlays.has(normalizeSemanticFileName(fileName,),) ? true : serverFS.useOS;
    },
  };
}
