/**
 Tests for the HTML control-panel renderer.
 
 Asserts structural properties (counts of repeated elements,
 presence of required IDs, attribute pass-through, search-input
 escape semantics) rather than a full-string snapshot, so trivial
 formatting tweaks don't break the test.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PackageProbe, } from './probe.ts';
import { renderControls, } from './render-controls.ts';
import { defaultState, } from './script/state.ts';

/**
 Minimal two-probe fixture; richness isn't required for structural tests.
 */
const PROBES: readonly PackageProbe[] = [
  {
    catalogKey: 'preact',
    npmName: 'preact',
    resolvedVersion: '10.26.0',
    isLeaf: true,
    weeklyDownloads: 5_000_000,
    installSizeBytes: 250_000,
    packageAgeDays: 1_500,
    licenseClass: 'permissive',
    runtimeDepCount: 0,
    transitiveDepCount: 0,
    tsRatioOrNull: 0.99,
    sourceBytesOrNull: 200_000,
    daysSinceLastCommitOrNull: 14,
    repositoryUrlOrNull: 'https://github.com/preactjs/preact',
    isMonorepoHoused: false,  },
  {
    catalogKey: 'ms',
    npmName: 'ms',
    resolvedVersion: '2.1.3',
    isLeaf: false,
    weeklyDownloads: 100_000,
    installSizeBytes: 5_000,
    packageAgeDays: 4_000,
    licenseClass: 'copyleft',
    runtimeDepCount: 2,
    transitiveDepCount: 5,
    tsRatioOrNull: 0.1,
    sourceBytesOrNull: 5_000,
    daysSinceLastCommitOrNull: 1_500,
    repositoryUrlOrNull: 'https://github.com/vercel/ms',
    isMonorepoHoused: false,  },
];

/**
 Counts non-overlapping occurrences of a substring inside a string.
 
 @param haystack - String to scan.
 @param needle - Substring to count; must be non-empty.
 
 @returns Number of times `needle` appears in `haystack`.
 */
function countOccurrences(
  { haystack, needle, }: { haystack: string; needle: string; },
): number {
  if (needle === '')
    return 0;
  return haystack.split(needle,).length - 1;
}

await describe({
  name: 'render-controls',
  children: [
    it({
      name: 'emits exactly six dim dropdowns (one per channel)',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        expect(countOccurrences({ haystack: html, needle: 'class="dim-row"', },),).toBe(
          6,
        );
      },
    },),

    it({
      name: 'each channel has a dim-<channel> select id',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        for (const channel of ['x', 'y', 'z', 'color', 'shape', 'size',])
          expect(html,).toContain(`id="dim-${channel}"`,);
      },
    },),

    it({
      name: 'emits exactly seven toggle rows',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        expect(countOccurrences({ haystack: html, needle: 'class="toggle-row"', },),)
          .toBe(7,);
      },
    },),

    it({
      name: 'each toggle row has a three-radio group',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        const radios = countOccurrences({ haystack: html,
          needle: '<input type="radio"', },);
        expect(radios,).toBe(7 * 3,);
      },
    },),

    it({
      name: 'emits six range rows with twelve sliders inside dual-thumb tracks',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        expect(countOccurrences({ haystack: html, needle: 'class="range-row', },),).toBe(
          6,
        );
        expect(countOccurrences({ haystack: html, needle: 'class="range-track"', },),)
          .toBe(6,);
        expect(countOccurrences({ haystack: html, needle: '<input type="range"', },),)
          .toBe(12,);
      },
    },),

    it({
      name: 'each channel has a range-<channel>-min and range-<channel>-max id',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        for (const channel of ['x', 'y', 'z', 'color', 'shape', 'size',]) {
          expect(html,).toContain(`id="range-${channel}-min"`,);
          expect(html,).toContain(`id="range-${channel}-max"`,);
        }
      },
    },),

    it({
      name:
        'emits the search input, display checkboxes, name-labels select, counter, and reset button',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        expect(html,).toContain('id="search"',);
        expect(html,).toContain('id="display-wireframe"',);
        expect(html,).toContain('id="display-planes"',);
        expect(html,).toContain('id="display-axis-labels"',);
        expect(html,).toContain('id="display-unknown"',);
        expect(html,).toContain('id="name-labels"',);
        expect(html,).toContain('id="visibility-counter"',);
        expect(html,).toContain('id="reset"',);
      },
    },),

    it({
      name: 'visibility counter starts at "N of N visible"',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        expect(html,).toContain(
          `${PROBES.length.toString()} of ${PROBES.length.toString()} visible`,
        );
      },
    },),

    it({
      name: 'shape dropdown disables continuous options it cannot represent',
      fn: async () => {
        const html = renderControls({
          probes: PROBES,
          state: defaultState({ probes: PROBES, },),
        },);
        const shapeBlock = html.slice(html.indexOf('id="dim-shape"',),);
        const shapeBlockOnly = shapeBlock.slice(0, shapeBlock.indexOf('</select>',),);
        expect(shapeBlockOnly,).toContain('value="tsRatio" disabled',);
        expect(shapeBlockOnly,).toContain('value="logSourceBytes" disabled',);
        expect(shapeBlockOnly,).toContain('value="isLeafNumeric"',);
      },
    },),

    it({
      name: 'search input value is HTML-attribute escaped',
      fn: async () => {
        const state = defaultState({ probes: PROBES, },);
        const html = renderControls({
          probes: PROBES,
          state: {
            ...state,
            search: '"><script>x</script>',
          },
        },);
        expect(html,).not.toContain('"><script>',);
        expect(html,).toContain('value="&quot;&gt;&lt;script&gt;x&lt;/script&gt;"',);
      },
    },),
  ],
},);
