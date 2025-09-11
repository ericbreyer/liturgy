use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::DateRule;

// ---------- Display & FromStr ----------
impl fmt::Display for DateRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateRule::Easter => write!(f, "Easter"),
            DateRule::Fixed { month, day } => write!(f, "{}/{}", month, day),
            DateRule::OffsetDays { rule, offset } => write!(f, "({}) + {} days", rule, offset),
            DateRule::OffsetSundays { rule, offset } => {
                write!(f, "({}) + {} Sundays", rule, offset)
            }
            DateRule::PreviousYear(rule) => write!(f, "{}", rule),
            DateRule::NextYear(rule) => write!(f, "{}", rule),
            DateRule::SundayBetweenOrFallback {
                start,
                end,
                fallback,
            } => {
                write!(
                    f,
                    "Sunday Between ({}) and ({}) or ({})",
                    start, end, fallback
                )
            }
            DateRule::LeapYearConditional {
                leap_year_rule,
                non_leap_year_rule,
            } => {
                write!(
                    f,
                    "({}) in leap year else ({})",
                    leap_year_rule, non_leap_year_rule
                )
            }
            DateRule::AvoidSunday { rule } => {
                write!(f, "({}) (transfered on sundays)", rule)
            }
            DateRule::DivinoAfflatuAnticipation => {
                write!(f, "(DivinoAfflatuAnticipation)")
            }
        }
    }
}

fn parse_three_args(s: &str) -> Result<Vec<&str>, String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);

    if parts.len() != 3 {
        return Err(format!("Expected 3 arguments, got {}", parts.len()));
    }

    Ok(parts.into_iter().map(|s| s.trim()).collect())
}

fn parse_two_args(s: &str) -> Result<Vec<&str>, String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);

    if parts.len() != 2 {
        return Err(format!("Expected 2 arguments, got {}", parts.len()));
    }

    Ok(parts.into_iter().map(|s| s.trim()).collect())
}

