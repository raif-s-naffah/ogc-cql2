// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Polygon geometry.
//!

use crate::{
    CRS, GTrait, Line, MyError,
    config::config,
    geom::{XY2V, XY3V},
    srid::SRID,
};
use core::fmt;
use geos::{ConstGeometry, CoordSeq, Geom, Geometry};
use std::slice::Iter;
use tracing::{error, warn};

/// 2D or 3D polygon geometry.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Polygon {
    pub(crate) rings: XY3V,
    srid: SRID,
}

impl GTrait for Polygon {
    fn is_2d(&self) -> bool {
        self.rings[0][0].len() == 2
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        if self.is_2d() {
            format!("POLYGON {}", Self::coords_with_dp(&self.rings, precision))
        } else {
            format!("POLYGON Z {}", Self::coords_with_dp(&self.rings, precision))
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        crs.check_polygon(&self.rings)
    }

    fn type_(&self) -> &str {
        "Polygon"
    }

    fn srid(&self) -> SRID {
        self.srid
    }
}

impl Polygon {
    /// Return the number of rings in this.
    pub fn num_rings(&self) -> usize {
        self.rings.len()
    }

    /// Return an iterator over the rings' coordinates.
    pub fn rings(&self) -> Iter<'_, XY2V> {
        self.rings.iter()
    }

    pub(crate) fn from_xy(rings: XY3V) -> Self {
        Self::from_xy_and_srid(rings, *config().default_srid())
    }

    pub(crate) fn from_xy_and_srid(rings: XY3V, srid: SRID) -> Self {
        let rings = Self::ensure_precision_xy(&rings);
        Self::from_xy_and_srid_unchecked(rings, srid)
    }

    pub(crate) fn from_xy_and_srid_unchecked(rings: XY3V, srid: SRID) -> Self {
        Polygon { rings, srid }
    }

    pub(crate) fn coords_as_txt(rings: &[XY2V]) -> String {
        Self::coords_with_dp(rings, config().default_precision())
    }

    pub(crate) fn ensure_precision_xy(rings: &[XY2V]) -> XY3V {
        rings.iter().map(|r| Line::ensure_precision_xy(r)).collect()
    }

    pub(crate) fn coords_with_dp(rings: &[XY2V], precision: usize) -> String {
        let rings: Vec<String> = rings
            .iter()
            .map(|x| Line::coords_with_dp(x, precision))
            .collect();
        format!("({})", rings.join(", "))
    }

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        Self::to_geos_xy(&self.rings, &self.srid)
    }

    pub(crate) fn to_geos_xy(rings: &[XY2V], srid: &SRID) -> Result<Geometry, MyError> {
        let vertices: Vec<&[f64]> = rings[0].iter().map(|x| x.as_slice()).collect();
        let xy = CoordSeq::new_from_vec(&vertices)?;
        let mut exterior = Geometry::create_linear_ring(xy)?;
        let srs_id = srid.as_usize()?;
        exterior.set_srid(srs_id);

        let mut interiors = vec![];
        for hole in &rings[1..] {
            let vertices: Vec<&[f64]> = hole.iter().map(|x| x.as_slice()).collect();
            let xy = CoordSeq::new_from_vec(&vertices)?;
            let mut hole = Geometry::create_linear_ring(xy)?;
            hole.set_srid(srs_id);
            interiors.push(hole);
        }

        let mut g = Geometry::create_polygon(exterior, interiors)?;
        g.set_srid(srs_id);

        Ok(g)
    }

    pub(crate) fn from_geos_xy<T: Geom>(gg: T) -> Result<XY3V, MyError> {
        let num_inners = gg.get_num_interior_rings()?;
        let mut result = Vec::with_capacity(num_inners + 1);

        let outer = gg.get_exterior_ring()?;
        let xy = Line::from_geos_xy(outer)?;
        result.push(xy);

        let n = u32::try_from(num_inners)?;
        for ndx in 0..n {
            let inner = gg.get_interior_ring_n(ndx)?;
            let xy = Line::from_geos_xy(inner)?;
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

    #[cfg(test)]
    fn outer_as_ring(&self) -> Line {
        Line::from_xy(self.rings[0].to_vec())
    }

    // Return TRUE if this has holes; i.e. more than 1 linear ring. Return
    // FALSE otherwise.
    #[cfg(test)]
    fn has_holes(&self) -> bool {
        self.rings.len() > 1
    }

    // Return the array of inner (holes) linear rings of this.
    #[cfg(test)]
    fn inners(&self) -> &[Vec<Vec<f64>>] {
        &self.rings.as_slice()[1..]
    }
}

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Polygon (...)")
    }
}

