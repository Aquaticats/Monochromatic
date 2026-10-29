import type { ReadonlyDeep, } from 'type-fest';
import {
  type Opml,
  parseOpml,
} from 'feedsmith';
import * as v from 'valibot';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { getOPMLTexts, } from './opml-text.ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the outline module.
 */
const l = tagged({
  tag: 'outline',
  l: parentLogger,
},);

/**
 OPML outline with a required, validated `xmlUrl` property.
 Represents a feed subscription entry ready for fetching.
 
 @see {@link Opml} for the base outline type
 */
export type InnerOutlineWUrl = Opml.Outline<string> & { xmlUrl: string; };

//region OPML parsing and outline extraction: Converts raw XML into validated feed outline structures

/**
 Extracts validated inner outlines with xmlUrl from raw OPML source URLs.
 Orchestrates the full pipeline: fetch texts via {@link getOPMLTexts}, parse
 XML through {@link parseSafe}, extract outlines, validate URLs.
 
 @param opmls - Validated OPML source URLs
 
 @returns Array of outlines with validated HTTP(S) xmlUrl properties
 
 @mutates opmls - `getOPMLTexts(opmls)` delegates to `mapIterableAsync`, which may invoke caller-owned iterator capabilities and passes reachable source values to `fetchOpml`.
 
 @example
 ```ts
 const outlines = await getOutlinesFromOpmls(getOpmls());
 ```
 */
export async function getOutlinesFromOpmls(
  opmls: string[],
): Promise<InnerOutlineWUrl[]> {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: getOutlinesFromOpmls.name,
    l,
  },);
  /**
   Raw OPML XML texts pulled before parsing so partial failures do not stall the pipeline.
   */
  const texts = await getOPMLTexts(opmls,);
  /**
   Parsed OPML documents, with unparseable inputs discarded by parseSafe.
   */
  const parsed = parseSafe(texts,);
  /**
   Top-level outline groups extracted from each OPML body.
   */
  const outerOutlines = parsed.flatMap(function extractBody(opml,) {
    return opml.body
      ?.outlines
      ?? [];
  },);
  /**
   Nested feed outlines unwrapped one level so the validate step sees flat entries.
   */
  const innerOutlines = outerOutlines.flatMap(function extractInner(outline,) {
    return outline.outlines
      ?? [];
  },);
  /**
   Outlines whose xmlUrl passes HTTP-domain validation, returned as the function output.
   */
  const result = innerOutlines.filter(
    function hasValidXmlUrl(
      outline,
    ): outline is InnerOutlineWUrl {
      /**
       Destructured xmlUrl so the empty/undefined gate reads on a named binding.
       */
      const { xmlUrl, } = outline;
      if ((xmlUrl === undefined) || (xmlUrl === '')) {
        innerL.warn(`outline ${outline.text
          ?? 'unnamed'} has no xmlUrl`,);
        return false;
      }
      try {
        v.parse(
          v.pipe(
            v.string(),
            v.url(),
            v.check(
              function isHttpDomainUrl(s,) {
                /**
                 Parsed URL so the protocol and hostname can be checked independently.
                 */
                const u = new URL(s,);
                return ((u.protocol
                  === 'http:') || (u.protocol
                    === 'https:'))
                  && v
                    .DOMAIN_REGEX
                    .test(u.hostname,);
              },
              'Invalid HTTP(S) URL with valid domain',
            ),
          ),
          xmlUrl,
        );
        return true;
      }
      catch (error) {
        innerL.warn(`${xmlUrl} failed validation: ${JSON.stringify(error,)}`,);
        return false;
      }
    },
  );
  innerL.debug(`${String(result.length,)} valid inner outlines`,);
  return result;
}

/**
 Safely parses an array of OPML XML strings, discarding unparseable entries.
 
 @param texts - Raw OPML XML strings
 
 @returns Successfully parsed OPML documents
 */
function parseSafe(texts: readonly string[],): ReadonlyDeep<Opml.Document<string>>[] {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: parseSafe.name,
    l,
  },);
  return texts.flatMap(function tryParse(text,) {
    try {
      return [parseOpml(text,),];
    }
    catch (error) {
      innerL.warn(`OPML parse failed: ${JSON.stringify(error,)}`,);
      return [];
    }
  },);
}

//endregion OPML parsing and outline extraction
