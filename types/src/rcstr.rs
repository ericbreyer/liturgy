use std::{fmt::{Debug, Display}, ops::Deref, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::ArcStr;

/// Shared, reference-counted string slice type for single-threaded contexts.
///
/// NOTE: This is intentionally `Rc<str>` (not `Arc`) per user's request.
/// `Rc<str>` is not `Send` or `Sync`. If a value holding an `RcStr` needs to be
/// sent across threads (for example via Rayon parallel iterators), you'll
/// either need to convert to `Arc<str>` or clone to an owned `String` at the
/// boundary.

#[derive(Serialize, Deserialize)]
pub struct  RcStr(Rc<str>);



impl From<String> for RcStr {
    fn from(s: String) -> Self {
        RcStr(Rc::from(s))
    }
}

impl From<&str> for RcStr {
    fn from(s: &str) -> Self {
        RcStr(Rc::from(s.to_string()))
    }
}

impl Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for RcStr {
    fn clone(&self) -> Self {
        RcStr(self.0.clone())
    }   
}

impl Debug for RcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for RcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RcStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq for RcStr {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<str> for RcStr {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref().eq(other)
    }
}

impl PartialEq<&str> for RcStr {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref().eq(*other)
    }
}

impl From<RcStr> for String {
    fn from(val: RcStr) -> Self {
        val.0.to_string()
    }
}

impl Default for RcStr {
    fn default() -> Self {
        RcStr(Rc::from(""))
    }
}

impl PartialOrd for RcStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RcStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}

impl Eq for RcStr {}

impl From<ArcStr> for RcStr {
    fn from(arc: ArcStr) -> Self {
        RcStr(Rc::from(arc.as_ref().to_string()))
    }
}