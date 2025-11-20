// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Line geometry.
//!

use crate::{
    CRS, GTrait, MyError, Point,
    config::config,
    geom::{XY1V, XY2V},
    srid::SRID,
};
use core::fmt;
use geos::{ConstGeometry, CoordDimensions, CoordSeq, Geom, Geometry};
use std::slice::Iter;
use tracing::{error, warn};

/// 2D or 3D line-string geometry.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Line {
    coord: XY2V,
    srid: SRID,
}

impl GTrait for Line {
    fn is_2d(&self) -> bool {
        self.coord[0].len() == 2
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        if self.is_2d() {
            format!(
                "LINESTRING {}",
                Self::coords_with_dp(&self.coord, precision)
            )
        } else {
            format!(
                "LINESTRING Z {}",
                Self::coords_with_dp(&self.coord, precision)
            )
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        crs.check_line(&self.coord)
    }

    fn type_(&self) -> &str {
        "LineString"
    }

    fn srid(&self) -> SRID {
        self.srid
    }
}

impl Line {
    /// Return the number of vertices/points in this.
    pub fn num_points(&self) -> usize {
        self.coord.len()
    }

    /// Return an iterator over the points' coordinates.
    pub fn points(&self) -> Iter<'_, XY1V> {
        self.coord.iter()
    }

    pub(crate) fn from_xy(coord: XY2V) -> Self {
        Self::from_xy_and_srid(coord, SRID::default())
    }

    pub(crate) fn from_xy_and_srid(coord: XY2V, srid: SRID) -> Self {
        let coord = Self::ensure_precision_xy(&coord);
        Line { coord, srid }
    }

    pub(crate) fn coords_as_txt(coord: &[XY1V]) -> String {
        Self::coords_with_dp(coord, config().default_precision())
    }

    pub(crate) fn ensure_precision_xy(coord: &[XY1V]) -> XY2V {
        coord
            .iter()
            .map(|x| Point::ensure_precision_xy(x))
            .collect()
    }

    pub(crate) fn coords_with_dp(coord: &[XY1V], precision: usize) -> String {
        let points: Vec<String> = coord
            .iter()
            .map(|x| Point::coords_with_dp(x, precision))
            .collect();
        format!("({})", points.join(", "))
    }

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        Self::to_geos_xy(&self.coord, &self.srid)
    }

    pub(crate) fn to_geos_xy(xy: &[XY1V], srid: &SRID) -> Result<Geometry, MyError> {
        let vertices: Vec<&[f64]> = xy.iter().map(|x| x.as_slice()).collect();
        let xy = CoordSeq::new_from_vec(&vertices)?;
        let mut g = Geometry::create_line_string(xy)?;
        let srs_id = srid.as_usize()?;
        g.set_srid(srs_id);

        Ok(g)
    }

    // Return TRUE if the first and last vertices coincide. FALSE otherwise.
    pub(crate) fn is_closed(&self) -> bool {
        self.coord.first() == self.coord.last()
    }

    pub(crate) fn from_geos_xy<T: Geom>(gg: T) -> Result<XY2V, MyError> {
        let cs = gg.get_coord_seq()?;
        let is_3d = match cs.dimensions()? {
            CoordDimensions::OneD => {
                let msg = "Don't know how to handle 1D points";
                error!("Failed: {msg}");
                return Err(MyError::Runtime(msg.into()));
            }
            CoordDimensions::TwoD => false,
            CoordDimensions::ThreeD => true,
        };
        let num_vertices = cs.size()?;
        let mut xy = Vec::with_capacity(num_vertices);
        for ndx in 0..num_vertices {
            let vertex = if is_3d {
                vec![cs.get_x(ndx)?, cs.get_y(ndx)?, cs.get_z(ndx)?]
            } else {
                vec![cs.get_x(ndx)?, cs.get_y(ndx)?]
            };
            xy.push(vertex);
        }
        Ok(xy)
    }

    pub(crate) fn set_srid_unchecked(&mut self, srid: &SRID) {
        if self.srid != *srid {
            warn!("Replacing current SRID ({}) w/ {srid}", self.srid);
            self.srid = srid.to_owned();
        }
    }

    // Return TRUE if this cnsists of at least 4 points w/ the first and last
    // ones coinciding. Return FALSE otherwise.
    #[cfg(test)]
    pub(crate) fn is_ring(&self) -> bool {
        self.num_points() > 3 && self.is_closed()
    }

    #[cfg(test)]
    pub(crate) fn first(&self) -> Option<&XY1V> {
        self.coord.first()
    }

    #[cfg(test)]
    pub(crate) fn last(&self) -> Option<&XY1V> {
        self.coord.last()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        if self.is_closed() {
            write!(f, "Ring (...)")
        } else {
            write!(f, "Line (...)")
        }
    }
}

impl TryFrom<Geometry> for Line {
    type Error = MyError;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!("Failed get_srid for GEOS Line. Will use Undefined: {}", x);
            Default::default()
        });
        let xy = Line::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Line::from_xy_and_srid(xy, srid))
    }
}

impl TryFrom<ConstGeometry<'_>> for Line {
    type Error = MyError;

    fn try_from(value: ConstGeometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!("Failed get_srid for GEOS Line. Will use Undefined: {}", x);
            Default::default()
        });
        let xy = Line::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Line::from_xy_and_srid(xy, srid))
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
    #[tracing_test::traced_test]
    fn test() {
        const G: &str = r#"LineString(43.72992 -79.2998, 43.73005 -79.2991, 43.73006 -79.2984,
                   43.73140 -79.2956, 43.73259 -79.2950, 43.73266 -79.2945,
                   43.73320 -79.2936, 43.73378 -79.2936, 43.73486 -79.2917)"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        let spa = exp.unwrap();
        let g = match spa {
            E::Spatial(G::Line(x)) => x,
            _ => panic!("Not a Line..."),
        };
        assert_eq!(g.is_2d(), true);
        assert_eq!(g.num_points(), 9);

        assert!(g.first().is_some());
        let first = g.first().unwrap();
        assert_relative_eq!(first[0], 43.729, epsilon = TOLERANCE);
        assert_relative_eq!(first[1], -79.299, epsilon = TOLERANCE);
        assert!(g.last().is_some());
        let last = g.last().unwrap();
        assert_relative_eq!(last[0], 43.734, epsilon = TOLERANCE);
        assert_relative_eq!(last[1], -79.291, epsilon = TOLERANCE);
    }

    #[test]
    #[should_panic]
    fn test_invalid() {
        let line = Line::from_xy(vec![vec![0.0, 45.0], vec![90.0, 180.0], vec![45.0, 45.0]]);
        let crs = CRS::default();
        line.check_coordinates(&crs).unwrap();
    }

    #[test]
    fn test_precision() -> Result<(), Box<dyn Error>> {
        const WKT: &str = "LINESTRING (82.400480 30.411477, 82.722734 30.365046)";

        let line_xy = vec![
            vec![82.400479770847, 30.4114773625851],
            vec![82.7227340026191, 30.3650460881709],
        ];

        let line = Line::from_xy(line_xy);
        let wkt = line.to_wkt_fmt(6);
        assert_eq!(wkt, WKT);

        Ok(())
    }
}