impl FromStr for DateRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s == "Easter" {
            return Ok(DateRule::Easter);
        }

        // Fixed(month, day)
        if let Some(rest) = s.strip_prefix("Fixed(").and_then(|s| s.strip_suffix(")")) {
            let mut parts = rest.split(',');
            let month: u8 = parts
                .next()
                .ok_or("Missing month")?
                .parse()
                .map_err(|_| "Invalid month")?;
            let day: u8 = parts
                .next()
                .ok_or("Missing day")?
                .parse()
                .map_err(|_| "Invalid day")?;

            if month == 0 || month > 12 {
                return Err(format!("Invalid month: {}", month));
            }
            if day == 0 || day > 31 {
                return Err(format!("Invalid day: {}", day));
            }

            return Ok(DateRule::Fixed { month, day });
        }

        // OffsetDays(...) and OffsetSundays(...)
        for prefix in &["OffsetDays(", "OffsetSundays("] {
            if let Some(rest) = s.strip_prefix(prefix).and_then(|s| s.strip_suffix(")")) {
                let mut depth = 0;
                let mut split_index = None;
                for (i, c) in rest.char_indices() {
                    match c {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        ',' if depth == 0 => {
                            split_index = Some(i);
                            break;
                        }
                        _ => {}
                    }
                }
                if let Some(i) = split_index {
                    let (rule_str, offset_str) = rest.split_at(i);
                    let offset_str = &offset_str[1..]; // skip comma
                    let rule = rule_str
                        .parse()
                        .map_err(|e| format!("Invalid rule: {e:?}"))?;
                    let offset: i32 = offset_str.trim().parse().map_err(|_| "Invalid offset")?;
                    if offset == 0 {
                        return Err("Offset cannot be zero".to_string());
                    }
                    return Ok(match *prefix {
                        "OffsetDays(" => DateRule::OffsetDays {
                            rule: Box::new(rule),
                            offset,
                        },
                        "OffsetSundays(" => DateRule::OffsetSundays {
                            rule: Box::new(rule),
                            offset,
                        },
                        _ => unreachable!(),
                    });
                }
            }
        }
        // PreviousYear(...)
        if let Some(rest) = s
            .strip_prefix("PreviousYear(")
            .and_then(|s| s.strip_suffix(")"))
        {
            let rule = rest
                .parse()
                .map_err(|e| format!("Invalid PreviousYear rule: {e:?}"))?;
            return Ok(DateRule::PreviousYear(Box::new(rule)));
        }

        // NextYear(...)
        if let Some(rest) = s
            .strip_prefix("NextYear(")
            .and_then(|s| s.strip_suffix(")"))
        {
            let rule = rest
                .parse()
                .map_err(|e| format!("Invalid NextYear rule: {e:?}"))?;
            return Ok(DateRule::NextYear(Box::new(rule)));
        }

        // SundayBetweenOrFallback(start, end, fallback)
        if let Some(rest) = s
            .strip_prefix("SundayBetweenOrFallback(")
            .and_then(|s| s.strip_suffix(")"))
        {
            let parts = parse_three_args(rest)?;
            let start = parts[0]
                .parse()
                .map_err(|e| format!("Invalid start rule: {e:?}"))?;
            let end = parts[1]
                .parse()
                .map_err(|e| format!("Invalid end rule: {e:?}"))?;
            let fallback = parts[2]
                .parse()
                .map_err(|e| format!("Invalid fallback rule: {e:?}"))?;
            return Ok(DateRule::SundayBetweenOrFallback {
                start: Box::new(start),
                end: Box::new(end),
                fallback: Box::new(fallback),
            });
        }

        // LeapYearConditional(leap_year_rule, non_leap_year_rule)
        if let Some(rest) = s
            .strip_prefix("LeapYearConditional(")
            .and_then(|s| s.strip_suffix(")"))
        {
            let parts = parse_two_args(rest)?;
            let leap_year_rule = parts[0]
                .parse()
                .map_err(|e| format!("Invalid leap year rule: {e:?}"))?;
            let non_leap_year_rule = parts[1]
                .parse()
                .map_err(|e| format!("Invalid non-leap year rule: {e:?}"))?;
            return Ok(DateRule::LeapYearConditional {
                leap_year_rule: Box::new(leap_year_rule),
                non_leap_year_rule: Box::new(non_leap_year_rule),
            });
        }

        // AvoidSunday(rule)
        if let Some(rest) = s
            .strip_prefix("AvoidSunday(")
            .and_then(|s| s.strip_suffix(")"))
        {
            let rule = rest
                .parse()
                .map_err(|e| format!("Invalid rule in AvoidSunday: {e:?}"))?;
            return Ok(DateRule::AvoidSunday {
                rule: Box::new(rule),
            });
        }

        // DivinoAfflatuAnticipation
        if s == "DivinoAfflatuAnticipation" {
            return Ok(DateRule::DivinoAfflatuAnticipation);
        }

        Err(format!("Could not parse DateRule from '{}'", s))
    }
}

// Serialize and Deserialize for DateRule
impl Serialize for DateRule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DateRule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
#[cfg(test)]
mod test {
    use test_case::test_case;

    use super::*;

    // ---------- Base rules ----------
    fn base_rules() -> Vec<DateRule> {
        vec![
            DateRule::Easter,
            DateRule::Fixed { month: 1, day: 1 },
            DateRule::Fixed { month: 12, day: 31 },
        ]
    }

    // ---------- Generate depth-1 OffsetDays/OffsetSundays ----------
    fn depth1_rules() -> Vec<DateRule> {
        let mut rules = vec![];
        for base in base_rules() {
            rules.push(DateRule::OffsetDays {
                rule: Box::new(base.clone()),
                offset: 3,
            });
            rules.push(DateRule::OffsetSundays {
                rule: Box::new(base.clone()),
                offset: -2,
            });
        }
        rules
    }

