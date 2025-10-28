use std::{
    collections::HashMap,
    env,
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use types::{CommemorationType, ConcuringVespersAction, DayRank62Office, LiturgicalUnit};

use crate::{
    Location,
    office_component::{OfficeComponentFamily, map_office_component, populate_defaults},
    vespers::{
        OrdinaryVespersSources, ProperVespersSources, Vespers, VespersCommemoration,
        VespersCommemorationOrdo, VespersOrdo,
    },
};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OrdoRuleToml {
    id: Option<String>,
    name: Option<String>,
    common: Option<String>,
    vespers: Option<ProperVespersSources>,
    first_vespers: Option<ProperVespersSources>,
}

// Office representation: similar metadata to `OrdoRuleToml` but contains the
// validated/owned `OrdinaryVespersSources` so the repo can retain additional
// office-level metadata in the future.
#[derive(Debug, Clone)]
struct OrdoOffice {
    vespers: OrdinaryVespersSources,
}

pub struct OrdoRepo {
    feasts: HashMap<String, OrdoRuleToml>,
    offices: HashMap<String, OrdoOffice>,
}

impl OrdoRepo {
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let mut propers = HashMap::new();
        let mut offices = HashMap::new();

        // Resolve relative paths against the crate's manifest dir so tests can use
        // relative paths.
        let rules_dir: PathBuf = if dir.as_ref().is_absolute() {
            dir.as_ref().to_path_buf()
        } else {
            let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
            let candidate1 = manifest.join(dir.as_ref());
            let candidate2 = manifest
                .parent()
                .map(|p| p.join(dir.as_ref()))
                .unwrap_or(candidate1.clone());
            // Prefer the first candidate that exists, otherwise use candidate1
            if candidate2.exists() {
                candidate2
            } else {
                candidate1
            }
        };
        eprintln!("OrdoRepo: loading rules from {}", rules_dir.display());

        let Ok(subdirs) = fs::read_dir(rules_dir.join("propers")) else {
            bail!(
                "failed to read propers subdirectory in {}",
                rules_dir.display()
            );
        };

        propers.extend(
            subdirs
                .into_iter()
                .flatten()
                .map(|s| s.path())
                .inspect(|p| eprintln!("OrdoRepo: processing proper dir {}", p.display()))
                .filter(|s| s.is_dir())
                .map(fs::read_dir)
                .filter_map(Result::ok)
                .flat_map(ReadDir::flatten)
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
                .map(|p| fs::read_to_string(p.clone()).map(|s| (p, s)))
                .inspect(|r| {
                    if let Err(err) = r {
                        panic!("OrdoRepo: failed to read proper file: {err}");
                    }
                })
                .filter_map(Result::ok)
                .map(|(p, s)| toml::from_str::<OrdoRuleToml>(&s).map(|r| (p, r)))
                .inspect(|r| {
                    if let Err(err) = r {
                        panic!("OrdoRepo: failed to parse proper file: {err}");
                    }
                })
                .filter_map(Result::ok)
                .map(|(p, r)| {
                    (
                        slug(p.file_stem().and_then(|osstr| osstr.to_str()).unwrap()),
                        r,
                    )
                }),
        );

        let Ok(entries) = fs::read_dir(rules_dir.join("offices")) else {
            bail!(
                "failed to read offices subdirectory in {}",
                rules_dir.display()
            );
        };
        offices.extend(
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
                .map(fs::read_to_string)
                .inspect(|r| {
                    if let Err(err) = r {
                        eprintln!("OrdoRepo: failed to read proper file: {err}");
                    }
                })
                .filter_map(Result::ok)
                .map(|s| toml::from_str::<OrdoRuleToml>(&s))
                .inspect(|r| {
                    if let Err(err) = r {
                        eprintln!("OrdoRepo: failed to parse proper file: {err}");
                    }
                })
                .filter_map(Result::ok)
                .filter_map(|r| (r.id.clone().or_else(|| r.name.clone()).map(|id| (id, r))))
                .map(|(id, r)| (id, r.vespers.and_then(|v| v.validate().ok())))
                .inspect(|(id, v)| {
                    if v.is_none() {
                        eprintln!("OrdoRepo: office {id} missing or invalid vespers table");
                    }
                })
                .filter_map(|(id, v)| v.map(|v| (id, v)))
                .map(|(id, v)| (slug(&id), OrdoOffice { vespers: v })),
        );

        Ok(OrdoRepo {
            feasts: propers,
            offices,
        })
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
                |o| o.first_vespers.clone()
            } else {
                |o| o.vespers.clone()
            },
            |o| o.vespers.clone(),
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
        struct TitleFlags {
            martyr: bool,
            virgin: bool,
            confessor: bool,
            bishop: bool,
            pope: bool,
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
        "".to_string()
    }

    fn retrieve_components<F: OfficeComponentFamily>(
        &self,
        day: &LiturgicalUnit<types::DayRank62>,
        season: &str,
        octave: Option<&str>,
        prop_comp_map: fn(&OrdoRuleToml) -> Option<F::ProperSourceType>,
        ord_comp_map: fn(&OrdoOffice) -> F::OrdinarySourceType,
    ) -> Result<(F::LocationType, Vec<String>)> {
        let feast_keys = get_feast_keys(day, season, octave);

        let (proper, common) = feast_keys
            .clone()
            .into_iter()
            .find_map(|fk| {
                self.feasts
                    .get(&fk)
                    .map(|r| (prop_comp_map(r), r.common.as_deref()))
            })
            .unwrap_or((None, None));

        // Ensure `common` is owned so we can safely pass a reference below without
        // returning a reference to a temporary.
        let common = common
            .map(|s| s.to_string())
            .unwrap_or_else(|| Self::obtain_common(day));

        // office: use the canonical office kind (Sunday, Feastial, Semifestial,
        // Ordinary, Ferial)
        let office_key = match day.rank.office {
            DayRank62Office::Sunday => "office-sunday".to_string(),
            DayRank62Office::Feastial => "office-feastial".to_string(),
            DayRank62Office::Semifestial => "office-semifestial".to_string(),
            DayRank62Office::Ordinary => "office-ordinary".to_string(),
            DayRank62Office::Ferial => "office-ferial".to_string(),
        };

        // retrieve office; it must exist and have a vespers table
        let office_vespers =
            ord_comp_map(self.offices.get(&office_key).unwrap_or_else(|| {
                panic!("office {office_key} must exist and have vespers table")
            }));

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
    fn vespers_location(
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
            .filter(|(_, ctype)| {
                *ctype == types::CommemorationType::LaudsAndVespers
                    
            })
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
