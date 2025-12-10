// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Point geometry.
//!

use crate::{
    CRS, GTrait, MyError,
    config::config,
    geom::{XY1V, ensure_precision},
    srid::SRID,
};
use core::fmt;
use geos::{ConstGeometry, CoordSeq, Geom, Geometry};
use tracing::{error, warn};

/// 2D or 3D point geometry.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Point {
    coord: XY1V,
    srid: SRID,
}

impl GTrait for Point {
    fn is_2d(&self) -> bool {
        self.coord.len() == 2
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        if self.is_2d() {
            format!("POINT ({})", Self::coords_with_dp(self.as_2d(), precision))
        } else {
            format!(
                "POINT Z ({})",
                Self::coords_with_dp(self.as_3d(), precision)
            )
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        crs.check_point(&self.coord)
    }

    fn type_(&self) -> &str {
        "Point"
    }

    fn srid(&self) -> SRID {
        self.srid
    }
}

impl Point {
    /// Return a reference to the point's coordinates.
    pub fn xy(&self) -> &Vec<f64> {
        &self.coord
    }

    pub(crate) fn from_xy(coord: XY1V) -> Self {
        Self::from_xy_and_srid(coord, *config().default_srid())
    }

    pub(crate) fn from_xy_and_srid(coord: XY1V, srid: SRID) -> Self {
        // shape the input coordinates to a fixed precision; i.e. a fixed
        // number of decimals so `geos` can reliably assert equality of
        // coordinate values.
        let coord = Self::ensure_precision_xy(&coord);
        Point { coord, srid }
    }

    // Output given coordinates sequentially seperated by a space.
    pub(crate) fn coords_as_txt(coord: &[f64]) -> String {
        Self::coords_with_dp(coord, config().default_precision())
    }

    pub(crate) fn ensure_precision_xy(coord: &[f64]) -> XY1V {
        coord.iter().map(ensure_precision).collect()
    }

    pub(crate) fn coords_with_dp(coord: &[f64], precision: usize) -> String {
        if coord.len() == 2 {
            format!("{:.2$} {:.2$}", coord[0], coord[1], precision)
        } else {
            format!(
                "{:.3$} {:.3$} {:.3$}",
                coord[0], coord[1], coord[2], precision
            )
        }
    }

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        Self::to_geos_xy(&self.coord, &self.srid)
    }

    pub(crate) fn to_geos_xy(xy: &[f64], srid: &SRID) -> Result<Geometry, MyError> {
        let xy = CoordSeq::new_from_vec(&[xy])?;
        let mut g = Geometry::create_point(xy)?;
        let srs_id = srid.as_usize()?;
        g.set_srid(srs_id);

        Ok(g)
    }

    // Return the 1st coordinate of this.
    pub(crate) fn x(&self) -> f64 {
        self.coord[0]
    }

    // Return the 2nd coordinate of this.
    pub(crate) fn y(&self) -> f64 {
        self.coord[1]
    }

    // Return the 3rd coordinate of this if it's a 3D one. Return `None` otherwise.
    pub(crate) fn z(&self) -> Option<f64> {
        if self.coord.len() == 2 {
            None
        } else {
            Some(self.coord[2])
        }
    }

    pub(crate) fn from_geos_xy<T: Geom>(gg: T) -> Result<XY1V, MyError> {
        let result = if gg.has_z()? {
            vec![gg.get_x()?, gg.get_y()?, gg.get_z()?]
        } else {
            vec![gg.get_x()?, gg.get_y()?]
        };
        Ok(result)
    }

    pub(crate) fn set_srid_unchecked(&mut self, srid: &SRID) {
        if self.srid != *srid {
            warn!("Replacing current SRID ({}) w/ {srid}", self.srid);
            self.srid = srid.to_owned();
        }
    }

    // Return the 2D coordinates of this point.
    fn as_2d(&self) -> &[f64; 2] {
        self.coord
            .as_slice()
            .try_into()
            .expect("Failed coercing Point to 2D")
    }

    // Return the 3D coordinates of this point.
    fn as_3d(&self) -> &[f64; 3] {
        self.coord
            .as_slice()
            .try_into()
            .expect("Failed coercing Point to 3D")
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point (...)")
    }
}

impl TryFrom<Geometry> for Point {
    type Error = MyError;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!("Failed get_srid for GEOS Point. Will use Undefined: {}", x);
            Default::default()
        });
        let xy = Point::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Point::from_xy_and_srid(xy, srid))
    }
}

impl TryFrom<ConstGeometry<'_>> for Point {
    type Error = MyError;

    fn try_from(value: ConstGeometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!("Failed get_srid for GEOS Point. Will use Undefined: {}", x);
            Default::default()
        });
        let xy = Point::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Point::from_xy_and_srid(xy, srid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{G, expr::E, text::cql2};
    use approx::assert_relative_eq;
    use std::error::Error;

    const TOLERANCE: f64 = 1.0E-3;

    #[test]
    fn test_equality() {
        let p1 = Point {
            coord: vec![1., 1.],
            srid: SRID::default(),
        };
        let p2 = Point {
            coord: vec![1.0, 1.0],
            srid: SRID::default(),
        };
        let p3 = Point {
            coord: vec![1.0, 1.1],
            srid: SRID::default(),
        };
        let p4 = Point {
            coord: vec![1.1, 1.0],
            srid: SRID::default(),
        };

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
        assert_ne!(p1, p4);
        assert!(p1 == p2);
        assert!(p1 != p3);
        assert!(p2 != p4);
        assert!(p3 != p4);
    }

    #[test]
    fn test_comparison() {
        let p1 = Point {
            coord: vec![1.0, 1.0],
            srid: SRID::default(),
        };
        let p2 = Point {
            coord: vec![1.0, 1.1],
            srid: SRID::default(),
        };
        let p3 = Point {
            coord: vec![1.1, 1.0],
            srid: SRID::default(),
        };

        assert!(p1 < p2);
        assert!(p1 < p3);
        assert!(p2 < p3);
    }

    #[test]
    #[tracing_test::traced_test]
    fn test() {
        const G: &str = r#"point (-3.508362 -1.754181)"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        let spa = exp.unwrap();
        let g = match spa {
            E::Spatial(G::Point(x)) => x,
            _ => panic!("Not a Point..."),
        };
        assert_eq!(g.is_2d(), true);
        // or...
        assert!(g.z().is_none());
        assert_relative_eq!(g.x(), -3.508, epsilon = TOLERANCE);
        assert_relative_eq!(g.y(), -1.754, epsilon = TOLERANCE);
    }

    #[test]
    #[should_panic]
    fn test_invalid() {
        let pt = Point::from_xy(vec![90.0, 180.0]);
        let crs = CRS::default();
        pt.check_coordinates(&crs).unwrap();
    }

    #[test]
    fn test_precision() -> Result<(), Box<dyn Error>> {
        const XYZ: [f64; 3] = [-16.0671326636424, -17.012041674368, 179.096609362997];
        const WKT: &str = "POINT Z (-16.067133 -17.012042 179.096609)";

        let pt = Point::from_xy(XYZ.to_vec());
        let wkt = pt.to_wkt_fmt(6);
        assert_eq!(wkt, WKT);

        Ok(())
    }
}
