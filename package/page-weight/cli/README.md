# @monochromatic-dev/page-weight-cli

Per-page transfer-size audit for a built static site.

## What it does

For each `.html` file under the given dist directory,
sums the wire transfer size of the HTML itself plus every asset a browser
would fetch to render it,
 then reports the distribution:
`min`,
 `max`,
 `mean`,
 `median`.

The script has no knowledge of any particular site;
it treats the dist directory as a black box of HTML and adjacent assets.

## Wire size rule

For each counted file,
 the script reports the size of the `.zst`
companion when one exists alongside it on disk,
 otherwise the raw size.
This matches file servers that serve precompressed variants when the client
advertises `Accept-Encoding: zstd`.

## Asset discovery

- **HTML**:
   `<link href>`,
   `<script src>`,
   `<img src/srcset>`,
   `<iframe src>`,
  `<embed src>`,
   `<object data>`,
   `<audio src>`,
   `<video src>`,
   `<use href>` / `<use xlink:href>`
- **`<picture>` / `<video>` / `<audio>`**:
   the first child `<source>` is treated
  as the canonical pick (browsers stop at the first matching `<source>`)
- **`<img srcset>`**:
   first candidate URL
- **CSS**:
   `url(...)` references in linked stylesheets **and** inline `<style>` blocks,
  with `@import` chains followed recursively

## Skipped

External URLs (`http://`,
 `https://`,
 `//cdn...`,
 `data:...`) and fragment-only
references (`#id`) do not contribute to transfer weight and are omitted.

## Usage

```sh
node package/page-weight/cli/src/index.ts path/to/dist
```

The script accepts any dist directory:
 use it against any SSG output,
not just this repo's.

## Output

```text
page                              bytes  assets
index.html                       12.4 KiB      8
about.html                       10.2 KiB      7
...

pages:  14
min:    8.1 KiB
max:    12.4 KiB
mean:   10.3 KiB
median: 10.5 KiB
```

When references can't be resolved to a file under the dist root,
a warning list is printed to stderr and the process exits with code `2`
so the mise task (or CI) can flag broken builds.

## Exit codes

- `0`:
   summary printed,
   every referenced asset resolved
- `1`:
   bad arguments (no dist directory,
   or no HTML files under it)
- `2`:
   summary printed,
   one or more referenced assets could not be resolved
