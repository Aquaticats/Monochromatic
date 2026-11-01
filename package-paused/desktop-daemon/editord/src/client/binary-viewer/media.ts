/**
 * Media display methods for the binary viewer.
 *
 * Each function clears the shadow root, appends media-specific content,
 * optionally appends ffprobe metadata, and makes the host visible.
 * Extracted from binary-viewer.ts to keep the class under max-lines.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import {
  createAudioContent,
  createImageContent,
  createVideoContent,
} from './content.ts';

/**
 * Appends a `<pre>` block with ffprobe metadata below the current content.
 * Does nothing when `mediaInfo` is undefined.
 *
 * @param shadow - shadow root to append to
 *
 * @param mediaInfo - trimmed ffprobe output, or undefined to skip
 */
function appendMediaInfo(
  {
    shadow,
    mediaInfo,
  }: {
    readonly shadow: ShadowRoot;
    readonly mediaInfo: string | undefined;
  },
): void {
  if (mediaInfo === undefined)
    return;
  shadow.append(h({
    tag: 'pre',
    class: 'media-info',
    text: mediaInfo,
  },),);
}

/**
 * Clears and shows an image in the shadow root.
 *
 * @param shadow - shadow root to render into
 *
 * @param host - host element to make visible
 *
 * @param clear - clears the shadow root
 *
 * @param url - source URL for the image
 *
 * @param mediaInfo - optional ffprobe metadata to display below the image
 *
 * @example
 * ```ts
 * showImage({ shadow: pane.shadowRoot, host: hostElement, clear: clearFn, url: '/_raw?token=tok\&path=/src/image.png', mediaInfo: { mime: "image/png", width: 800, height: 600 }, });
 * ```
 */
export function showImage({
  shadow,
  host,
  clear,
  url,
  mediaInfo,
}: {
  readonly shadow: ShadowRoot;
  readonly host: HTMLElement;
  readonly clear: () => void;
  readonly url: string;
  readonly mediaInfo: string | undefined;
},): void {
  clear();
  shadow.append(createImageContent({ url, },),);
  appendMediaInfo({
    shadow,
    mediaInfo,
  },);
  host.style
    .display = 'flex';
}

/**
 * Clears and shows an audio player with repeat toggle in the shadow root.
 *
 * @param shadow - shadow root to render into
 *
 * @param host - host element to make visible
 *
 * @param clear - clears the shadow root
 *
 * @param url - source URL for the audio file
 *
 * @param mediaInfo - optional ffprobe metadata to display below the player
 *
 * @example
 * ```ts
 * showAudio({ shadow: pane.shadowRoot, host: hostElement, clear: clearFn, url: '/_raw?token=tok\&path=/src/track.mp3', mediaInfo: 'audio/mpeg, 3:42', });
 * ```
 */
export function showAudio({
  shadow,
  host,
  clear,
  url,
  mediaInfo,
}: {
  readonly shadow: ShadowRoot;
  readonly host: HTMLElement;
  readonly clear: () => void;
  readonly url: string;
  readonly mediaInfo: string | undefined;
},): void {
  clear();
  shadow.append(createAudioContent({ url, },),);
  appendMediaInfo({
    shadow,
    mediaInfo,
  },);
  host.style
    .display = 'flex';
}

/**
 * Clears and shows a video player in the shadow root.
 *
 * @param shadow - shadow root to render into
 *
 * @param host - host element to make visible
 *
 * @param clear - clears the shadow root
 *
 * @param url - source URL for the video file
 *
 * @param mediaInfo - optional ffprobe metadata to display below the player
 *
 * @example
 * ```ts
 * showVideo({ shadow: pane.shadowRoot, host: hostElement, clear: clearFn, url: '/_raw?token=tok\&path=/src/clip.mp4', mediaInfo: 'video/mp4, 1920x1080', });
 * ```
 */
export function showVideo({
  shadow,
  host,
  clear,
  url,
  mediaInfo,
}: {
  readonly shadow: ShadowRoot;
  readonly host: HTMLElement;
  readonly clear: () => void;
  readonly url: string;
  readonly mediaInfo: string | undefined;
},): void {
  clear();
  shadow.append(createVideoContent({ url, },),);
  appendMediaInfo({
    shadow,
    mediaInfo,
  },);
  host.style
    .display = 'flex';
}
