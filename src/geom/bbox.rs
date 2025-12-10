// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Bounding box geometry.
//!

use crate::{
    CRS, EPSG_4326, GTrait, MyError, Polygon, Polygons, geom::ensure_precision, srid::SRID,
};
use core::fmt;
use geos::{CoordSeq, Geometry};
use tracing::{error, warn};

/// 2D or 3D bounding box.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BBox {
    w: f64,             // west bound longitude
    s: f64,             // south bound latitude
    z_min: Option<f64>, // minimum elevation
    e: f64,             // east bound longitude
    n: f64,             // north bound latitude
    z_max: Option<f64>, // maximum elevation

    srid: SRID,
}

impl GTrait for BBox {
    fn is_2d(&self) -> bool {
        self.z_min.is_none()
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        if self.z_min.is_none() {
            format!(
                "BBOX ({:.4$}, {:.4$}, {:.4$}, {:.4$})",
                self.w, self.s, self.e, self.n, precision
            )
        } else {
            format!(
                "BBOX ({:.6$}, {:.6$}, {:.6$}, {:.6$}, {:.6$}, {:.6$})",
                self.w,
                self.s,
                self.z_min.unwrap(),
                self.e,
                self.n,
                self.z_max.unwrap(),
                precision
            )
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        crs.check_point([self.w, self.s].as_ref())?;
        crs.check_point([self.e, self.n].as_ref())?;
        Ok(())
    }

    fn type_(&self) -> &str {
        "BBox"
    }

    fn srid(&self) -> SRID {
        self.srid
    }
}

impl BBox {
    /// (from [1]) If the vertical axis is included, the third and the sixth
    /// number are the bottom and the top of the 3-dimensional bounding box.
    ///
    /// [1]: https://docs.ogc.org/is/21-065r2/21-065r2.html#basic-spatial-data-types
    pub(crate) fn from(xy: Vec<f64>) -> Self {
        // bounding boxes SRID is always EPSG:4326...
        let srid = EPSG_4326;
        if xy.len() == 4 {
            BBox {
                w: ensure_precision(&xy[0]),
                s: ensure_precision(&xy[1]),
                z_min: None,
                e: ensure_precision(&xy[2]),
                n: ensure_precision(&xy[3]),
                z_max: None,
                srid,
            }
        } else {
            // panics if not 6-element long...
            BBox {
                w: ensure_precision(&xy[0]),
                s: ensure_precision(&xy[1]),
                z_min: Some(ensure_precision(&xy[2])),
                e: ensure_precision(&xy[3]),
                n: ensure_precision(&xy[4]),
                z_max: Some(ensure_precision(&xy[5])),
                srid,
            }
        }
    }

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        // convert this to one 2D polygon, or in the case of a box that spans the
        // antimeridian, a 2D multi-polygon.
        let x1 = self.w;
        let y1 = self.s;
        let x2 = self.e;
        let y2 = self.n;

