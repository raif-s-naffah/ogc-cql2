// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Collection of line-string geometries.
//!

use crate::{
    CRS, GTrait, Line, MyError,
    config::config,
    geom::{XY2V, XY3V},
    srid::SRID,
};
use core::fmt;
use geos::{ConstGeometry, Geom, Geometry};
use std::slice::Iter;
use tracing::{error, warn};

/// Collection of line-string geometries.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Lines {
    lines: XY3V,
    srid: SRID,
}

impl GTrait for Lines {
    fn is_2d(&self) -> bool {
        self.lines[0][0].len() == 2
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        if self.is_2d() {
            format!(
                "MULTILINESTRING {}",
                Self::coords_with_dp(self.lines.as_slice(), precision)
            )
        } else {
            format!(
                "MULTILINESTRING Z {}",
                Self::coords_with_dp(self.lines.as_slice(), precision)
            )
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        if self.lines.iter().all(|l| crs.check_line(l).is_ok()) {
            Ok(())
        } else {
            Err(MyError::Runtime(
                "At least one line has invalid coordinates".into(),
            ))
        }
    }

    fn type_(&self) -> &str {
        "MultiLineString"
    }

    fn srid(&self) -> SRID {
        self.srid
    }
}

impl Lines {
    /// Return the number of lines in this.
    pub fn num_lines(&self) -> usize {
        self.lines.len()
    }

    /// Return an iterator over the lines' coordinates.
    pub fn lines(&self) -> Iter<'_, XY2V> {
        self.lines.iter()
    }

    pub(crate) fn from_xy(lines: XY3V) -> Self {
        Self::from_xy_and_srid(lines, SRID::default())
    }

    pub(crate) fn from_xy_and_srid(lines: XY3V, srid: SRID) -> Self {
        let lines = lines.iter().map(|x| Line::ensure_precision_xy(x)).collect();
        Lines { lines, srid }
    }

    pub(crate) fn coords_as_txt(lines: &[XY2V]) -> String {
        Self::coords_with_dp(lines, config().default_precision())
    }

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        let mut lines: Vec<Geometry> = vec![];
        for l in &self.lines {
            let g = Line::to_geos_xy(l, &self.srid)?;
            lines.push(g);
        }
        let mut g = Geometry::create_multiline_string(lines)?;
        let srs_id = self.srid.as_usize()?;
        g.set_srid(srs_id);

        Ok(g)
    }

    pub(crate) fn from_geos_xy<T: Geom>(gg: T) -> Result<XY3V, MyError> {
        let num_lines = gg.get_num_geometries()?;
        let mut result = Vec::with_capacity(num_lines);
        for ndx in 0..num_lines {
            let line = gg.get_geometry_n(ndx)?;
            let xy = Line::from_geos_xy(line)?;
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

    fn coords_with_dp(lines: &[XY2V], precision: usize) -> String {
        let lines: Vec<String> = lines
            .iter()
            .map(|x| Line::coords_with_dp(x.as_slice(), precision))
            .collect();
        format!("({})", lines.join(", "))
    }
}

impl fmt::Display for Lines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lines (...)")
    }
}

impl TryFrom<Geometry> for Lines {
    type Error = MyError;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS MultiLine. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let lines = Lines::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Lines::from_xy_and_srid(lines, srid))
    }
}

impl TryFrom<ConstGeometry<'_>> for Lines {
    type Error = MyError;

    fn try_from(value: ConstGeometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS MultiLine. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let lines = Lines::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Lines::from_xy_and_srid(lines, srid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{G, expr::E, text::cql2};

    #[test]
    #[tracing_test::traced_test]
    fn test() {
        const G: &str = r#"MultiLineString(
        ( -1.9 -0.99999, 75.292574 1.5,     -0.5 -4.016458, -31.708594 -74.743801, 179.0 -90.0 ),
        (-1.9 -1.1,      1.5      8.547371))"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        let spa = exp.unwrap();
        let g = match spa {
            E::Spatial(G::Lines(x)) => x,
            _ => panic!("Not a Polygon..."),
        };
        assert_eq!(g.is_2d(), true);
        let lines = g.lines();
        assert_eq!(lines.len(), 2);
        // or...
        assert_eq!(lines.len(), g.num_lines());
    }
}
