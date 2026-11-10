/**
 The source set whose content decides every cached post's rendered HTML.
 
 The build stores rendered MDX HTML keyed by the post's own content hash plus
 one pipeline fingerprint, so a pipeline file that changes without the
 fingerprint changing leaves stale HTML in the cache. This module owns the
 glob that names those files, and the build reads it from here.
 
 The set is deliberately wider than what feeds a post body today: `client` and
 `i18n` sources do not change rendered post HTML, and matching them only costs
 a cache miss, while missing a source the renderer reads costs a wrong page.
 
 @module
 */

/**
 Glob matching every pipeline source file that a change to invalidates the
 whole rendered-post cache.
 
 Directory names are relative to the package root, which is the build's
 working directory. `src/component` is the singular directory the MDX
 renderer imports through `renderMdx`'s `useMDXComponents` scope; typing it as
 `components` matched no file at all.
 */
export const PIPELINE_GLOB = 'src/{lib,component,client,i18n}/**/*.ts';
