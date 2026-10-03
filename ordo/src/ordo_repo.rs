use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use types::{CommemorationType, ConcuringVespersAction, DayRank62Office, LiturgicalUnit};

use crate::{
    Location,
    office_component::{OfficeComponentFamily, map_office_component, populate_defaults},
    rule_provider::{RuleProvider, office_key_for_rank},
    toml_rule_provider::TomlRuleProvider,
    title_classification::TitleClassifier,
    concurring_vespers::ConcurringVespersResolution,
    location_formats,
    vespers::{
        OrdinaryVespersSources, ProperVespersSources, Vespers, VespersCommemoration,
        VespersCommemorationOrdo, VespersOrdo,
    },
};

#[derive(Debug, Deserialize)]
struct ProperRuleToml {
    id: Option<String>,
    name: Option<String>,
    common: Option<String>,
    vespers: Option<ProperVespersSources>,
    first_vespers: Option<ProperVespersSources>,
}

#[derive(Debug, Deserialize)]
struct OfficeRuleToml {
    vespers: Option<OrdinaryVespersSources>,
}

/// The main ordo repository, decoupled from any specific rule storage implementation.
///
/// Uses a RuleProvider abstraction to support different data sources (TOML files,
/// databases, APIs, etc.) without coupling the business logic to storage details.
pub struct OrdoRepo {
    rules: Box<dyn RuleProvider>,
}

impl OrdoRepo {
    /// Create an OrdoRepo with a custom RuleProvider.
    pub fn with_provider(rules: Box<dyn RuleProvider>) -> Self {
        OrdoRepo { rules }
    }

