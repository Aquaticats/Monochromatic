---
"@monochromatic-dev/fs-path-core": patch
---

`ensureFile` no longer truncates a file a second writer creates while the call runs.
The file was created from a `stat` result with an open that truncates,
so a file created in the window between the two lost its contents.
The create is now exclusive:
`EEXIST` falls back to the existing-file path,
which checks the entry kind and the owner bits,
and the file the second writer created keeps its contents.
