//! What:
//!  The configuration and attribute locations a content decision reads in a fixture.
//! Why:
//!  A control that settles a file by content must not depend on the machine's own Git configuration;
//!  this names none,
//!  and reaches no Git to learn about one.
//! The code sits in a file of its own so `test_support.rs` stays under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const environment = contentEnvironment(); // no system or global file, no system attributes file
//! ```

use crate::repo_read_config_scan::ScopeEnvironment;
use crate::worktree_clean_content::{ConversionEnvironment, SystemAttributes};
use std::path::PathBuf;

/// What:
///  The configuration and attribute locations a content decision reads in a fixture:
/// no system or global file,
///  and no system attributes file.
/// Why:
///   A control that settled a file by content must not depend on the machine's own Git configuration;
/// this names none,
///  and reaches no Git to learn about one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const environment = { scope: { noSystem: true, global: '/nonexistent-global-config' }, systemAttributes: 'disabled', variables: [] };
/// ```
pub(crate) fn content_environment() -> ConversionEnvironment {
    return ConversionEnvironment {
        scope: ScopeEnvironment { no_system: true, global: Some(PathBuf::from("/nonexistent-global-config")), home: None, xdg: None, ..ScopeEnvironment::default() },
        system_attributes: SystemAttributes::Disabled,
        alternate_attribute_source: None,
        variables: Vec::new(),
    };
}
