---
"@monochromatic-dev/fs-path-core": patch
---

`ensureFile` creates a bare file name such as `bare.txt` instead of calling `mkdir('')`.
A bare name has no parent segment,
and the empty parent string was passed to `ensureDir`,
whose `mkdir('')` threw `ENOENT` before the file was written.
The parent directory is now created only when the path names one.
