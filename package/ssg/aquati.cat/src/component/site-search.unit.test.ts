/**
 Tests for the site-search component styles.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { css as siteSearchStyles, } from './site-search.ts';

/**
 Marker prefixing every emitted outline declaration.
 */
const OUTLINE_MARKER = 'outline-';

await describe({
  name: 'site-search styles',
  children: [
    it({
      name: 'leaves the focus ring to the site-wide :focus-visible rule',
      fn: async function leavesFocusRingToGlobalRule(): Promise<void> {
        /**
         Generated search stylesheet.
         */
        const styles = siteSearchStyles();

        expect(styles,).not.toContain(OUTLINE_MARKER,);
      },
    },),
  ],
},);