        // if x_min is larger than x_max, then the box spans the antimeridian...
        if x1 < x2 {
            let cs =
                CoordSeq::new_from_vec(&[&[x1, y1], &[x2, y1], &[x2, y2], &[x1, y2], &[x1, y1]])
                    .map_err(|x| {
                        error!("Failed creating BBOX outer ring coordinates: {x}");
                        MyError::Geos(x)
                    })?;

            let outer = Geometry::create_linear_ring(cs).map_err(|x| {
                error!("Failed creating BBOX outer ring: {x}");
                MyError::Geos(x)
            })?;

            Geometry::create_polygon(outer, vec![]).map_err(|x| {
                error!("Failed creating BBOX polygon: {x}");
                MyError::Geos(x)
            })
        } else {
            let cs1 = CoordSeq::new_from_vec(&[
                &[x1, y1],
                &[180.0, y1],
                &[180.0, y2],
                &[x1, y2],
                &[x1, y1],
            ])
            .map_err(|x| {
                error!("Failed creating BBOX 1st outer ring coordinates: {x}");
                MyError::Geos(x)
            })?;

            let cs2 = CoordSeq::new_from_vec(&[
                &[x2, y1],
                &[x2, y2],
                &[-180.0, y2],
                &[-180.0, y1],
                &[x2, y1],
            ])
            .map_err(|x| {
                error!("Failed creating BBOX 2nd outer ring coordinates: {x}");
                MyError::Geos(x)
            })?;

            let outer1 = Geometry::create_linear_ring(cs1).map_err(|x| {
                error!("Failed creating BBOX 1st outer ring: {x}");
                MyError::Geos(x)
            })?;

            let outer2 = Geometry::create_linear_ring(cs2).map_err(|x| {
                error!("Failed creating BBOX 2nd outer ring: {x}");
                MyError::Geos(x)
            })?;

            let p1 = Geometry::create_polygon(outer1, vec![]).map_err(|x| {
                error!("Failed creating BBOX 1st polygon: {x}");
                MyError::Geos(x)
            })?;
            let p2 = Geometry::create_polygon(outer2, vec![]).map_err(|x| {
                error!("Failed creating BBOX 1st polygon: {x}");
                MyError::Geos(x)
            })?;

            Geometry::create_multipolygon(vec![p1, p2]).map_err(|x| {
                error!("Failed creating BBOX multi-polygon: {x}");
                MyError::Geos(x)
            })
        }
    }

    pub(crate) fn set_srid_unchecked(&mut self, srid: &SRID) {
        if self.srid != *srid {
            warn!("Replacing current SRID ({}) w/ {srid}", self.srid);
            self.srid = srid.to_owned();
        }
    }

    // some services do not handle this type of geometry but can deal w/ it if
    // it's portrayed as a [Multi]Polygon instead.
    pub(crate) fn to_sql(&self) -> Result<String, MyError> {
        // similar to `to_geos` we need to detect when crossing dateline...
        let x1 = self.w;
        let y1 = self.s;
        let x2 = self.e;
        let y2 = self.n;

        // if x_min is larger than x_max, then the box spans the antimeridian...
        let wkt = if x1 < x2 {
            let p = Polygon::from_xy_and_srid_unchecked(
                vec![vec![
                    vec![x1, y1],
                    vec![x2, y1],
                    vec![x2, y2],
                    vec![x1, y2],
                    vec![x1, y1],
                ]],
                self.srid,
            );
            p.to_wkt()
        } else {
            let pp = Polygons::from_xy_and_srid(
                vec![
                    vec![vec![
                        vec![x1, y1],
                        vec![180.0, y1],
                        vec![180.0, y2],
                        vec![x1, y2],
                        vec![x1, y1],
                    ]],
                    vec![vec![
                        vec![x2, y1],
                        vec![x2, y2],
                        vec![-180.0, y2],
                        vec![-180.0, y1],
                        vec![x2, y1],
                    ]],
                ],
                self.srid,
            );
            pp.to_wkt()
        };

        let srid = self.srid().as_usize()?;
        Ok(format!("ST_GeomFromText('{wkt}', {srid})"))
    }

    // Return the west bound longitude coordinate of this.
    #[cfg(test)]
    fn west(&self) -> f64 {
        self.w
    }

    // Return the east bound longitude coordinate of this.
    #[cfg(test)]
    fn east(&self) -> f64 {
        self.e
    }

    // Return the south bound latitude coordinate of this.
    #[cfg(test)]
    fn south(&self) -> f64 {
        self.s
    }

    // Return the north bound latitude coordinate of this.
    #[cfg(test)]
    fn north(&self) -> f64 {
        self.n
    }

    // Return the lowest (minimum) elevation of this.
    #[cfg(test)]
    fn z_min(&self) -> Option<f64> {
        self.z_min
    }

    // Return the highest (maximum) elevation of this.
    #[cfg(test)]
    fn z_max(&self) -> Option<f64> {
        self.z_max
    }
}

