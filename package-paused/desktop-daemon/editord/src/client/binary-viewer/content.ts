/**
 * Content factories for the binary viewer component.
 *
 * Creates DOM elements for each media type: image, audio (with repeat
 * toggle), video, and hex dump. Used by {@link BinaryViewer} to keep
 * the component class concise.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Repeat/loop icon as inline SVG.
 * Single circular arrow indicating track repeat.
 */
const REPEAT_ICON_SVG =
  `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m17 2 4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/></svg>`;

/**
 * Creates an image preview element.
 *
 * @param url - source URL for the image
 *
 * @returns img element
 *
 * @example
 * ```ts
 * const result = createImageContent({ url: '/_raw?token=tok\&path=/src/image.png', });
 * ```
 */
export function createImageContent({ url, }: { readonly url: string; },): HTMLElement {
  return h({
    tag: 'img',
    attrs: {
      src: url,
      alt: 'Image preview',
    },
  },);
}

/**
 * Creates an audio player with a repeat toggle button.
 *
 * @param url - source URL for the audio file
 *
 * @returns container with audio element and repeat button
 *
 * @example
 * ```ts
 * const result = createAudioContent({ url: '/_raw?token=tok\&path=/src/image.png', });
 * ```
 */
export function createAudioContent({ url, }: { readonly url: string; },): HTMLElement {
  /**
   * Native `<audio controls>` element; its `loop` flag is toggled by `repeatBtn`.
   */
  const audio = h({
    tag: 'audio',
    attrs: {
      src: url,
      controls: '',
    },
  },);

  /**
   * Custom repeat-track button whose click handler flips `audio.loop`.
   */
  const repeatBtn = h({
    tag: 'button',
    class: 'repeat-btn',
    html: REPEAT_ICON_SVG,
    attrs: {
      type: 'button',
      title: 'Repeat track',
    },
    on: {
      click: function handleRepeatToggle(): void {
        audio.loop = !audio.loop;
        repeatBtn.toggleAttribute(
          'data-active',
          audio.loop,
        );
      },
    },
  },);

  return h({
    tag: 'div',
    class: 'audio-controls',
    children: [
      audio,
      repeatBtn,
    ],
  },);
}

/**
 * Creates a video player element.
 *
 * @param url - source URL for the video file
 *
 * @returns video element
 *
 * @example
 * ```ts
 * const result = createVideoContent({ url: '/_raw?token=tok\&path=/src/image.png', });
 * ```
 */
export function createVideoContent({ url, }: { readonly url: string; },): HTMLElement {
  return h({
    tag: 'video',
    attrs: {
      src: url,
      controls: '',
    },
  },);
}

/**
 * Creates a preformatted hex dump display.
 *
 * @param content - hex dump text to display
 *
 * @returns pre element
 *
 * @example
 * ```ts
 * const result = createHexDumpContent({ content: 'const x = 42;', });
 * ```
 */
export function createHexDumpContent({ content, }: { readonly content: string; },): HTMLElement {
  return h({
    tag: 'pre',
    text: content,
  },);
}
