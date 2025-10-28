use anyhow::{Context, Result};

use crate::Location;

pub trait OfficeComponent<T>: OfficeComponentFamily {
    fn into_boxed_slice(self) -> Box<[T]>;
    fn from_slice(slice: &[T]) -> Self;
}

pub trait OfficeComponentFamily {
    const SIZE: usize;
    type LocationType: Clone + OfficeComponent<Location>;
    type ProperSourceType: Clone + Default + OfficeComponent<Option<String>>;
    type OrdinarySourceType: Clone + Default + OfficeComponent<String>;
}

pub fn populate_defaults<F>(
    proper: F::ProperSourceType,
    ordinary: F::OrdinarySourceType,
) -> F::OrdinarySourceType
where
    F: OfficeComponentFamily,
{
    let proper_box = proper.into_boxed_slice();
    let ordinary_box = ordinary.into_boxed_slice();

    assert!(
        !(proper_box.len() != F::SIZE || ordinary_box.len() != F::SIZE),
        "expected component slices of length {} but got {} and {}",
        F::SIZE,
        proper_box.len(),
        ordinary_box.len()
    );

    let combined: Vec<String> = proper_box
        .into_iter()
        .zip(ordinary_box)
        .map(|(prop_opt, ord)| prop_opt.unwrap_or(ord))
        .collect();

    F::OrdinarySourceType::from_slice(&combined)
}

pub fn map_office_component<T, U, C, K>(comp: C, f: impl FnMut(T) -> Result<U>) -> Result<K>
where
    C: OfficeComponent<T>,
    K: OfficeComponent<U>,
{
    let boxed = comp.into_boxed_slice();
    assert!(
        (boxed.len() == C::SIZE),
        "expected component slice of length {} but got {}",
        C::SIZE,
        boxed.len()
    );
    let vec_u: Result<Vec<U>> = boxed.into_iter().map(f).collect();
    vec_u
        .map(|v| K::from_slice(&v))
        .context("mapping office component failed")
}