    /// Load rules from the filesystem using the TomlRuleProvider.
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let provider = TomlRuleProvider::load_from_dir(dir)?;
        Ok(OrdoRepo {
            rules: Box::new(provider),
        })
    }

    /// Parse a proper rule from raw TOML string.
    fn parse_proper_rule(toml_str: &str) -> Result<ProperRuleToml> {
        toml::from_str(toml_str)
            .context("failed to parse proper rule TOML")
    }

    /// Parse an office rule from raw TOML bytes.
    fn parse_office_rule(toml_bytes: &[u8]) -> Result<OfficeRuleToml> {
        let toml_str = std::str::from_utf8(toml_bytes)
            .context("office rule TOML is not valid UTF-8")?;
        toml::from_str(toml_str)
            .context("failed to parse office rule TOML")
    }

    fn retrieve_vespers_components(
        &self,
        day: &types::LiturgicalUnit<types::DayRank62>,
        season: &str,
        octave: Option<&str>,
        first_vespers: bool,
    ) -> Result<(VespersOrdo, Vec<String>)> {
        self.retrieve_components::<crate::vespers::VespersContainer<String>>(
            day,
            season,
            octave,
            if first_vespers {
                |o: &ProperRuleToml| o.first_vespers.clone()
            } else {
                |o: &ProperRuleToml| o.vespers.clone()
            },
            |o: &OfficeRuleToml| o.vespers.clone().unwrap_or_default(),
        )
        .context(format!(
            "retrieving vespers components for day {} failed",
            day.desc.as_ref()
        ))
    }

    fn retrieve_vespers_commemoration_components(
        &self,
        day: &LiturgicalUnit<types::DayRank62>,
        season: &str,
        octave: Option<&str>,
    ) -> Result<(VespersCommemorationOrdo, Vec<String>)> {
        let x = self
            .retrieve_vespers_components(day, season, octave, false)
            .context(format!(
                "retrieving vespers commemoration components for day {} failed",
                day.desc.as_ref()
            ))?;
        Ok((x.0.to_full_commemoration(), x.1))
    }

    fn obtain_common(day: &LiturgicalUnit<types::DayRank62>) -> String {
        let titles = day
            .titles
            .iter()
            .map(|t| t.to_lowercase())
            .collect::<Vec<_>>();
        // Use an explicit struct with named boolean fields for clarity.
        #[derive(Default, Copy, Clone)]
        #[allow(clippy::struct_excessive_bools)]
        struct TitleFlags {
            martyr: bool,
            virgin: bool,
            confessor: bool,
            bishop: bool,
            pope: bool,
            apostle: bool,
        }

        impl TitleFlags {
            fn from_iter<I, S>(it: I) -> Self
            where
                I: IntoIterator<Item = S>,
                S: AsRef<str>,
            {
                let mut f = TitleFlags::default();
                for s in it {
                    let sref = s.as_ref();
                    if sref.contains("martyr") {
                        f.martyr = true;
                    }
                    if sref.contains("virgin") {
                        f.virgin = true;
                    }
                    if sref.contains("confessor") | sref.contains("abbot") {
                        f.confessor = true;
                    }
                    if sref.contains("bishop") {
                        f.bishop = true;
                    }
                    if sref.contains("pope") {
                        f.pope = true;
                    }
                    if sref.contains("apostle") {
                        f.apostle = true;
                    }
                }
                f
            }
        }

        let flags = TitleFlags::from_iter(titles.iter());
        let bvm = day.desc.to_lowercase().contains("blessed virgin mary");
        if bvm {
            return "the Blessed Virgin Mary".to_string();
        }
        if flags.virgin {
            return "Virgins".to_string();
        }

        if flags.martyr && !flags.bishop {
            return "Martyrs".to_string();
        }
        if flags.bishop || flags.pope {
            return "Confessor Bishops".to_string();
        }
        if flags.confessor && !flags.bishop {
            return "Confessors (Non-Bishop)".to_string();
        }
        if flags.apostle {
            return "Apostles".to_string();
        }
        String::new()
    }

    fn retrieve_components<F: OfficeComponentFamily>(
        &self,
        day: &LiturgicalUnit<types::DayRank62>,
        season: &str,
        octave: Option<&str>,
        prop_comp_map: fn(&ProperRuleToml) -> Option<F::ProperSourceType>,
        ord_comp_map: fn(&OfficeRuleToml) -> F::OrdinarySourceType,
    ) -> Result<(F::LocationType, Vec<String>)> {
        let feast_keys = get_feast_keys(day, season, octave);

        let (proper, common) = if let Some(matched) = self.rules.get_proper_rule(&feast_keys)? {
            let toml_str = std::str::from_utf8(&matched.toml_bytes)
                .with_context(|| format!("proper rule '{}' is not valid UTF-8", matched.id))?;
            let rule = Self::parse_proper_rule(toml_str)
                .with_context(|| format!("parsing proper rule '{}' failed", matched.id))?;
            (prop_comp_map(&rule), matched.common.or(rule.common))
        } else {
            (Option::<F::ProperSourceType>::None, None)
        };

        // Ensure `common` is owned so we can safely pass a reference below without
        // returning a reference to a temporary.
        let common = common.unwrap_or_else(|| Self::obtain_common(day));

        // Get the office key for this day's rank
    let office_key = office_key_for_rank(&day.rank.office);

        // Retrieve the office rule from the provider
        let office_toml_bytes = self.rules.get_office_rule(office_key)
            .context(format!("office rule '{}' not found", office_key))?;
        let office_rule = Self::parse_office_rule(&office_toml_bytes)?;
        let office_vespers = ord_comp_map(&office_rule);

        get_components_proper_and_ordinary_generic::<F>(
            proper.unwrap_or_default(),
            office_vespers,
            common.as_str(),
            season,
            octave,
        )
        .map(|loc| (loc, feast_keys))
        .context(format!(
            "retrieving components for day {} failed",
            day.desc.as_ref()
        ))
    }
}

fn slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn slugs(name: &str) -> Vec<String> {
    let mut slugs = Vec::new();
    slugs.push(slug(name));
    if let Some(short) = name.split(',').next() {
        let short = short.trim();
        if !short.is_empty() && short != name {
            slugs.push(slug(short));
        }
    }
    slugs
}

