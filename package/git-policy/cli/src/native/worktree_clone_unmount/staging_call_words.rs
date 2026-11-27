//! What:
//!  The words a staging call's refusal carries:
//!  the calls by name,
//!  one refusal of one call with its errno,
//!  and the one constructor the route's calls build a refusal with.
//! Why:
//!  A receipt names the step that refused and the kernel's errno,
//!  and the guest arms match those words
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The unmount call from staging").
//!  The words sit apart from the trait and the kernel's calls,
//!  so a reader of either reads a page of names rather than a page of
//!  `unsafe` calls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type StagingCall = 'bind' | 'open-bind' | 'make-private' | 'move-mount' | ...;
//! type StagingCallRefusal = { call: StagingCall; errno: number; detail: string };
//! ```

/// What:
///  One call of the staging route,
///  named as a receipt names the failed step.
/// Why:
///  A refusal must say which call refused and its errno,
///  and the words are read by a person and matched by the guest arms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingCall = 'bind' | 'open-bind' | 'make-private' | 'move' | 'unmount' | 'name-handle' | 'open-handle' | 'mount-id' | 'mount-table' | 'namespace' | 'boot';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StagingCall {
    /// `mount(2)` with `MS_BIND` of the folder onto itself.
    Bind,
    /// The by-name open of the bind under the cache descriptor.
    OpenBind,
    /// `mount_setattr(2)` making the bind private.
    MakePrivate,
    /// `move_mount(2)` of the clone onto the bind.
    Move,
    /// `umount2(2)` through the held descriptor.
    Unmount,
    /// `name_to_handle_at(2)` of the folder.
    NameHandle,
    /// `open_by_handle_at(2)` of the folder.
    OpenHandle,
    /// The final slot hop of the unmount's walk.
    OpenSlot,
    /// A second whole walk of the canonical slot.
    LookAgain,
    /// The mount number in `/proc/self/fdinfo`.
    MountId,
    /// This process's mount table.
    MountTable,
    /// This process's mount namespace identity.
    Namespace,
    /// This boot's identifier.
    Boot,
}

impl StagingCall {
    /// The word a refusal and a receipt carry.
    pub fn word(self) -> &'static str {
        return match self {
            StagingCall::Bind => "bind",
            StagingCall::OpenBind => "open-bind",
            StagingCall::MakePrivate => "make-private",
            StagingCall::Move => "move-mount",
            StagingCall::Unmount => "unmount",
            StagingCall::NameHandle => "name-handle",
            StagingCall::OpenHandle => "open-handle",
            StagingCall::OpenSlot => "open-slot",
            StagingCall::LookAgain => "look-again",
            StagingCall::MountId => "mount-id",
            StagingCall::MountTable => "mount-table",
            StagingCall::Namespace => "namespace",
            StagingCall::Boot => "boot",
        };
    }
}

/// What:
///  One refused call:
///  which call,
///  its errno and the kernel's words.
/// Why:
///  The receipt carries the step and errno,
///  and a refusal that lost either would leave a person with nothing to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingCallRefusal = { call: StagingCall; errno: number; detail: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagingCallRefusal {
    /// The call that refused.
    pub call: StagingCall,
    /// The kernel's errno,
    /// or zero when the call answered without one.
    pub errno: i32,
    /// What the call answered.
    pub detail: String,
}

impl StagingCallRefusal {
    /// The refusal in words.
    pub fn message(
        &self,
        subject: &str,
    ) -> String {
        return format!(
            "{subject}: {} answered {}{}. Nothing later in the unmount ran, so the mount stays where it was and the receipt names this step.",
            self.call.word(),
            self.detail,
            if self.errno == 0 {
                String::new()
            } else {
                format!(" (errno {})", self.errno)
            },
        );
    }
}

/// The refusal of one call with the thread's last error.
pub(super) fn refused(
    call: StagingCall,
    error: std::io::Error,
) -> StagingCallRefusal {
    return StagingCallRefusal {
        call,
        errno: error.raw_os_error().unwrap_or(0),
        detail: error.to_string(),
    };
}
