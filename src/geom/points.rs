// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Collection of point geometries.
//!

use crate::{
    CRS, GTrait, MyError, Point,
    config::config,
    geom::{XY1V, XY2V},
    srid::SRID,
};
use core::fmt;
use geos::{ConstGeometry, Geom, Geometry};
use std::slice::Iter;
use tracing::{error, warn};

/// Collection of point geometries.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Points {
    points: XY2V,
    srid: SRID,
}

impl GTrait for Points {
    fn is_2d(&self) -> bool {
        self.points[0].len() == 2
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        if self.is_2d() {
            format!(
                "MULTIPOINT {}",
                Self::coords_with_dp(&self.points, precision)
            )
        } else {
            format!(
                "MULTIPOINT Z {}",
                Self::coords_with_dp(&self.points, precision)
            )
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        if self.points.iter().all(|p| crs.check_point(p).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "At least one point has invalid coordinates".into(),
            ))
        }
    }

    fn type_(&self) -> &str {
        "MultiPoint"
    }

    fn srid(&self) -> SRID {
        self.srid
    }
}

impl Points {
    /// Return the number of points in this.
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    /// Return an iterator over the points' coordinates.
    pub fn points(&self) -> Iter<'_, XY1V> {
        self.points.iter()
    }

    pub(crate) fn from_xy(points: XY2V) -> Self {
        Self::from_xy_and_srid(points, SRID::default())
    }

    pub(crate) fn from_xy_and_srid(points: XY2V, srid: SRID) -> Self {
        let points = points
            .iter()
            .map(|x| Point::ensure_precision_xy(x))
            .collect();
        Points { points, srid }
    }

    pub(crate) fn coords_as_txt(points: &[XY1V]) -> String {
        Self::coords_with_dp(points, config().default_precision())
    }

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        let mut points: Vec<Geometry> = vec![];
        for p in &self.points {
            let g = Point::to_geos_xy(p, &self.srid)?;
            points.push(g);
        }
        let mut g = Geometry::create_multipoint(points)?;
        let srs_id = self.srid.as_usize()?;
        g.set_srid(srs_id);

        Ok(g)
    }

    pub(crate) fn from_geos_xy<T: Geom>(gg: T) -> Result<XY2V, MyError> {
        let num_points = gg.get_num_geometries()?;
        let mut result = Vec::with_capacity(num_points);
        for ndx in 0..num_points {
            let z_point = gg.get_geometry_n(ndx)?;
            let xy = if z_point.has_z()? {
                vec![gg.get_x()?, gg.get_y()?, gg.get_z()?]
            } else {
                vec![gg.get_x()?, gg.get_y()?]
            };
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

    fn coords_with_dp(points: &[XY1V], precision: usize) -> String {
        let points: Vec<String> = points
            .iter()
            .map(|x| Point::coords_with_dp(x, precision))
            .collect();
        format!("({})", points.join(", "))
    }
}

impl fmt::Display for Points {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Points (...)")
    }
}

impl TryFrom<Geometry> for Points {
    type Error = MyError;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS MultiPoint. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let points = Points::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Points { points, srid })
    }
}

impl TryFrom<ConstGeometry<'_>> for Points {
    type Error = MyError;

    fn try_from(value: ConstGeometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS MultiPoint. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let points = Points::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Points { points, srid })
    }
}
