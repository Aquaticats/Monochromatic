# Firefox image loaders in an isolated XDG data directory

In a nested Wayland test, Firefox aborted before opening a page when `XDG_DATA_DIRS`
pointed only to an empty fixture directory. GTK reported
`Failed to load /org/gtk/libgtk/icons/16x16/status/image-missing.png` with
`Unrecognized image file format (gdk-pixbuf-error-quark, 3)`.
Adding the shared MIME data alone changed the error to `No image loaders are configured`.

On the test machine, `pacman --query --list glycin` showed the loader configuration under
`/usr/share/glycin-loaders` and its executables under `/usr/lib/glycin-loaders`.
The passing Browser Selector test linked `/usr/share/mime` and `/usr/share/glycin-loaders`
into the disposable XDG data directory. Firefox then opened the local HTTP targets with
the selected profile on both initial and running-instance launches. The fixture still had
its own registered applications and default associations.

## Developer Edition tab processes

In the same fixture, Firefox Developer Edition reported
`Failed to launch tab subprocess @FSC::SFNS::Recv (Error:0)`.
Setting `dom.ipc.forkserver.enable` to `false` in the disposable profile allowed the cold launch.
The running instance received the second target after a startup delay in the passing run.
Some content processes still exited on signal 11. This setting was used only for the test profile.