impl fmt::Display for BBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BBox (...)")
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
    fn test() {
        const G1: &str = "bbox(-128.098193, -1.1, -99999.0, 180.0, 90.0, 100000.0)";
        const G2: &str = "bbox(-128.098193,-1.1, -99999.0,180.0 , \t90.0, \n 100000.0)";

        let x = cql2::geom_expression(G1);
        assert!(x.is_ok());
        let g = x.unwrap();
        assert!(matches!(g, E::Spatial(G::BBox(_))));
        let bbox1 = match g {
            E::Spatial(G::BBox(x)) => x,
            _ => panic!("Not a BBox"),
        };
        assert!(!bbox1.is_2d());

        // should also succeed when coordinate sequence contains no, or other
        // sorts of whitespaces wherever they're placed...
        let x = cql2::geom_expression(G2);
        assert!(x.is_ok());
        let g = x.unwrap();
        assert!(matches!(g, E::Spatial(G::BBox(_))));
        let bbox2 = match g {
            E::Spatial(G::BBox(x)) => x,
            _ => panic!("Not a BBox"),
        };
        assert!(!bbox2.is_2d());

        assert_eq!(bbox1.west(), bbox2.west());
        assert_eq!(bbox1.east(), bbox2.east());
        assert_eq!(bbox1.south(), bbox2.south());
        assert_eq!(bbox1.north(), bbox2.north());
        assert_eq!(bbox1.z_min(), bbox2.z_min());
        assert_eq!(bbox1.z_max(), bbox2.z_max());
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_to_polygon() {
        const G1: &str = "BBOX(-180,-90,180,90)";
        const WKT: &str = "POLYGON ((-180 -90, 180 -90, 180 90, -180 90, -180 -90))";
        const G2: &str = "bbox(-180.0,-90.,-99999.0,180.0,90.0,100000.0)";

        let x1 = cql2::geom_expression(G1);
        assert!(x1.is_ok());
        let g1 = x1.unwrap();
        assert!(matches!(g1, E::Spatial(G::BBox(_))));
        let bbox1 = match g1 {
            E::Spatial(G::BBox(x)) => x,
            _ => panic!("Not a BBox"),
        };
        assert!(bbox1.is_2d());
        let g1 = bbox1.to_geos();
        assert!(g1.is_ok());
        let g1 = g1.unwrap();
        let wkt1 = g1.to_wkt().unwrap();
        assert_eq!(wkt1, WKT);

        let x2 = cql2::geom_expression(G2);
        assert!(x2.is_ok());
        let g2 = x2.unwrap();
        assert!(matches!(g2, E::Spatial(G::BBox(_))));
        let bbox2 = match g2 {
            E::Spatial(G::BBox(x)) => x,
            _ => panic!("Not a BBox"),
        };
        assert!(!bbox2.is_2d());
        let g2 = bbox2.to_geos();
        assert!(g2.is_ok());
        let g2 = g2.unwrap();
        let wkt2 = g2.to_wkt().unwrap();
        assert_eq!(wkt2, WKT);
    }

    #[test]
    fn test_antimeridian() -> Result<(), Box<dyn Error>> {
        const WKT: &str = "MULTIPOLYGON (((150 -90, 180 -90, 180 90, 150 90, 150 -90)), ((-150 -90, -150 90, -180 90, -180 -90, -150 -90)))";

        let bbox = BBox::from(vec![150.0, -90.0, -150.0, 90.0]);
        let mp = bbox.to_geos()?;
        assert_eq!(mp.get_type()?, "MultiPolygon");

        let wkt = mp.to_wkt()?;
        assert_eq!(wkt, WKT);

        let pt = Geometry::new_from_wkt("POINT(152 10)")?;

        pt.within(&mp)?;
        mp.contains(&pt)?;

        Ok(())
    }

    #[test]
    fn test_precision() -> Result<(), Box<dyn Error>> {
        const WKT: &str = "BBOX (6.043073, 50.128052, 6.242751, 49.902226)";

        let bbox_xy = vec![
            6.043073357781111,
            50.128051662794235,
            6.242751092156993,
            49.90222565367873,
        ];

        let bbox = BBox::from(bbox_xy);
        let wkt = bbox.to_wkt_fmt(6);
        assert_eq!(wkt, WKT);

        Ok(())
    }
}