impl crate::OrdoRules for OrdoRepo {
    fn vespers_location_62(
        &self,
        day: &crate::DayDescription<crate::DayRank62>,
    ) -> Result<(Vespers, Vec<String>)> {
        let (day_used, day_commemorated) = if let Some((cv, ca)) = &day.concuring_vespers {
            match ca {
                ConcuringVespersAction::Use => (cv, None),
                ConcuringVespersAction::Commemorate => (&day.day, Some(cv)),
                ConcuringVespersAction::UseCommemorateSelf => (cv, Some(&day.day)),
            }
        } else {
            (&day.day, None)
        };

        let is_first_vespers = day.day.desc != day_used.desc;

        let (ordo, fks) = self
            .retrieve_vespers_components(
                day_used,
                day.season.as_ref(),
                day.underlying_octave.as_deref(),
                is_first_vespers,
            )
            .context(format!(
                "building vespers location for day {} failed",
                day_used.desc.as_ref()
            ))?;

        let desc = if is_first_vespers {
            format!("First Vespers of {}", day_used.desc.as_ref())
        } else if day.day.rank.has_first_vespers() {
            format!("Second Vespers of {}", day.day.desc.as_ref())
        } else {
            format!("Vespers of {}", day.day.desc.as_ref())
        };

        // Build the commemorations using iterator combinators only (no
        // mutables). If we're in first vespers or there's no concuring
        // vespers to be commemorated, include the day's own LaudsAndVespers
        // commemorations; otherwise skip them. Then append the optional
        // concuring day if present.
        let include_day_commemorations = is_first_vespers || day_commemorated.is_none();

        let commemorations: Vec<VespersCommemoration> = day
            .commemorations
            .clone()
            .into_iter()
            .filter(|(_, ctype)| *ctype == types::CommemorationType::LaudsAndVespers)
            .filter(move |_| include_day_commemorations)
            .chain(
                day_commemorated.map(|dc| (dc.clone(), types::CommemorationType::LaudsAndVespers)),
            )
            .map(|(c, ctype)| {
                Ok((
                    match ctype {
                        CommemorationType::LaudsAndVespers => self
                            .retrieve_vespers_commemoration_components(
                                &c,
                                day.season.as_ref(),
                                day.underlying_octave.as_deref(),
                            )
                            .context(format!(
                                "building vespers commemoration for day {} failed",
                                c.desc.as_ref()
                            ))
                            .map(|(o, _)| o)?,
                        CommemorationType::PeterAndPaulSpecial => {
                            VespersCommemorationOrdo::SpecialCommemoration(Location::Proper)
                        }
                        _ => unreachable!(),
                    },
                    c.desc.to_string(),
                ))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|(ordo, name)| VespersCommemoration { name, ordo })
            .collect();

        Ok((
            Vespers {
                ordo,
                name: desc,
                commemorations,
            },
            fks,
        ))
    }
}

fn get_feast_keys(
    day: &types::LiturgicalUnit<types::DayRank62>,
    season: &str,
    octave: Option<&str>,
) -> Vec<String> {
    let mut feast_keys: Vec<String> = Vec::new();
    let desc = day.desc.as_ref();
    feast_keys.extend(slugs(desc));

    match day.rank.office {
        types::DayRank62Office::Sunday => {
            if !season.is_empty() {
                feast_keys.extend(slugs(&format!("Dominica of {season}")));
            }
        }
        types::DayRank62Office::Ferial => {
            if !season.is_empty() {
                feast_keys.extend(slugs(&format!("Feria of {season}")));
            }
        }
        types::DayRank62Office::Feastial
        | types::DayRank62Office::Semifestial
        | types::DayRank62Office::Ordinary => {
            if let Some(octave) = octave {
                feast_keys.extend(slugs(&format!("Octave of {octave}")));
            }
        }
    }

    feast_keys
}

/// Generic helper: given a "proper" component with Option<T> fields and an
/// "ordinary" component with T fields, populate defaults and map each field
/// through `mapper` to produce the target component type.
pub fn get_components_proper_and_ordinary_generic<F: OfficeComponentFamily>(
    prop: F::ProperSourceType,
    ordinary: F::OrdinarySourceType,
    common: &str,
    season: &str,
    octave: Option<&str>,
) -> Result<F::LocationType> {
    let merged = populate_defaults::<F>(prop, ordinary);
    map_office_component(merged, |t| {
        into_location_with_inherited(&t, common, season, octave)
    })
}

// like into_location but allows a higher-precedence inherited_common to be
// provided; if the token vector contains "Common" and the local rule does
// not provide a name, the inherited common will be used.
fn into_location_with_inherited(
    loc: &str,
    common_name: &str,
    season: &str,
    octave: Option<&str>,
) -> Result<Location> {
    Ok(match loc {
        "Proper" => Location::Proper,
        "Psalter" => Location::Psalter,
        "Common" => Location::Common(common_name.to_string()),
        "Feria" | "Ferial" | "Ordinary" => {
            if let Some(season) = octave {
                Location::Octave(season.to_string())
            } else {
                Location::Ordinary(season.to_string())
            }
        }

        x if x.starts_with("Octave of ") => {
            let name = x.trim_start_matches("Octave of ").trim();
            Location::Octave(name.to_string())
        }
        x if x.starts_with("Common of ") => {
            let name = x.trim_start_matches("Common of ").trim();
            Location::Common(name.to_string())
        }
        x if x.starts_with("Sunday") => {
            let name = x.trim_start_matches("Sunday").trim();
            let name_opt = if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            };
            Location::Sunday(name_opt)
        }
        _ => bail!("unrecognized location token: {}", loc),
    })
}

