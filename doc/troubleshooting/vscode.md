# VS Code Remote-WSL single-window-per-workspace constraint prevents opening the same workspace in multiple windows

## Symptom

In a WSL environment with the VS Code Remote-WSL extension installed,
attempting to open the same workspace folder in more than one window
fails:

- `File > New Window` then opening the same folder switches focus back
  to the existing window instead of creating a second one.
- `code --new-window /path/to/workspace` from the WSL shell behaves
  the same way.
- Opening a new window first and then the workspace inside it skips
  the Remote-WSL connection;
   the window opens as a Windows-side
  window with no WSL filesystem access,
   so it cannot edit the
  workspace files at all.

Use case that hit this:
 needing a second VS Code window on the same
workspace to keep a long-running terminal and a debugger separate
from the editing window.

## Root cause

VS Code identifies a workspace by its absolute path.
 The
"focus existing window" behaviour is intentional:
 opening a folder
that matches an already-open window's path is treated as a
"return to that workspace" gesture,
 not as a request for a second
instance.
 The Remote-WSL extension does not expose a setting to
override this identity check.

Without Remote-WSL,
 the same trick (open in a new local window) works
because the new local window connects to a different host context.
With Remote-WSL,
 the new window has to either reuse the existing WSL
connection (which deduplicates by path) or connect natively to
Windows (which loses access to the WSL filesystem).

There is no public VS Code API that lets the user say "treat this
path as a different workspace".
 The identifier is the canonicalised
absolute path,
 full stop.

## Verification

Version under test:

- VS Code 1.85+ (any version with the current Remote-WSL extension)
- Remote-WSL extension 0.81+
- WSL 2 on Windows 11

Reproduce:

```bash
# In WSL shell:
code /home/user/projects/Monochromatic
# Window 1 opens.

code --new-window /home/user/projects/Monochromatic
# Window 1 receives focus; no new window appears.
```

Verify the workaround below makes both windows appear independently
(both showing the same files;
 edits in one appear in the other
immediately).

## Verified workaround: Linux bind mounts present the same directory under multiple paths

The identity check is path-based,
 so giving the directory two paths
makes VS Code treat them as two workspaces while the underlying
filesystem stays single-sourced.

### Setup

```bash
sudo mkdir -p /home/user/projects/Monochromatic-view2
sudo mkdir -p /home/user/projects/Monochromatic-view3

sudo mount --bind /home/user/projects/Monochromatic /home/user/projects/Monochromatic-view2
sudo mount --bind /home/user/projects/Monochromatic /home/user/projects/Monochromatic-view3
```

### Open multiple instances

```bash
code /home/user/projects/Monochromatic        # window 1
code /home/user/projects/Monochromatic-view2  # window 2
code /home/user/projects/Monochromatic-view3  # window 3
```

### Cleanup

```bash
sudo umount /home/user/projects/Monochromatic-view2
sudo umount /home/user/projects/Monochromatic-view3
rmdir /home/user/projects/Monochromatic-view2
rmdir /home/user/projects/Monochromatic-view3
```

### Persistence across reboots

Add `/etc/fstab` entries:

```text
/home/user/projects/Monochromatic /home/user/projects/Monochromatic-view2 none bind 0 0
/home/user/projects/Monochromatic /home/user/projects/Monochromatic-view3 none bind 0 0
```

### Tradeoffs

- Requires sudo to mount,
   unmount,
   and edit `/etc/fstab`.
- All instances share the same git state (HEAD,
   branches,
   staged
  changes).
   Two windows checking out different branches will fight.
- Simultaneous debug sessions on the same files can conflict (two
  debuggers attaching to one process,
   two task runners spawning the
  same port).
- Each window's terminal is independent (no shared shell state).
   This
  is usually the goal of the second window.

## What does not work

- Git worktrees:
   each worktree has its own checkout,
   so changes
  in one do not propagate live to the other.
   Suitable for working on
  different branches concurrently,
   not for "same files in two
  windows";
   different goal.
- **Symbolic links** (`ln -s`):
   VS Code canonicalises the path before
  the identity check,
   so a symlink resolves back to the same target
  and gets treated as the same workspace.
   Mount binds are not
  canonicalised away.
- Code-server / VS Code in browser:
   works in a different process
  altogether so collisions are avoided,
   but the workflow is
  browser-based and tooling integrations (extensions,
   debugger
  protocols) differ from the desktop VS Code.
- A Windows-native second window of the WSL folder via `\\wsl$\…`:
  loses Remote-WSL integration;
   editing files is fine but extensions
  expecting the WSL connection (test runners,
   debuggers,
   language
  servers configured for WSL paths) misbehave.
- Virtual machines:
   resource heavy,
   slow boot,
   file synchronisation
  becomes its own problem.

## Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** Borderline.
     The
    single-window-per-workspace identification is an explicit product
    choice;
     many users prefer it.
     The Remote-WSL extension's lack of
    an override is the actual friction point.
2.  **Can upstream fix it?
    ** Yes,
     by exposing a setting such as
    `remote.WSL.allowMultipleWindows` that disables the path-based
    dedup for WSL workspaces.
     The change is local to the extension.
3.  **Are they supporting this use case?
    ** Not directly;
     the docs
    recommend reorganising work to fit a single window.
4.  **Will they likely fix it?
    ** Unknown.
     The bind-mount workaround is
    well known on the GitHub tracker.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report.
 The bind-mount workaround is simple,
reversible,
 and survives reboots once `/etc/fstab` is updated.
 Revisit
if the workaround stops working after a VS Code or Remote-WSL update.

## Related

- [dprint VS Code extension PATH gap in WSL](dprint.md)
  ;
   a sibling Remote-WSL pain point with a different shape.
