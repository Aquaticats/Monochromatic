//! What:
//!  The options one mode takes,
//!  read one mode at a time:
//!  the adoption's three,
//!  the holder scan's dataset and its optional nonce,
//!  the record mode's checkout and its optional token,
//!  and the mount mode's one dataset.
//! Why:
//!  Every option is given once and in any order,
//!  a repeated one would leave which value was meant,
//!  and a name or a count the kernel would refuse is refused here rather than
//!  read as another value,
//!  so each mode's own words are read in the one function that names them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const adoption = parseAdoption(words);       // throws ArgumentRefusal
//! const holderScan = parseHolderScan(words);
//! const recordCheckout = parseRecordCheckout(words);
//! const descendant = parseMountPathDescendant(words);
//! ```

use super::handler_mode_argument_refusals::ArgumentRefusal;
use super::handler_mode_arguments_requests::{
    AdoptRequest,
    HolderScanInvocation,
    MountPathDescendantRequest,
    RecordCheckoutRequest,
};
use super::handler_mode_words::{
    CHECKOUT_OPTION,
    DATASET_OPTION,
    DEVICE_OPTION,
    INODE_OPTION,
    NONCE_OPTION,
    REPLACE_OPTION,
};
use crate::start_probe::probe_source_name::check_dataset_name;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::PathBuf;

/// What:
///  Which of an adoption's three options one word is.
/// Why:
///   The grammar reads each option by name and then falls through one exhaustive match,
///  so a new option is a compiler error rather than a word no branch handles.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AdoptionOption = 'checkout' | 'device' | 'inode';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdoptionOption {
    /// `--checkout`.
    Checkout,
    /// `--device`.
    Device,
    /// `--inode`.
    Inode,
}

/// What:
///  Read an adoption's options,
///  each given once and in any order.
/// Why:
///   The three values together are the administrator's identification,
///  and a repeated option would leave which one was meant to be read;
///  the checkout is a path and the other two are decimal counts,
///  so a value the kernel would read as another number is refused here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = parseAdoption(['--checkout', '/repo', '--device', '66306', '--inode', '1234567']);
/// ```
pub(super) fn parse_adoption(words: &[&str]) -> Result<AdoptRequest, ArgumentRefusal> {
    let mut given_checkout: Option<PathBuf> = None;
    let mut given_device: Option<u64> = None;
    let mut given_inode: Option<u64> = None;
    let mut index: usize = 0;
    while index < words.len() {
        let option: &str = words[index];
        let (known, name): (AdoptionOption, &'static str) = match option {
            CHECKOUT_OPTION => (AdoptionOption::Checkout, CHECKOUT_OPTION),
            DEVICE_OPTION => (AdoptionOption::Device, DEVICE_OPTION),
            INODE_OPTION => (AdoptionOption::Inode, INODE_OPTION),
            other => {
                return Err(ArgumentRefusal::UnexpectedArgument {
                    word: other.to_string(),
                });
            }
        };
        let Some(value) = words.get(index + 1).copied() else {
            return Err(ArgumentRefusal::MissingValue {
                option: name,
            });
        };
        match known {
            AdoptionOption::Checkout => {
                if given_checkout.is_some() {
                    return Err(ArgumentRefusal::RepeatedOption {
                        option: name,
                    });
                }
                given_checkout = Some(PathBuf::from(value));
            }
            AdoptionOption::Device => {
                if given_device.is_some() {
                    return Err(ArgumentRefusal::RepeatedOption {
                        option: name,
                    });
                }
                given_device = Some(decimal_count(name, value)?);
            }
            AdoptionOption::Inode => {
                if given_inode.is_some() {
                    return Err(ArgumentRefusal::RepeatedOption {
                        option: name,
                    });
                }
                given_inode = Some(decimal_count(name, value)?);
            }
        }
        index += 2;
    }
    let Some(checkout) = given_checkout else {
        return Err(ArgumentRefusal::MissingOption {
            option: CHECKOUT_OPTION,
        });
    };
    let Some(device) = given_device else {
        return Err(ArgumentRefusal::MissingOption {
            option: DEVICE_OPTION,
        });
    };
    let Some(inode) = given_inode else {
        return Err(ArgumentRefusal::MissingOption {
            option: INODE_OPTION,
        });
    };
    return Ok(AdoptRequest {
        checkout,
        device,
        inode,
    });
}

/// What:
///  Read a holder scan's dataset and its optional nonce,
///  each option given once and in any order.
/// Why:
///   The dataset names the clone the scan reports on,
///  and the nonce names the request this answer belongs to,
///  so a repeated option would leave which one was meant to be read,
///  and a name the kernel would refuse or a nonce that is not 32 lowercase
///  hexadecimal digits is refused here rather than read as another value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invocation = parseHolderScan(['--dataset', 'zpcachyos/…/clones/one', '--nonce', digits]);
/// ```
pub(super) fn parse_holder_scan(words: &[&str]) -> Result<HolderScanInvocation, ArgumentRefusal> {
    let mut dataset: Option<String> = None;
    let mut nonce: Option<RequestNonce> = None;
    let mut index: usize = 0;
    while index < words.len() {
        let option: &str = match words[index] {
            DATASET_OPTION => DATASET_OPTION,
            NONCE_OPTION => NONCE_OPTION,
            other => {
                return Err(ArgumentRefusal::UnexpectedArgument {
                    word: other.to_string(),
                });
            }
        };
        let Some(value) = words.get(index + 1).copied() else {
            return Err(ArgumentRefusal::MissingValue {
                option,
            });
        };
        if option == DATASET_OPTION {
            if dataset.is_some() {
                return Err(ArgumentRefusal::RepeatedOption {
                    option,
                });
            }
            check_dataset_name(value).map_err(|refusal| return ArgumentRefusal::DatasetName {
                mode: "holder-scan",
                what: "the clone's full dataset name, as `zfs list -H -o name` prints it, with no snapshot part",
                name: String::from(value),
                reason: refusal.message(),
            })?;
            dataset = Some(String::from(value));
        } else {
            if nonce.is_some() {
                return Err(ArgumentRefusal::RepeatedOption {
                    option,
                });
            }
            nonce = Some(RequestNonce::parse(value).map_err(|refusal| {
                return ArgumentRefusal::Nonce {
                    reason: refusal.message(),
                };
            })?);
        }
        index += 2;
    }
    let Some(given_dataset) = dataset else {
        return Err(ArgumentRefusal::MissingOption {
            option: DATASET_OPTION,
        });
    };
    return Ok(HolderScanInvocation {
        dataset: given_dataset,
        nonce,
    });
}