#[cfg(test)]
mod proper_resolution_tests {
    use super::*;
    use crate::rule_provider::MatchedProperRule;

    const OFFICE: &str = r#"
[vespers]
antiphons = "Psalter"
psalms = "Psalter"
chapter = "Common"
hymn = "Ferial"
verse = "Common"
magnificat_antiphon = "Ferial"
collect = "Common"
"#;

    struct FixtureProvider {
        proper: Option<MatchedProperRule>,
    }

    impl RuleProvider for FixtureProvider {
        fn get_proper_rule(&self, _: &[String]) -> Result<Option<MatchedProperRule>> {
            Ok(self.proper.clone())
        }

        fn get_office_rule(&self, key: &str) -> Result<Vec<u8>> {
            assert_eq!(key, "office-feastial");
            Ok(OFFICE.as_bytes().to_vec())
        }

        fn list_proper_keys(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        fn list_office_keys(&self) -> Result<Vec<String>> {
            Ok(vec!["office-feastial".to_string()])
        }
    }

    fn day() -> LiturgicalUnit<types::DayRank62> {
        LiturgicalUnit {
            desc: "Fixture Saint".into(),
            rank: types::DayRank62::new(types::DayRank62Office::Feastial, "I"),
            date: "2026-12-25".parse().unwrap(),
            color: "White".into(),
            day_kind: types::DayKind::Feast("Fixture Saint".into()),
            titles: vec!["Confessor".into()],
        }
    }

    fn repo(proper: Option<&str>) -> OrdoRepo {
        OrdoRepo::with_provider(Box::new(FixtureProvider {
            proper: proper.map(|raw| MatchedProperRule {
                id: "fixture-proper".to_string(),
                common: None,
                toml_bytes: raw.as_bytes().to_vec(),
            }),
        }))
    }

