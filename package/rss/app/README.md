# RSS Reader

An RSS/Atom feed reader that converts feeds to an HTML interface with memoized updates.

## Features

- **Multi-format support**:
   reads both RSS and Atom feeds
- **OPML integration**:
   configure feeds using OPML files
- **Memoized pipeline**:
   time-bucketed and content-derived cache invalidation avoids redundant fetches
- **Embedded content**:
   safe rendering of feed descriptions in sandboxed iframes
- **Auto-dismiss**:
   items scrolled past are automatically marked as ignored

## Architecture

Pull-based memoized pipeline triggered on each page request:

```text
OPML URLs -> Fetch OPML texts -> Parse outlines -> Fetch feeds -> Sort items -> Filter ignored -> Render HTML
```

Two memoize layers with distinct salt strategies:

- **Fetch layer**:
   time-bucketed salt (`date % interval`),
   so feeds are re-fetched only when the interval window advances
- **Render layer**:
   salt combines fetch time bucket + ignore file content,
   so rendering updates when either feeds or ignore list change

### Core modules

- **`opmls.ts`**:
   reads and validates OPML source URLs from environment
- **`outline.ts`**:
   fetches OPML files and extracts feed outlines
- **`feed.ts`**:
   fetches and parses RSS/Atom feeds
- **`item.ts`**:
   extracts,
   normalizes,
   and sorts feed items
- **`html.ts`**:
   renders items to HTML,
   filtering out ignored entries
- **`interval.ts`**:
   configurable time-bucket for fetch cache invalidation
- **`index.ts`**:
   memoized pipeline orchestration and HTTP server

## Configuration

### Environment variables

Create a `.env` file in the project root:

```bash
# Comma-separated list of OPML file URLs
OPMLS=file://package/rss/app/src/monitor.opml
RSS_PORT=4112
# Fetch cache interval in milliseconds (default: 300000 = 5 minutes)
RSS_FETCH_INTERVAL_MS=300000
```

### OPML format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head>
    <title>Feed Collection</title>
  </head>
  <body>
    <outline text="Category Name">
      <outline text="Feed Name" type="rss" xmlUrl="https://example.com/feed.xml" htmlUrl="https://example.com/"/>
    </outline>
  </body>
</opml>
```

## API endpoints

- `GET /`:
   serve the RSS reader interface
- `POST /api/ignore/new`:
   mark a feed item as ignored

## Usage

1. Configure your OPML file with desired RSS/Atom feeds
2. Set the `OPMLS` environment variable to point to your OPML file(s)
3. Start the server:
    `mise run //package/rss/app:dev`
4. Visit `http://localhost:4112` to view your feeds

## Technical details

- Built with h3's `H3` router and `serve()` helper for the HTTP server
- Uses h-html for string-based HTML generation
- Supports both HTTP and file-based OPML sources
- Salted memoize caching with automatic invalidation
