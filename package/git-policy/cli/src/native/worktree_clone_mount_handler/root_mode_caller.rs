//! What:
//!  The refusal a root mode answers a caller that is not root with:
//!  the mode's own name,
//!  what the refusal left untouched,
//!  why the mode runs as root,
//!  and the one way forward.
//! Why:
//!  Both root modes refuse the same way,
//!  so a person meets one plain sentence rather than two that differ in tone
//!  (the adoption answered this first,
//!  and the start probe answers the same words).
//!  Naming the effective user is what tells the caller that the fix is theirs:
//!  the file or directory the mode would have touched is not the problem.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sentence = new NotRootRefusal({ mode: 'start-probe', nothing: 'read, mounted or recorded', because, way, effectiveUser: 1000 }).sentence();
//! ```

/// What:
///  One root mode's refusal of a caller that is not root.
/// Why:
///   The four phrases are what the sentence needs,
///  and keeping them apart keeps the sentence one format string rather than
///  two nearly identical ones in two folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NotRootRefusal = { mode: string; nothing: string; because: string; way: string; effectiveUser: number };
/// ```
pub struct NotRootRefusal<'a> {
    /// The mode as a person types it.
    pub mode: &'a str,
    /// What the refusal left untouched,
    /// as the phrase after "Nothing was".
    pub nothing: &'a str,
    /// Why the mode runs as root.
    pub because: &'a str,
    /// The one way forward,
    /// as the phrase after "Ways forward: ".
    pub way: &'a str,
    /// The effective user the kernel reported.
    pub effective_user: u32,
}

impl NotRootRefusal<'_> {
    /// The sentence a person reads,
    /// opening with the words every root mode is reached through,
    /// and then naming the mode,
    /// the effective user,
    /// what was not touched,
    /// why root is required and what to do.
    pub fn sentence(&self) -> String {
        let named: String = super::handler_install_paths::root_modes_name();
        return format!(
            "{named}: the {} mode runs as root, and this process's effective user is {}. Nothing was {}, because {}. Ways forward: {}.",
            self.mode, self.effective_user, self.nothing, self.because, self.way
        );
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "root_mode_caller_tests.rs"]
mod tests;
