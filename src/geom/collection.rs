// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Collection of heterogeous geometries.
//!

use crate::{CRS, G, GTrait, MyError, srid::SRID};
use core::fmt;
use geos::{ConstGeometry, Geom, Geometry};
use std::slice::Iter;
use tracing::{error, warn};

/// Collection of mixed geometries.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Geometries {
    items: Vec<G>,
    srid: SRID,
}

impl GTrait for Geometries {
    fn is_2d(&self) -> bool {
        match &self.items[0] {
            G::Point(x) => x.is_2d(),
            G::Line(x) => x.is_2d(),
            G::Polygon(x) => x.is_2d(),
            G::Points(x) => x.is_2d(),
            G::Lines(x) => x.is_2d(),
            G::Polygons(x) => x.is_2d(),
            _ => {
                error!("Unexpected geometries item");
                false
            }
        }
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        let items: Vec<String> = self
            .items
            .iter()
            .map(|x| match x {
                G::Point(x) => x.to_wkt_fmt(precision),
                G::Line(x) => x.to_wkt_fmt(precision),
                G::Polygon(x) => x.to_wkt_fmt(precision),
                G::Points(x) => x.to_wkt_fmt(precision),
                G::Lines(x) => x.to_wkt_fmt(precision),
                G::Polygons(x) => x.to_wkt_fmt(precision),
                G::BBox(x) => x.to_wkt_fmt(precision),
                _ => panic!("Unexpected geometries item"),
            })
            .collect();
        if self.is_2d() {
            format!("GEOMETRYCOLLECTION ({})", items.join(", "))
        } else {
            format!("GEOMETRYCOLLECTION Z ({})", items.join(", "))
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        if self.items.iter().all(|x| match x {
            G::Point(x) => x.check_coordinates(crs).is_ok(),
            G::Line(x) => x.check_coordinates(crs).is_ok(),
            G::Polygon(x) => x.check_coordinates(crs).is_ok(),
            G::Points(x) => x.check_coordinates(crs).is_ok(),
            G::Lines(x) => x.check_coordinates(crs).is_ok(),
            G::Polygons(x) => x.check_coordinates(crs).is_ok(),
            _ => {
                error!("Unexpected geometries item");
                false
            }
        }) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "At least one geometry has invalid coordinates".into(),
            ))
        }
    }

    fn type_(&self) -> &str {
        "GeometryCollection"
    }

    fn srid(&self) -> SRID {
        self.srid
    }
}

impl Geometries {
    /// Return the number of items in this.
    pub fn num_geometries(&self) -> usize {
        self.items.len()
    }

    /// Return an iterator over the geometries.
    pub fn geometries(&self) -> Iter<'_, G> {
        self.items.iter()
    }

    pub(crate) fn from_items(items: Vec<G>) -> Self {
        Self::from_items_and_srid(items, SRID::default())
    }

    pub(crate) fn from_items_and_srid(items: Vec<G>, srid: SRID) -> Self {
        Geometries { items, srid }
    }

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        let items: Result<Vec<Geometry>, MyError> = self
            .items
            .iter()
            .map(|x| match x {
                G::Point(x) => x.to_geos(),
                G::Line(x) => x.to_geos(),
                G::Polygon(x) => x.to_geos(),
                G::Points(x) => x.to_geos(),
                G::Lines(x) => x.to_geos(),
                G::Polygons(x) => x.to_geos(),
                _ => panic!("Unexpected geometries item"),
            })
            .collect();
        let mut g = Geometry::create_geometry_collection(items?)?;
        let srs_id = self.srid.as_usize()?;
        g.set_srid(srs_id);

        Ok(g)
    }

    pub(crate) fn from_geos_xy<T: Geom>(gg: T) -> Result<Vec<G>, MyError> {
        let num_geometries = gg.get_num_geometries()?;
        let mut result = Vec::with_capacity(num_geometries);
        for ndx in 0..num_geometries {
            let g = gg.get_geometry_n(ndx)?;
            let item = G::try_from(g)?;
            result.push(item);
        }
        Ok(result)
    }

    pub(crate) fn set_srid_unchecked(&mut self, srid: &SRID) {
        if self.srid != *srid {
            warn!("Replacing current SRID ({}) w/ {srid}", self.srid);
            self.items
                .iter_mut()
                .for_each(|g| g.set_srid_unchecked(srid));
            self.srid = srid.to_owned();
        }
    }
}

impl fmt::Display for Geometries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Geometries (...)")
    }
}

impl TryFrom<Geometry> for Geometries {
    type Error = MyError;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS GeometryCollection. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let items = Geometries::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Geometries::from_items_and_srid(items, srid))
    }
}

impl TryFrom<ConstGeometry<'_>> for Geometries {
    type Error = MyError;

    fn try_from(value: ConstGeometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS GeometryCollection. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let items = Geometries::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Geometries::from_items_and_srid(items, srid))
    }
}
