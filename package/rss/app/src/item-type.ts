import type {
  AtomFeed,
  Opml,
  RssFeed,
} from 'feedsmith';
import type { DeepReadonly, } from './types.ts';

/**
 Individual feed item with its parent feed metadata and OPML outline.
 Deeply readonly because the pipeline only reads these parsed values.
 */
export type Item = DeepReadonly<{
  feed: Omit<RssFeed.Feed<string> | AtomFeed.Feed<string>, 'entries' | 'items'>;
  outline: Opml.Outline<string>;
  item: RssFeed.Item<string> | AtomFeed.Entry<string>;
}>;

/**
 Feed item normalized to RSS-like structure for uniform rendering.
 Atom entries are converted to match RSS item shape.
 Deeply readonly because the renderers only read these values.
 */
export type NormalizedItem = DeepReadonly<{
  feed: Omit<RssFeed.Feed<string>, 'items'>;
  originalFeed?: Omit<RssFeed.Feed<string> | AtomFeed.Feed<string>, 'entries' | 'items'>;
  outline: Opml.Outline<string>;
  item: RssFeed.Item<string>;
  originalItem?: RssFeed.Item<string> | AtomFeed.Entry<string>;
}>;

/**
 Atom feed item narrowed to its specific types for normalization.
 Deeply readonly because normalization only reads these fields.
 */
export type AtomItem = DeepReadonly<{
  feed: Omit<AtomFeed.Feed<string>, 'entries'>;
  outline: Opml.Outline<string>;
  item: AtomFeed.Entry<string>;
}>;

/**
 Normalized item with an extracted publication date for sorting.
 */
export type ItemWDate = NormalizedItem & { readonly pubDateDate: Date; };