impl TryFrom<Geometry> for Polygon {
    type Error = MyError;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS Polygon. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let rings = Self::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Polygon::from_xy_and_srid(rings, srid))
    }
}

impl TryFrom<ConstGeometry<'_>> for Polygon {
    type Error = MyError;

    fn try_from(value: ConstGeometry) -> Result<Self, Self::Error> {
        let srs_id = value.get_srid().unwrap_or_else(|x| {
            error!(
                "Failed get_srid for GEOS Polygon. Will use Undefined: {}",
                x
            );
            Default::default()
        });
        let rings = Self::from_geos_xy(value)?;
        let srid = SRID::try_from(srs_id)?;
        Ok(Polygon::from_xy_and_srid(rings, srid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{G, expr::E, text::cql2};
    use geos::Geom;
    use std::error::Error;

    #[test]
    #[tracing_test::traced_test]
    fn test_2d() {
        const G: &str = r#"PolyGon ((-0.333333 89.0, -102.723546 -0.5, -179.0 -89.0, -1.9 89.0, -0.0 89.0, 2.00001 -1.9, -0.333333 89.0))"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        let spa = exp.unwrap();
        let g = match spa {
            E::Spatial(G::Polygon(x)) => x,
            _ => panic!("Not a Polygon..."),
        };
        assert_eq!(g.is_2d(), true);

        let outer_ring = g.outer_as_ring();
        assert!(outer_ring.is_ring());
        assert!(outer_ring.is_closed());
        assert_eq!(outer_ring.num_points(), 7);

        // has no holes...
        assert!(!g.has_holes());
        assert!(g.inners().is_empty());
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_3d() {
        const G: &str = r#"POLYGON Z ((-49.88024 0.5 -75993.341684, -1.5 -0.99999 -100000.0, 0.0 0.5 -0.333333, -49.88024 0.5 -75993.341684), (-65.887123 2.00001 -100000.0, 0.333333 -53.017711 -79471.332949, 180.0 0.0 1852.616704, -65.887123 2.00001 -100000.0))"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        let spa = exp.unwrap();
        let g = match spa {
            E::Spatial(G::Polygon(x)) => x,
            _ => panic!("Not a Polygon..."),
        };
        assert_eq!(g.is_2d(), false);

        let outer_ring = g.outer_as_ring();
        assert!(outer_ring.is_ring());
        assert!(outer_ring.is_closed());
        assert_eq!(outer_ring.num_points(), 4);

        // has 1 hole...
        assert!(g.has_holes());
        assert_eq!(g.inners().len(), 1);
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_touches() -> Result<(), Box<dyn Error>> {
        const WKT1: &str = "POLYGON ((0 -90, 0 0, 180 0, 180 -90, 0 -90))";
        const WKT2: &str = "POLYGON ((-180 -90, -180 90, 180 90, 180 -90, -180 -90))";

        let p1 = Geometry::new_from_wkt(WKT1)?;
        let p2 = Geometry::new_from_wkt(WKT2)?;

        // although p1 and p2 share a segment of their bottom side, their
        // interiors are NOT disjoint and as such they are considered to
        // not "touch" each other.
        assert!(!p1.touches(&p2)?);

        Ok(())
    }
}