/// What:
///  Read a record mode's checkout and its optional replacement token,
///  each option given once and in any order.
/// Why:
///   The checkout is the folder whose dataset the route serves and the token is
///  the GUID the kernel shows now,
///  so the checkout must be an absolute path and a repeated option would leave
///  which one was meant;
///  the token is only text here because the mode compares it with the kernel's
///  own GUID and refuses `replace-token-differs` when they differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = parseRecordCheckout(['--checkout', '/home/user/Monochromatic', '--replace', digits]);
/// ```
pub(super) fn parse_record_checkout(
    words: &[&str]
) -> Result<RecordCheckoutRequest, ArgumentRefusal> {
    let mut checkout: Option<PathBuf> = None;
    let mut replace: Option<String> = None;
    let mut index: usize = 0;
    while index < words.len() {
        let option: &str = match words[index] {
            CHECKOUT_OPTION => CHECKOUT_OPTION,
            REPLACE_OPTION => REPLACE_OPTION,
            other => {
                return Err(ArgumentRefusal::UnexpectedArgument {
                    word: other.to_string(),
                });
            }
        };
        let Some(value) = words.get(index + 1).copied() else {
            return Err(ArgumentRefusal::MissingValue {
                option,
            });
        };
        if option == CHECKOUT_OPTION {
            if checkout.is_some() {
                return Err(ArgumentRefusal::RepeatedOption {
                    option,
                });
            }
            checkout = Some(PathBuf::from(value));
        } else {
            if replace.is_some() {
                return Err(ArgumentRefusal::RepeatedOption {
                    option,
                });
            }
            if value.is_empty() {
                return Err(ArgumentRefusal::MissingValue {
                    option,
                });
            }
            replace = Some(String::from(value));
        }
        index += 2;
    }
    let Some(given_checkout) = checkout else {
        return Err(ArgumentRefusal::MissingOption {
            option: CHECKOUT_OPTION,
        });
    };
    return Ok(RecordCheckoutRequest {
        checkout: given_checkout,
        replace,
    });
}

/// What:
///  Read a mount mode's dataset.
/// Why:
///   The dataset names the work and the record names the checkout it is placed
///  under,
///  so the mode takes no second option and no path,
///  and a name the kernel would refuse is refused here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = parseMountPathDescendant(['--dataset', 'zpcachyos/nosnap/vm-builder-output']);
/// ```
pub(super) fn parse_mount_path_descendant(
    words: &[&str]
) -> Result<MountPathDescendantRequest, ArgumentRefusal> {
    let mut dataset: Option<String> = None;
    let mut index: usize = 0;
    while index < words.len() {
        let option: &str = match words[index] {
            DATASET_OPTION => DATASET_OPTION,
            other => {
                return Err(ArgumentRefusal::UnexpectedArgument {
                    word: other.to_string(),
                });
            }
        };
        let Some(value) = words.get(index + 1).copied() else {
            return Err(ArgumentRefusal::MissingValue {
                option,
            });
        };
        if dataset.is_some() {
            return Err(ArgumentRefusal::RepeatedOption {
                option,
            });
        }
        check_dataset_name(value).map_err(|refusal| return ArgumentRefusal::DatasetName {
            mode: "mount-path-descendant",
            what: "the nested dataset's full name, as `zfs list -H -o name` prints it, with no snapshot part",
            name: String::from(value),
            reason: refusal.message(),
        })?;
        dataset = Some(String::from(value));
        index += 2;
    }
    let Some(given_dataset) = dataset else {
        return Err(ArgumentRefusal::MissingOption {
            option: DATASET_OPTION,
        });
    };
    return Ok(MountPathDescendantRequest {
        dataset: given_dataset,
    });
}

/// What:
///  One option's value as the decimal count it must be,
///  or the refusal naming the option and what was typed.
/// Why:
///   A device or inode is an unsigned number in decimal;
///  a value with a sign,
///  a prefix or a letter is another number to `u64::from_str` or no number at all,
///  so the digits are checked before the parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = decimalCount('--device', '66306'); // throws on '0x10' or '-1'
/// ```
fn decimal_count(
    option: &'static str,
    text: &str,
) -> Result<u64, ArgumentRefusal> {
    if text.is_empty() || !text.bytes().all(|byte| return byte.is_ascii_digit()) {
        return Err(ArgumentRefusal::OptionNotACount {
            option,
            text: text.to_string(),
        });
    }
    let Ok(count) = text.parse::<u64>() else {
        return Err(ArgumentRefusal::OptionNotACount {
            option,
            text: text.to_string(),
        });
    };
    return Ok(count);
}