    // ---------- Generate depth-2 nested rules ----------
    fn depth2_rules() -> Vec<DateRule> {
        let mut rules = vec![];
        for r1 in depth1_rules() {
            for _r2 in base_rules() {
                // Nested OffsetDays
                rules.push(DateRule::OffsetDays {
                    rule: Box::new(r1.clone()),
                    offset: 1,
                });
                // Nested OffsetSundays
                rules.push(DateRule::OffsetSundays {
                    rule: Box::new(r1.clone()),
                    offset: -1,
                });
            }
        }
        rules
    }

    // ---------- Combine all rules ----------
    fn all_test_rules() -> Vec<DateRule> {
        let mut rules = vec![];
        rules.extend(base_rules());
        rules.extend(depth1_rules());
        rules.extend(depth2_rules());
        rules
    }


    // ---------- Hardcoded display tests ----------
    #[test_case(DateRule::Easter, "Easter"; "display_easter")]
    #[test_case(DateRule::Fixed { month: 1, day: 1 }, "1/1"; "display_fixed_jan_1")]
    #[test_case(DateRule::Fixed { month: 2, day: 14 }, "2/14"; "display_fixed_feb_14")]
    #[test_case(DateRule::Fixed { month: 7, day: 4 }, "7/4"; "display_fixed_july_4")]
    #[test_case(DateRule::Fixed { month: 12, day: 25 }, "12/25"; "display_fixed_dec_25")]
    #[test_case(DateRule::OffsetDays { rule: Box::new(DateRule::Easter), offset: 0 }, "(Easter) + 0 days"; "display_offsetdays_easter_0")]
    #[test_case(DateRule::OffsetDays { rule: Box::new(DateRule::Fixed { month: 12, day: 25 }), offset: 5 }, "(12/25) + 5 days"; "display_offsetdays_dec_25_5")]
    #[test_case(DateRule::OffsetSundays { rule: Box::new(DateRule::Easter), offset: -1 }, "(Easter) + -1 Sundays"; "display_offsetsundays_easter_minus1")]
    #[test_case(DateRule::OffsetSundays { rule: Box::new(DateRule::Fixed { month: 7, day: 4 }), offset: 2 }, "(7/4) + 2 Sundays"; "display_offsetsundays_july4_2")]
    #[test_case(
        DateRule::OffsetDays { rule: Box::new(DateRule::OffsetSundays { rule: Box::new(DateRule::Fixed { month: 12, day: 25 }), offset: 2 }), offset: -3 },
        "((12/25) + 2 Sundays) + -3 days";
        "display_nested_offset"
    )]
    fn test_display_hardcoded(rule: DateRule, expected: &str) {
        assert_eq!(
            rule.to_string(),
            expected,
            "Display mismatch for {:?}",
            rule
        );
    }

    // ---------- Invalid parse tests ----------
    #[test_case(""; "empty")]
    #[test_case("Fixed()"; "fixed_empty")]
    #[test_case("Fixed(0,10)"; "fixed_zero_month")]
    #[test_case("Fixed(13,32)"; "fixed_overflow")]
    #[test_case("OffsetDays(Easter)"; "offsetdays_missing")]
    #[test_case("OffsetSundays(Fixed(12,25))"; "offsetsundays_missing")]
    #[test_case("UnknownRule"; "unknown")]
    #[test_case("OffsetDays(Easter, abc)"; "offset_non_numeric")]
    fn test_parse_invalid(input: &str) {
        let result: Result<DateRule, _> = input.parse();
        assert!(result.is_err(), "Expected error parsing '{}'", input);
    }

    // ---------- Nested parse example ----------
    #[test]
    fn test_nested_parse_example() {
        let sin = "OffsetDays(OffsetSundays(Fixed(12,25), 2), -3)";
        let sout = "((12/25) + 2 Sundays) + -3 days";
        let rule: DateRule = sin.parse().expect("Failed to parse nested rule");

        let expected = DateRule::OffsetDays {
            rule: Box::new(DateRule::OffsetSundays {
                rule: Box::new(DateRule::Fixed { month: 12, day: 25 }),
                offset: 2,
            }),
            offset: -3,
        };

        assert_eq!(rule, expected);
        assert_eq!(rule.to_string(), sout);
    }
}
