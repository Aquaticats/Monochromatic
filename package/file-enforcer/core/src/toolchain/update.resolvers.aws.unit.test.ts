/**
 Unit tests for the `aws` resolver over recorded tag listings and fake zip
 bytes. No test touches the network.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { createHash, } from 'node:crypto';
import {
  type Fetcher,
  resolveAws,
  type ToolPlanEntry,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Tag listing endpoint the resolver reads.
 */
const TAGS_URL = 'https://api.github.com/repos/aws/aws-cli/tags?per_page=100';

/**
 Builds the recorded zip URL of one architecture and version.

 @param arch - vendor architecture name
 @param version - release version

 @returns vendor zip URL
 */
function zipUrl(
  arch: string,
  version: string,
): string {
  return `https://awscli.amazonaws.com/awscli-exe-linux-${arch}-${version}.zip`;
}

/**
 Plan entry shaped like the real `aws` entry at version 2.38.2, with the musl
 rows sharing the glibc archives.
 */
const AWS_ENTRY: ToolPlanEntry = {
  id: 'aws',
  request: { track: 'latest' },
  version: '2.38.2',
  route: 'archive',
  bins: ['aws', 'aws_completer'],
  options: {},
  env: {},
  artifacts: {
    'linux-arm64': { url: zipUrl('aarch64', '2.38.2',), sha256: '1'.repeat(64,), },
    'linux-arm64-musl': { url: zipUrl('aarch64', '2.38.2',), sha256: '1'.repeat(64,), },
    'linux-x64': { url: zipUrl('x86_64', '2.38.2',), sha256: '2'.repeat(64,), },
    'linux-x64-musl': { url: zipUrl('x86_64', '2.38.2',), sha256: '2'.repeat(64,), },
  },
};

/**
 Renders a tag listing the way the GitHub API does.

 @param names - tag names in listing order

 @returns JSON body
 */
function tagListing(names: readonly string[],): string {
  return JSON.stringify(names.map((name,) => ({ name, })),);
}

/**
 Builds a fetcher that serves a tag listing and zip bytes, recording each GET.

 @param tags - tag listing body
 @param zips - zip bytes keyed by URL

 @returns fetcher and the URLs it served, in order
 */
function recordingFetcher(
  tags: string,
  zips: Readonly<Record<string, string>>,
): {
  readonly fetcher: Fetcher;
  readonly served: string[];
} {
  /**
   URLs served so far.
   */
  const served: string[] = [];
  return {
    served,
    fetcher: async (url,) => {
      served.push(url,);
      if (url === TAGS_URL)
        return { ok: true, status: 200, text: tags, };
      /**
       Recorded zip content for this URL.
       */
      const content = zips[url];
      if (content === undefined)
        throw new Error(`unexpected fetch: ${url}`,);
      return {
        ok: true,
        status: 200,
        text: '',
        bytes: new TextEncoder().encode(content,),
      };
    },
  };
}

/**
 SHA-256 of a text, as the resolver computes it for a zip.

 @param content - zip content

 @returns lowercase hexadecimal digest
 */
function sha256Of(content: string,): string {
  return createHash('sha256',)
    .update(content,)
    .digest('hex',);
}

//endregion Fixtures

await describe({
  name: resolveAws.name,
  children: [
    it({
      name: 'picks the highest stable v2 tag regardless of listing order and hashes each distinct zip once',
      fn: async () => {
        const { fetcher, served, } = recordingFetcher(
          tagListing(['2.39.1', '1.44.9', '2.39.10', '2.39.2', '2.40.0-rc1', '2.9.0',],),
          {
            [zipUrl('aarch64', '2.39.10',)]: 'arm zip',
            [zipUrl('x86_64', '2.39.10',)]: 'x64 zip',
          },
        );
        expect(await resolveAws({ entry: AWS_ENTRY, fetcher, },),)
          .toEqual({
            version: '2.39.10',
            artifacts: {
              'linux-arm64': { url: zipUrl('aarch64', '2.39.10',), sha256: sha256Of('arm zip',), },
              'linux-arm64-musl': { url: zipUrl('aarch64', '2.39.10',), sha256: sha256Of('arm zip',), },
              'linux-x64': { url: zipUrl('x86_64', '2.39.10',), sha256: sha256Of('x64 zip',), },
              'linux-x64-musl': { url: zipUrl('x86_64', '2.39.10',), sha256: sha256Of('x64 zip',), },
            },
          },);
        expect(served,)
          .toEqual([TAGS_URL, zipUrl('aarch64', '2.39.10',), zipUrl('x86_64', '2.39.10',),],);
      },
    },),
    it({
      name: 'returns the recorded rows without downloading when the newest tag is the recorded version',
      fn: async () => {
        const { fetcher, served, } = recordingFetcher(
          tagListing(['2.38.2', '2.38.1',],),
          {},
        );
        expect(await resolveAws({ entry: AWS_ENTRY, fetcher, },),)
          .toEqual({ version: '2.38.2', artifacts: AWS_ENTRY.artifacts, },);
        expect(served,)
          .toEqual([TAGS_URL,],);
      },
    },),
    it({
      name: 'throws when the listing holds no stable v2 release',
      fn: async () => {
        const { fetcher, } = recordingFetcher(
          tagListing(['1.44.9', '2.40.0-rc1', 'nightly',],),
          {},
        );
        await expect(resolveAws({ entry: AWS_ENTRY, fetcher, },),)
          .rejects
          .toThrow('github.com/aws/aws-cli lists no stable v2 release',);
      },
    },),
    it({
      name: 'throws when a recorded URL does not carry the recorded version',
      fn: async () => {
        const { fetcher, } = recordingFetcher(
          tagListing(['2.39.2',],),
          {},
        );
        await expect(resolveAws({
          entry: {
            ...AWS_ENTRY,
            artifacts: { 'linux-x64': { url: 'https://example.com/aws.zip', sha256: '2'.repeat(64,), }, },
          },
          fetcher,
        },),)
          .rejects
          .toThrow('does not carry the recorded version 2.38.2',);
      },
    },),
    it({
      name: 'throws when a zip download fails',
      fn: async () => {
        /**
         Fetcher whose zip downloads answer 404.
         */
        const fetcher: Fetcher = async (url,) => url === TAGS_URL
          ? { ok: true, status: 200, text: tagListing(['2.39.2',],), }
          : { ok: false, status: 404, text: '', };
        await expect(resolveAws({ entry: AWS_ENTRY, fetcher, },),)
          .rejects
          .toThrow('HTTP 404',);
      },
    },),
  ],
},);
