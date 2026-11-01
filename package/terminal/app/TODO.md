# Terminal TODO

## Fix severe input stutter

Typing in the terminal stutters extremely.
Investigate the input path from Slint key events through PTY writes and Ghostty rendering,
 then reduce latency at the user boundary.

## FIXME: Ghostty resize crash on package run

Running the package task can panic inside Ghostty page-list resize integrity checks.
Reproduce with `mise run //package/terminal/app:run`,
 then inspect the resize path that sends Slint-measured
rows and columns into Ghostty.

```txt
user@bazzite:~/Monochromatic$ mise run //package/terminal/app:run
[//package/terminal/app:run] $ let home = ($env.HOME? | default '')
    Finished `release` profile [optimized] target(s) in 0.85s
[//package/terminal/app:stage:r…] $ let lib_dirs = (glob "target/relea…
warning(page_list): PageList integrity violation: viewport pin rows too small rows=23 needed=27
error(page_list): PageList integrity check failed: error.ViewportPinInsufficientRows
thread 3687167 panic: PageList integrity check failed
/work/target/release/build/libghostty-vt-sys-debc892d5bbdb6dd/out/ghostty-src/src/terminal/PageList.zig:619:13: 0x7f19a3b084b8 in verifyIntegrity (lib_vt.zig)
/work/target/release/build/libghostty-vt-sys-debc892d5bbdb6dd/out/ghostty-src/src/terminal/PageList.zig:515:9: 0x7f19a3b17f14 in grow (lib_vt.zig)
/work/target/release/build/libghostty-vt-sys-debc892d5bbdb6dd/out/ghostty-src/src/terminal/PageList.zig:1169:30: 0x7f19a3b0f163 in resizeCols (lib_vt.zig)
/work/target/release/build/libghostty-vt-sys-debc892d5bbdb6dd/out/ghostty-src/src/terminal/PageList.zig:978:32: 0x7f19a3a54541 in resize (lib_vt.zig)
/work/target/release/build/libghostty-vt-sys-debc892d5bbdb6dd/out/ghostty-src/src/terminal/Screen.zig:1753:26: 0x7f19a3a020fc in resize (lib_vt.zig)
/work/target/release/build/libghostty-vt-sys-debc892d5bbdb6dd/out/ghostty-src/src/terminal/c/terminal.zig:396:13: 0x7f19a3a08974 in resize (lib_vt.zig)
???:?:?: 0x557bd2d9d48f in ??? ()
Unwind information for `:0x557bd2d9d48f` was not available, trace may be incomplete
```
