use std::{
    fmt::{Debug, Display},
    ops::Deref,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::RcStr;

/// Shared, reference-counted string slice type for single-threaded contexts.
///
/// NOTE: This is intentionally `Arc<str>` (not `AArc`) per user's request.
/// `Arc<str>` is not `Send` or `Sync`. If a value holding an `ArcStr` needs to
/// be sent across threads (for example via Rayon parallel iterators), you'll
/// either need to convert to `AArc<str>` or clone to an owned `String` at the
/// boundary.

#[derive(Serialize, Deserialize)]
pub struct ArcStr(Arc<str>);

impl From<String> for ArcStr {
    fn from(s: String) -> Self {
        ArcStr(Arc::from(s))
    }
}

impl From<&str> for ArcStr {
    fn from(s: &str) -> Self {
        ArcStr(Arc::from(s.to_string()))
    }
}

impl Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for ArcStr {
    fn clone(&self) -> Self {
        ArcStr(self.0.clone())
    }
}

impl Debug for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ArcStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq for ArcStr {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<str> for ArcStr {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref().eq(other)
    }
}

impl PartialEq<&str> for ArcStr {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref().eq(*other)
    }
}

impl From<ArcStr> for String {
    fn from(val: ArcStr) -> Self {
        val.0.to_string()
    }
}

impl Default for ArcStr {
    fn default() -> Self {
        ArcStr(Arc::from(""))
    }
}

impl PartialOrd for ArcStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArcStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}

impl Eq for ArcStr {}

impl From<RcStr> for ArcStr {
    fn from(rc: RcStr) -> Self {
        ArcStr(Arc::from(rc.as_ref().to_string()))
    }
}
