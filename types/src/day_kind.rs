use crate::ArcStr;

/// A small, best-effort classification of a day's textual description.
/// This is intentionally lightweight and only extracts the common patterns
/// we need for Ordo resolution (Dominica / Sunday / Feria style names).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DayKind {
    Feast(ArcStr),
    // Sundays (Dominica) now require a number and season string. The
    // optional `week_of_month` captures cases like "Week 2 of August".
    Sunday {
        number: ArcStr,
        season: ArcStr,
        week_of_month: Option<u8>,
    },
    // Feria requires day/week/season strings; `week_of_month` is optional.
    Feria {
        day: ArcStr,
        week: ArcStr,
        season: ArcStr,
        week_of_month: Option<u8>,
    },
    Unknown,
}

impl DayKind {
    /// Parse a human-readable day description into a `DayKind`. This is a
    /// heuristic parser intended to be tolerant of variations.
    #[must_use]
    pub fn from_desc(desc: &str) -> Self {
        let s = desc.trim();
        if s.is_empty() {
            return DayKind::Unknown;
        }

        let lc = s.to_lowercase();
        // Dominica / Sunday patterns
        if lc.starts_with("dominica") || lc.starts_with("dom.") || lc.contains("sunday") {
            // Try to extract a season following 'of' or 'in'
            // We return required number/season strings. If a part isn't
            // present, we use an empty string (the caller should prefer
            // constructing via the helpers below instead of parsing).
            if let Some(pos) = lc.find(" of ") {
                let number = s["dominica".len()..pos].trim().to_string();
                let season = s[pos + 4..].trim().to_string();
                return DayKind::Sunday {
                    number: ArcStr::from(number),
                    season: ArcStr::from(season),
                    week_of_month: None,
                };
            }
            if let Some(pos) = lc.find(" in ") {
                let number = s["dominica".len()..pos].trim().to_string();
                let season = s[pos + 4..].trim().to_string();
                return DayKind::Sunday {
                    number: ArcStr::from(number),
                    season: ArcStr::from(season),
                    week_of_month: None,
                };
            }
            if let Some(space) = s.find(' ') {
                let rest = s[space + 1..].trim().to_string();
                return DayKind::Sunday {
                    number: ArcStr::from(String::new()),
                    season: ArcStr::from(rest),
                    week_of_month: None,
                };
            }
            return DayKind::Sunday {
                number: ArcStr::from(String::new()),
                season: ArcStr::from(String::new()),
                week_of_month: None,
            };
        }

        // Feria patterns (heuristic)
        if lc.starts_with("feria") || lc.starts_with("weekday") || lc.starts_with("monday") {
            // crude split: look for 'of' or 'after' to find season/qualifier
            let mut season = String::new();
            if let Some(pos) = lc.find(" of ") {
                let sfx = s[pos + 4..].trim();
                if !sfx.is_empty() {
                    season = sfx.to_string();
                }
            }
            return DayKind::Feria {
                day: ArcStr::from(s.to_string()),
                week: ArcStr::from(String::new()),
                season: ArcStr::from(season),
                week_of_month: None,
            };
        }

        // Default: treat as a named feast
        DayKind::Feast(ArcStr::from(s.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_dominica_of_advent() {
        let k = DayKind::from_desc("Dominica II of Advent");
        match k {
            DayKind::Sunday {
                number,
                season,
                week_of_month,
            } => {
                assert!(!number.as_ref().is_empty());
                assert_eq!(season.as_ref().to_lowercase(), "advent");
                assert!(week_of_month.is_none());
            }
            _ => panic!("expected Sunday kind"),
        }
    }

    #[test]
    fn parse_dominica_palm() {
        let k = DayKind::from_desc("Dominica Palm Sunday");
        match k {
            DayKind::Sunday { season, .. } => {
                assert_eq!(season.as_ref().to_lowercase(), "palm sunday");
            }
            _ => panic!("expected Sunday kind"),
        }
    }
}
