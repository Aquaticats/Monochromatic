---
"@monochromatic-dev/fs-path-core": patch
---

The ensure and empty families now read one path spelling the same way.
`ensurePath('dist/x.js?raw')` created a file literally named `dist/x.js?raw`
while `emptyPath('dist/x.js?raw')` truncated the unrelated `dist/x.js`,
so a
caller that ensured then emptied one spelling destroyed a different file.
Every function in both families strips a `?query` suffix before it inspects or
touches the disk,
and returns the spelling it was given.
