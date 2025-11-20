// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Collection of polygon geometries.
//!

use crate::{
    CRS, GTrait, MyError, Polygon,
    config::config,
    geom::{XY3V, XY4V},
    srid::SRID,
};
use core::fmt;
use geos::{ConstGeometry, Geom, Geometry};
use std::slice::Iter;
use tracing::{error, warn};

/// Collection of polygon geometries.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Polygons {
    polygons: XY4V,
    srid: SRID,
}

impl GTrait for Polygons {
    fn is_2d(&self) -> bool {
        self.polygons[0][0][0].len() == 2
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        if self.is_2d() {
            format!(
                "MULTIPOLYGON {}",
                Self::coords_with_dp(self.polygons.as_slice(), precision)
            )
        } else {
            format!(
                "MULTIPOLYGON Z {}",
                Self::coords_with_dp(self.polygons.as_slice(), precision)
            )
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        if self
            .polygons
            .iter()
            .all(|poly| crs.check_polygon(poly).is_ok())
        {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "At least one polygon has invalid coordinates".into(),
            ))
        }
    }

    fn type_(&self) -> &str {
        "MultiPolygon"
    }

    fn srid(&self) -> SRID {
        self.srid
    }
}

impl Polygons {
    /// Return the number of polygons in this.
    pub fn num_polygons(&self) -> usize {
        self.polygons.len()
    }

    /// Return an iterator over this polygons' coordinates.
    pub fn polygons(&self) -> Iter<'_, XY3V> {
        self.polygons.iter()
    }

    pub(crate) fn from_xy(polygons: XY4V) -> Self {
        Self::from_xy_and_srid(polygons, SRID::default())
    }

    pub(crate) fn from_xy_and_srid(polygons: XY4V, srid: SRID) -> Self {
        let polygons = polygons
            .iter()
            .map(|x| Polygon::ensure_precision_xy(x))
            .collect();
        Polygons { polygons, srid }
    }

    pub(crate) fn coords_as_txt(polygons: &[XY3V]) -> String {
        Self::coords_with_dp(polygons, config().default_precision())
    }

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        let mut polygons: Vec<Geometry> = vec![];
        for p in &self.polygons {
            let g = Polygon::to_geos_xy(p, &self.srid)?;
            polygons.push(g);
        }
        let mut g = Geometry::create_multipolygon(polygons)?;
        let srs_id = self.srid.as_usize()?;
        g.set_srid(srs_id);

        Ok(g)
    }

    pub(crate) fn from_geos_xy<T: Geom>(gg: T) -> Result<XY4V, MyError> {
        let num_polygons = gg.get_num_geometries()?;
        let mut result = Vec::with_capacity(num_polygons);
        for ndx in 0..num_polygons {
            let polygon = gg.get_geometry_n(ndx)?;
            let xy = Polygon::from_geos_xy(polygon)?;
            result.push(xy);
        }
        Ok(result)
    }

    pub(crate) fn set_srid_unchecked(&mut self, srid: &SRID) {
        if self.srid != *srid {
            warn!("Replacing current SRID ({}) w/ {srid}", self.srid);
            self.srid = srid.to_owned();
        }
    }

    fn coords_with_dp(polygons: &[XY3V], precision: usize) -> String {
        let polygons: Vec<String> = polygons
            .iter()
            .map(|x| Polygon::coords_with_dp(x.as_slice(), precision))
            .collect();
        format!("({})", polygons.join(", "))
    }
}

impl fmt::Display for Polygons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Polygons (...)")
    }
}

impl TryFrom<Geometry> for Polygons {
    type Error = MyError;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS MultiPolygon. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let polygons = Polygons::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Polygons::from_xy_and_srid(polygons, srid))
    }
}

impl TryFrom<ConstGeometry<'_>> for Polygons {
    type Error = MyError;

    fn try_from(value: ConstGeometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS MultiPolygon. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let polygons = Polygons::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Polygons::from_xy_and_srid(polygons, srid))
    }
}
