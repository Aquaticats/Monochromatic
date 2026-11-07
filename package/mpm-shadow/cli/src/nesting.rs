//! What:
//! The depth guard of the shadow:
//! how many `mpm-shadow` processes sit above this one in a chain of starts.
//! Why:
//! A shadow that mistakes another shadow (or its launcher) for the real `mpm` starts it,
//! which does the same again;
//! without a bound that chain grows until the host refuses new processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const depth = Number(process.env.MPM_SHADOW_NESTING ?? 0);
//! if (depth >= MAX_NESTING) refuse();
//! ```

/// The environment variable that carries the depth down to the program the shadow starts.
pub const NESTING_ENV: &str = "MPM_SHADOW_NESTING";

/// The depth at which a shadow refuses to start another program.
/// A person's shell,
/// one shadow and the real `mpm` make depth one;
/// a few more cover mpm calling mpm.
pub const MAX_NESTING: u32 = 4;

/// The depth a value of `NESTING_ENV` names;
/// absent or unreadable text names none.
pub fn nesting_depth(value: Option<&str>) -> u32 {
    let Some(text) = value else {
        return 0;
    };
    return text.parse::<u32>().unwrap_or(0);
}

/// Whether a shadow at `depth` must refuse to start the program it found.
pub fn must_refuse(depth: u32) -> bool {
    return depth >= MAX_NESTING;
}
