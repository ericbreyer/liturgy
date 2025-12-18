use std::fmt::Display;

use anyhow::Result;
use nestify::nest;
use serde::{Deserialize, Serialize};

use crate::{
    Location,
    office_component::{OfficeComponent, OfficeComponentFamily},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vespers {
    pub name: String,
    pub ordo: VespersOrdo,
    pub commemorations: Vec<VespersCommemoration>,
}

impl Display for Vespers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.name)?;
        write!(f, "{}", self.ordo)?;
        for commemoration in &self.commemorations {
            writeln!(
                f,
                "{}",
                commemoration
                    .to_string()
                    .lines()
                    .map(|l| format!("  {l}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VespersContainer<T> {
    pub antiphons: T,
    pub psalms: T,
    pub chapter: T,
    pub hymn: T,
    pub verse: T,
    pub magnificat_antiphon: T,
    pub collect: T,
}

impl<T> OfficeComponent<T> for VespersContainer<T>
where
    T: Clone,
{
    fn into_boxed_slice(self) -> Box<[T]> {
        let v = vec![
            self.antiphons,
            self.psalms,
            self.chapter,
            self.hymn,
            self.verse,
            self.magnificat_antiphon,
            self.collect,
        ];
        v.into_boxed_slice()
    }

    fn from_slice(slice: &[T]) -> Self {
        assert!(
            slice.len() == 7,
            "expected 7 elements for VespersContainer::from_slice, got {}",
            slice.len()
        );
        VespersContainer {
            antiphons: slice[0].clone(),
            psalms: slice[1].clone(),
            chapter: slice[2].clone(),
            hymn: slice[3].clone(),
            verse: slice[4].clone(),
            magnificat_antiphon: slice[5].clone(),
            collect: slice[6].clone(),
        }
    }
}

pub type VespersOrdo = VespersContainer<Location>;
pub type ProperVespersSources = VespersContainer<Option<String>>;
pub type OrdinaryVespersSources = VespersContainer<String>;

impl ProperVespersSources {
    pub fn validate(&self) -> Result<OrdinaryVespersSources> {
        Ok(OrdinaryVespersSources {
            antiphons: self
                .antiphons
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Missing antiphons"))?,
            psalms: self
                .psalms
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Missing psalms"))?,
            chapter: self
                .chapter
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Missing chapter"))?,
            hymn: self
                .hymn
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Missing hymn"))?,
            verse: self
                .verse
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Missing verse"))?,
            magnificat_antiphon: self
                .magnificat_antiphon
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Missing magnificat antiphon"))?,
            collect: self
                .collect
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Missing collect"))?,
        })
    }
}

impl<T> OfficeComponentFamily for VespersContainer<T> {
    const SIZE: usize = 7;
    type LocationType = VespersOrdo;
    type ProperSourceType = ProperVespersSources;
    type OrdinarySourceType = OrdinaryVespersSources;
}

impl Display for VespersOrdo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Vespers:")?;
        // Use a fixed width for the label column so values line up regardless of label
        // length. Include the colon in the label and left-align within the
        // width.
        writeln!(f, "  {:<12} {:?}", "Antiphons:", self.antiphons)?;
        writeln!(f, "  {:<12} {:?}", "Psalms:", self.psalms)?;
        writeln!(f, "  {:<12} {:?}", "Chapter:", self.chapter)?;
        writeln!(f, "  {:<12} {:?}", "Hymn:", self.hymn)?;
        writeln!(f, "  {:<12} {:?}", "Verse:", self.verse)?;
        writeln!(f, "  {:<12} {:?}", "Magnificat:", self.magnificat_antiphon)?;
        writeln!(f, "  {:<12} {:?}", "Collect:", self.collect)?;
        Ok(())
    }
}

nest! {
#[derive(Clone, Debug, Serialize, Deserialize)]*
    pub struct VespersCommemoration {
        pub name: String,
        pub ordo: pub enum VespersCommemorationOrdo {
            FullCommemoration(
        pub struct FullVespersCommemorationOrdo {
                    pub magnificat_antiphon: Location,
                    pub verse: Location,
                    pub collect: Location,
                }),
                SpecialCommemoration(pub Location),
            }
    }
}

impl Display for VespersCommemoration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Commemoration of {}:", self.name)?;
        write!(f, "{}", self.ordo)
    }
}

impl Display for FullVespersCommemorationOrdo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Magnificat Antiphon: {:?}", self.magnificat_antiphon)?;
        writeln!(f, "  Verse: {:?}", self.verse)?;
        writeln!(f, "  Collect: {:?}", self.collect)?;
        Ok(())
    }
}

impl Display for VespersCommemorationOrdo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VespersCommemorationOrdo::FullCommemoration(ordo) => {
                write!(f, "{ordo}")
            }
            VespersCommemorationOrdo::SpecialCommemoration(location) => {
                writeln!(f, "  Collect: {location:?}")
            }
        }
    }
}

impl VespersOrdo {
    pub fn to_full_commemoration(&self) -> VespersCommemorationOrdo {
        VespersCommemorationOrdo::FullCommemoration(FullVespersCommemorationOrdo {
            magnificat_antiphon: self.magnificat_antiphon.clone(),
            verse: self.verse.clone(),
            collect: self.collect.clone(),
        })
    }
}