    #[test]
    fn partial_proper_overrides_only_specified_components() {
        let repo = repo(Some(r#"
common = "Fixture Common"
[vespers]
antiphons = "Proper"
collect = "Proper"
"#));
        let (office, _) = repo.retrieve_vespers_components(&day(), "Advent", None, false).unwrap();
        assert_eq!(office.antiphons, Location::Proper);
        assert_eq!(office.collect, Location::Proper);
        assert_eq!(office.psalms, Location::Psalter);
        assert_eq!(office.chapter, Location::Common("Fixture Common".to_string()));
        assert_eq!(office.verse, Location::Common("Fixture Common".to_string()));
        assert_eq!(office.hymn, Location::Ordinary("Advent".to_string()));
    }

    #[test]
    fn first_and_ordinary_vespers_use_their_own_proper_sections() {
        let repo = repo(Some(r#"
[first_vespers]
antiphons = "Proper"
[vespers]
collect = "Proper"
"#));
        let (first, _) = repo.retrieve_vespers_components(&day(), "Christmas", None, true).unwrap();
        let (ordinary, _) = repo.retrieve_vespers_components(&day(), "Christmas", None, false).unwrap();
        assert_eq!(first.antiphons, Location::Proper);
        assert_eq!(first.collect, Location::Common("Confessors (Non-Bishop)".to_string()));
        assert_eq!(ordinary.antiphons, Location::Psalter);
        assert_eq!(ordinary.collect, Location::Proper);
    }

    #[test]
    fn missing_match_or_office_section_keeps_ordinary_fallbacks() {
        for proper in [None, Some("common = 'Fixture Common'"), Some("[vespers]\ncollect = 'Proper'")] {
            let repo = repo(proper);
            let (office, _) = repo.retrieve_vespers_components(&day(), "Advent", Some("Christmas"), true).unwrap();
            assert_eq!(office.antiphons, Location::Psalter);
            assert_eq!(office.hymn, Location::Octave("Christmas".to_string()));
            let expected_common = if proper == Some("common = 'Fixture Common'") {
                "Fixture Common"
            } else {
                "Confessors (Non-Bishop)"
            };
            assert_eq!(office.collect, Location::Common(expected_common.to_string()));
        }
    }

    #[test]
    fn invalid_proper_data_reports_the_rule_instead_of_silently_falling_back() {
        for raw in [vec![0xff], b"[vespers".to_vec()] {
            let repo = OrdoRepo::with_provider(Box::new(FixtureProvider {
                proper: Some(MatchedProperRule {
                    id: "broken-proper".to_string(),
                    common: None,
                    toml_bytes: raw,
                }),
            }));
            let error = repo.retrieve_vespers_components(&day(), "Advent", None, false).unwrap_err();
            assert!(format!("{error:#}").contains("broken-proper"));
        }
    }

    #[test]
    fn provider_common_takes_precedence_over_embedded_and_inferred_common() {
        let repo = OrdoRepo::with_provider(Box::new(FixtureProvider {
            proper: Some(MatchedProperRule {
                id: "fixture-proper".to_string(),
                common: Some("Provider Common".to_string()),
                toml_bytes: b"common = 'Embedded Common'\n[vespers]\nchapter = 'Common'".to_vec(),
            }),
        }));
        let (office, _) = repo.retrieve_vespers_components(&day(), "Advent", None, false).unwrap();
        assert_eq!(office.chapter, Location::Common("Provider Common".to_string()));
        assert_eq!(office.collect, Location::Common("Provider Common".to_string()));
    }

    #[test]
    fn canonical_christmas_rule_supplies_first_and_second_vespers() {
        let repo = OrdoRepo::load_from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("rules")).unwrap();
        let mut christmas = day();
        christmas.desc = "The Nativity of our Lord Jesus Christ".into();
        let (first, _) = repo.retrieve_vespers_components(&christmas, "Christmas", None, true).unwrap();
        let (second, _) = repo.retrieve_vespers_components(&christmas, "Christmas", None, false).unwrap();
        assert_eq!(first.psalms, Location::Sunday(Some("w.116".to_string())));
        assert_eq!(second.psalms, Location::Proper);
        assert_eq!(first.antiphons, Location::Proper);
        assert_eq!(second.antiphons, Location::Proper);
    }
}
