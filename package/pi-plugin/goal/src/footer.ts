/**
 Goal footer preview formatting with grapheme safety.
 
 @module
 */

import {
  MAX_FOOTER_OBJECTIVE_GRAPHEMES,
  TRUNCATED_FOOTER_OBJECTIVE_GRAPHEMES,
} from './constants.ts';

/**
 Locale-neutral grapheme segmenter reused for bounded footer previews.
 */
const GRAPHEME_SEGMENTER = new Intl.Segmenter(
  undefined,
  {
    granularity: 'grapheme',
  },
);

/**
 Format objective preview with at most ten displayed graphemes.
 
 @param objective - exact normalized objective
 
 @returns full objective or first nine graphemes plus ellipsis
 
 @example
 ```ts
 objectivePreview('abcdefghijk');
 ```
 */
function objectivePreview(objective: string,): string {
  /**
   Display graphemes preserving emoji and combining sequences.
   */
  const graphemes = Array.from(
    GRAPHEME_SEGMENTER.segment(objective,),
    function segmentText(segment: Readonly<Intl.SegmentData>,) {
      return segment.segment;
    },
  );
  if (graphemes.length <= MAX_FOOTER_OBJECTIVE_GRAPHEMES)
    return objective;
  return `${graphemes.slice(
    0,
    TRUNCATED_FOOTER_OBJECTIVE_GRAPHEMES,
  )
    .join('')}…`;
}

/**
 Build live active-goal footer text.
 
 @param objective - exact normalized objective
 
 @returns footer status text
 
 @example
 ```ts
 formatGoalFooter('ship feature');
 ```
 */
function formatGoalFooter(objective: string,): string {
  return `goal ${objectivePreview(objective,)}`;
}

export {
  formatGoalFooter,
  objectivePreview,
};
