/**
 Tests for the pinned-tooltip HTML renderer.

 The tooltip splices probe-derived values, which include npm names, licence
 classes and repository URLs, into an HTML fragment that the page sets as
 `innerHTML`, so every value must be entity-escaped for text and for either
 quoting style.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  formatTooltipHtml,
  type PackageProbe,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Probe carrying the hostile text under test in its npm name, licence class and
 repository URL, which are the three fields the tooltip splices verbatim.
 */
const HOSTILE_PROBE: PackageProbe = {
  catalogKey: 'hostile',
  npmName: `hostile's "package" <script>alert('x')</script>`,
  resolvedVersion: '1.0.0',
  isLeaf: true,
  weeklyDownloads: 0,
  installSizeBytes: 0,
  packageAgeDays: 0,
  licenseClass: 'permissive',
  runtimeDepCount: 0,
  transitiveDepCount: 0,
  repositoryUrlOrNull: `'https://example.test/on'click='x`,
  isMonorepoHoused: false,
};

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: formatTooltipHtml.name,
      children: [
        //region Every spliced value is entity escaped

        it({
          name: 'escapes the double quote and the angle brackets of a probe name',
          fn: async () => {
            /**
             Rendered tooltip fragment.
             */
            const html = formatTooltipHtml({ probe: HOSTILE_PROBE, },);
            expect(html,).not.toContain('<script>',);
            expect(html,).toContain('&lt;script&gt;',);
            expect(html,).toContain('&quot;package&quot;',);
          },
        },),

        it({
          name: 'escapes the apostrophe of a probe name so a single-quoted attribute cannot break out',
          fn: async () => {
            /**
             Rendered tooltip fragment.
             */
            const html = formatTooltipHtml({ probe: HOSTILE_PROBE, },);
            expect(html,).not.toContain("hostile's",);
            expect(html,).toContain('hostile&#39;s',);
          },
        },),

        it({
          name: 'escapes the apostrophes of a repository URL',
          fn: async () => {
            /**
             Rendered tooltip fragment.
             */
            const html = formatTooltipHtml({ probe: HOSTILE_PROBE, },);
            expect(html,).toContain('&#39;https://example.test/on&#39;click=&#39;x',);
          },
        },),

        //endregion Every spliced value is entity escaped
      ],
    },),
  ],
},);
