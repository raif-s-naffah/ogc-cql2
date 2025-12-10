// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Basic spatial type facades visible from this library.
//!

mod bbox;
mod collection;
mod line;
mod lines;
mod point;
mod points;
mod polygon;
mod polygons;

pub use bbox::*;
pub use collection::*;
pub use line::*;
pub use lines::*;
pub use point::*;
pub use points::*;
pub use polygon::*;
pub use polygons::*;

use crate::{MyError, config::config, crs::CRS, srid::SRID, text::cql2::wkt, wkb::*};
use core::fmt;
use geos::{ConstGeometry, Geom, Geometry, GeometryTypes};
use tracing::error;

// type aliases to silence clippy + work nicely w/ macros...
pub(crate) type XY1V = Vec<f64>;
pub(crate) type XY2V = Vec<Vec<f64>>;
pub(crate) type XY3V = Vec<Vec<Vec<f64>>>;
pub(crate) type XY4V = Vec<Vec<Vec<Vec<f64>>>>;

/// Ensure a float only has a fixed number of decimal digits in its fractional
/// part.
fn ensure_precision(x: &f64) -> f64 {
    let d = 10.0_f64.powi(
        config()
            .default_precision()
            .try_into()
            .expect("Failed coercing DEFAULT_PRECISION"),
    );
    (x * d).round() / d
}

/// Geometry type variants handled by this library.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub enum G {
    /// Undefined geometry.
    #[default]
    Null,

    /// Point geometry.
    Point(Point),
    /// Line geometry.
    Line(Line),
    /// Polygon geometry.
    Polygon(Polygon),
    /// Point collection.
    Points(Points),
    /// Line collection.
    Lines(Lines),
    /// Polygon collection.
    Polygons(Polygons),
    /// Mixed collection excluding BBOX.
    Vec(Geometries),
    /// Bounding box geometry.
    BBox(BBox),
}

/// Geometry Trait implemented by all [geometry][G] types in this library.
pub trait GTrait {
    /// Return TRUE if coordinates are 2D. Return FALSE otherwise.
    fn is_2d(&self) -> bool;

    /// Generate a WKT string representing this.
    ///
    /// This is a convenience method that calls the `to_wkt_fmt()` method w/ a
    /// pre-configured default precision value.
    ///
    /// See the documentation in `.env.template` for `DEFAULT_PRECISION`.
    fn to_wkt(&self) -> String {
        self.to_wkt_fmt(config().default_precision())
    }

    /// Generate a WKT string similar to the `to_wkt()`alternative but w/ a
    /// given `precision` paramter representing the number of digits to print
    /// after the decimal point. Note though that if `precision` is `0` only
    /// the integer part of the coordinate will be shown.
    ///
    /// Here are some examples...
    /// ```rust
    /// use ogc_cql2::prelude::*;
    /// # use std::error::Error;
    /// # fn test() -> Result<(), Box<dyn Error>> {
    ///     let g = G::try_from("LINESTRING(-180 -45,0 -45)")?;
    ///     assert_eq!(g.to_wkt_fmt(1), "LINESTRING (-180.0 -45.0, 0.0 -45.0)");
    ///     // ...
    ///     let g = G::try_from("POINT(-46.035560 -7.532500)")?;
    ///     assert_eq!(g.to_wkt_fmt(0), "POINT (-46 -7)");
    /// # Ok(())
    /// # }
    /// ```
    fn to_wkt_fmt(&self, precision: usize) -> String;

    /// Check if all geometry coordinates fall w/in a given CRS's Area-of-Use,
    /// aka Extent-of-Validity.
    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError>;

    /// Return the name/type of this geometry.
    fn type_(&self) -> &str;

    /// Return the Spatial Reference IDentifier of this.
    fn srid(&self) -> SRID;
}

impl GTrait for G {
    fn is_2d(&self) -> bool {
        match self {
            G::Point(x) => x.is_2d(),
            G::Line(x) => x.is_2d(),
            G::Polygon(x) => x.is_2d(),
            G::Points(x) => x.is_2d(),
            G::Lines(x) => x.is_2d(),
            G::Polygons(x) => x.is_2d(),
            G::Vec(x) => x.is_2d(),
            G::BBox(x) => x.is_2d(),
            _ => unreachable!("N/A for this geometry type"),
        }
    }

    fn to_wkt_fmt(&self, precision: usize) -> String {
        match self {
            G::Point(x) => x.to_wkt_fmt(precision),
            G::Line(x) => x.to_wkt_fmt(precision),
            G::Polygon(x) => x.to_wkt_fmt(precision),
            G::Points(x) => x.to_wkt_fmt(precision),
            G::Lines(x) => x.to_wkt_fmt(precision),
            G::Polygons(x) => x.to_wkt_fmt(precision),
            G::Vec(x) => x.to_wkt_fmt(precision),
            G::BBox(x) => x.to_wkt_fmt(precision),
            _ => unreachable!("N/A for this geometry type"),
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        match self {
            G::Point(x) => x.check_coordinates(crs),
            G::Line(x) => x.check_coordinates(crs),
            G::Polygon(x) => x.check_coordinates(crs),
            G::Points(x) => x.check_coordinates(crs),
            G::Lines(x) => x.check_coordinates(crs),
            G::Polygons(x) => x.check_coordinates(crs),
            G::Vec(x) => x.check_coordinates(crs),
            G::BBox(x) => x.check_coordinates(crs),
            _ => unreachable!("N/A for this geometry type"),
        }
    }

    fn type_(&self) -> &str {
        match self {
            G::Point(x) => x.type_(),
            G::Line(x) => x.type_(),
            G::Polygon(x) => x.type_(),
            G::Points(x) => x.type_(),
            G::Lines(x) => x.type_(),
            G::Polygons(x) => x.type_(),
            G::Vec(x) => x.type_(),
            G::BBox(x) => x.type_(),
            _ => unreachable!("N/A for this geometry type"),
        }
    }

    fn srid(&self) -> SRID {
        match self {
            G::Point(x) => x.srid(),
            G::Line(x) => x.srid(),
            G::Polygon(x) => x.srid(),
            G::Points(x) => x.srid(),
            G::Lines(x) => x.srid(),
            G::Polygons(x) => x.srid(),
            G::Vec(x) => x.srid(),
            G::BBox(x) => x.srid(),
            _ => unreachable!("N/A for this geometry type"),
        }
    }
}

impl G {
    /// Return this if it was indeed a Point, `None` otherwise.
    pub fn as_point(&self) -> Option<&Point> {
        match self {
            G::Point(x) => Some(x),
            _ => None,
        }
    }

    /// Return this if it was indeed a Line, `None` otherwise.
    pub fn as_line(&self) -> Option<&Line> {
        match self {
            G::Line(x) => Some(x),
            _ => None,
        }
    }

    /// Return this if it was indeed a Polygon, `None` otherwise.
    pub fn as_polygon(&self) -> Option<&Polygon> {
        match self {
            G::Polygon(x) => Some(x),
            _ => None,
        }
    }

    /// Return this if it was indeed a Point collection, `None` otherwise.
    pub fn as_points(&self) -> Option<&Points> {
        match self {
            G::Points(x) => Some(x),
            _ => None,
        }
    }

    /// Return this if it was indeed a Line collection, `None` otherwise.
    pub fn as_lines(&self) -> Option<&Lines> {
        match self {
            G::Lines(x) => Some(x),
            _ => None,
        }
    }

    /// Return this if it was indeed a Polygon collection, `None` otherwise.
    pub fn as_polygons(&self) -> Option<&Polygons> {
        match self {
            G::Polygons(x) => Some(x),
            _ => None,
        }
    }

    // ----- GEOS related methods...

    pub(crate) fn to_geos(&self) -> Result<Geometry, MyError> {
        match self {
            G::Point(x) => x.to_geos(),
            G::Line(x) => x.to_geos(),
            G::Polygon(x) => x.to_geos(),
            G::Points(x) => x.to_geos(),
            G::Lines(x) => x.to_geos(),
            G::Polygons(x) => x.to_geos(),
            G::Vec(x) => x.to_geos(),
            G::BBox(x) => x.to_geos(),
            _ => unreachable!("N/A for this geometry type"),
        }
    }

    pub(crate) fn intersects(&self, other: &G) -> Result<bool, MyError> {
        let lhs = self.to_geos()?;
        let rhs = other.to_geos()?;
        let result = lhs.intersects(&rhs)?;
        Ok(result)
    }

    pub(crate) fn equals(&self, other: &G) -> Result<bool, MyError> {
        let lhs = self.to_geos()?;
        let rhs = other.to_geos()?;
        let result = lhs.equals(&rhs)?;
        Ok(result)
    }

    pub(crate) fn disjoint(&self, other: &G) -> Result<bool, MyError> {
        let lhs = self.to_geos()?;
        let rhs = other.to_geos()?;
        let result = lhs.disjoint(&rhs)?;
        Ok(result)
    }

    pub(crate) fn touches(&self, other: &G) -> Result<bool, MyError> {
        let lhs = self.to_geos()?;
        let rhs = other.to_geos()?;
        let result = lhs.touches(&rhs)?;
        Ok(result)
    }

    pub(crate) fn within(&self, other: &G) -> Result<bool, MyError> {
        let lhs = self.to_geos()?;
        let rhs = other.to_geos()?;
        let result = lhs.within(&rhs)?;
        Ok(result)
    }

    pub(crate) fn overlaps(&self, other: &G) -> Result<bool, MyError> {
        let lhs = self.to_geos()?;
        let rhs = other.to_geos()?;
        let result = lhs.overlaps(&rhs)?;
        Ok(result)
    }

    pub(crate) fn crosses(&self, other: &G) -> Result<bool, MyError> {
        let lhs = self.to_geos()?;
        let rhs = other.to_geos()?;
        let result = lhs.crosses(&rhs)?;
        Ok(result)
    }

    pub(crate) fn contains(&self, other: &G) -> Result<bool, MyError> {
        let lhs = self.to_geos()?;
        let rhs = other.to_geos()?;
        let result = lhs.contains(&rhs)?;
        Ok(result)
    }

    // ----- methods exposed for use by Functions...

    pub(crate) fn boundary(&self) -> Result<Self, MyError> {
        let g1 = self.to_geos()?;
        let g2 = g1.boundary()?;
        let it = G::try_from(g2)?;
        Ok(it)
    }

    pub(crate) fn buffer(&self, width: f64, quadsegs: i32) -> Result<Self, MyError> {
        let g1 = self.to_geos()?;
        let g2 = g1.buffer(width, quadsegs)?;
        let it = G::try_from(g2)?;
        Ok(it)
    }

    pub(crate) fn envelope(&self) -> Result<Self, MyError> {
        let g1 = self.to_geos()?;
        let g2 = g1.envelope()?;
        let it = G::try_from(g2)?;
        Ok(it)
    }

    pub(crate) fn centroid(&self) -> Result<Self, MyError> {
        let g1 = self.to_geos()?;
        let g2 = g1.get_centroid()?;
        let it = G::try_from(g2)?;
        Ok(it)
    }

    pub(crate) fn convex_hull(&self) -> Result<Self, MyError> {
        let g1 = self.to_geos()?;
        let g2 = g1.convex_hull()?;
        let it = G::try_from(g2)?;
        Ok(it)
    }

    pub(crate) fn get_x(&self) -> Result<f64, MyError> {
        if let Some(pt) = self.as_point() {
            Ok(pt.x())
        } else {
            Err(MyError::Runtime("This is NOT a Point".into()))
        }
    }

    pub(crate) fn get_y(&self) -> Result<f64, MyError> {
        if let Some(pt) = self.as_point() {
            Ok(pt.y())
        } else {
            Err(MyError::Runtime("This is NOT a Point".into()))
        }
    }

    pub(crate) fn get_z(&self) -> Result<f64, MyError> {
        if let Some(pt) = self.as_point() {
            if let Some(z) = pt.z() {
                Ok(z)
            } else {
                Err(MyError::Runtime("This is NOT a 3D Point".into()))
            }
        } else {
            Err(MyError::Runtime("This is NOT a Point".into()))
        }
    }

    // ----- methods used to accommodate GeoPackage related ops...

    pub(crate) fn to_sql(&self) -> Result<String, MyError> {
        match self {
            G::BBox(x) => x.to_sql(),
            x => {
                let wkt = x.to_wkt();
                let srid = self.srid().as_usize()?;
                Ok(format!("ST_GeomFromText('{wkt}', {srid})"))
            }
        }
    }

    // ----- crate-private methods invisible to the outside...

    pub(crate) fn set_srid_unchecked(&mut self, srid: &SRID) {
        match self {
            G::Point(x) => x.set_srid_unchecked(srid),
            G::Line(x) => x.set_srid_unchecked(srid),
            G::Polygon(x) => x.set_srid_unchecked(srid),
            G::Points(x) => x.set_srid_unchecked(srid),
            G::Lines(x) => x.set_srid_unchecked(srid),
            G::Polygons(x) => x.set_srid_unchecked(srid),
            G::Vec(x) => x.set_srid_unchecked(srid),
            G::BBox(x) => x.set_srid_unchecked(srid),
            _ => unreachable!("N/A for this geometry type"),
        }
    }
}

impl fmt::Display for G {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            G::Null => write!(f, ""),
            G::Point(x) => write!(f, "{x}"),
            G::Line(x) => write!(f, "{x}"),
            G::Polygon(x) => write!(f, "{x}"),
            G::Points(x) => write!(f, "{x}"),
            G::Lines(x) => write!(f, "{x}"),
            G::Polygons(x) => write!(f, "{x}"),
            G::Vec(x) => write!(f, "{x}"),
            G::BBox(x) => write!(f, "{x}"),
        }
    }
}

// Construct new instance from WKT string...
impl TryFrom<&str> for G {
    type Error = MyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut g = wkt(value).map_err(MyError::Text)?;
        // NOTE (rsn) 20251023 - WKT does not encode SRIDs.  assign configured
        // global default set in .env...
        g.set_srid_unchecked(config().default_srid());

        Ok(g)
    }
}

// Construct new instance from WKB byte array...
impl TryFrom<&[u8]> for G {
    type Error = MyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let wkb = GeoPackageBinary::try_from(value)?;
        Ok(wkb.geom())
    }
}

// Construct new instance from GEOS Geometry instance...
impl TryFrom<Geometry> for G {
    type Error = MyError;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value.geometry_type() {
            GeometryTypes::Point => {
                let g = Point::try_from(value)?;
                Ok(G::Point(g))
            }
            GeometryTypes::LineString | GeometryTypes::LinearRing => {
                let g = Line::try_from(value)?;
                Ok(G::Line(g))
            }
            GeometryTypes::Polygon => {
                let g = Polygon::try_from(value)?;
                Ok(G::Polygon(g))
            }
            GeometryTypes::MultiPoint => {
                let g = Points::try_from(value)?;
                Ok(G::Points(g))
            }
            GeometryTypes::MultiLineString => {
                let g = Lines::try_from(value)?;
                Ok(G::Lines(g))
            }
            GeometryTypes::MultiPolygon => {
                let g = Polygons::try_from(value)?;
                Ok(G::Polygons(g))
            }
            GeometryTypes::GeometryCollection => {
                let g = Geometries::try_from(value)?;
                Ok(G::Vec(g))
            }
            x => {
                let msg = format!("Unknown ({x:?}) geometry type");
                error!("Failed: {msg}");
                Err(MyError::Runtime(msg.into()))
            }
        }
    }
}

// Construct new instance from GEOS ConstGeometry instance...
impl TryFrom<ConstGeometry<'_>> for G {
    type Error = MyError;

    fn try_from(value: ConstGeometry) -> Result<Self, Self::Error> {
        match value.geometry_type() {
            GeometryTypes::Point => {
                let g = Point::try_from(value)?;
                Ok(G::Point(g))
            }
            GeometryTypes::LineString | GeometryTypes::LinearRing => {
                let g = Line::try_from(value)?;
                Ok(G::Line(g))
            }
            GeometryTypes::Polygon => {
                let g = Polygon::try_from(value)?;
                Ok(G::Polygon(g))
            }
            GeometryTypes::MultiPoint => {
                let g = Points::try_from(value)?;
                Ok(G::Points(g))
            }
            GeometryTypes::MultiLineString => {
                let g = Lines::try_from(value)?;
                Ok(G::Lines(g))
            }
            GeometryTypes::MultiPolygon => {
                let g = Polygons::try_from(value)?;
                Ok(G::Polygons(g))
            }
            GeometryTypes::GeometryCollection => {
                let g = Geometries::try_from(value)?;
                Ok(G::Vec(g))
            }
            x => {
                let msg = format!("Unknown ({x:?}) geometry type");
                error!("Failed: {msg}");
                Err(MyError::Runtime(msg.into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expr::E, text::cql2};
    use geos::Geom;
    use std::error::Error;

    #[test]
    #[tracing_test::traced_test]
    fn test_to_wkt() {
        const G: &str = r#"Polygon Z (
        (
            -49.88024    0.5      -75993.341684, 
             -1.5       -0.99999 -100000.0, 
              0.0        0.5          -0.333333, 
            -49.88024    0.5      -75993.341684
        ), (
            -65.887123   2.00001 -100000.0,
              0.333333 -53.017711 -79471.332949,
            180.0        0.0        1852.616704,
            -65.887123   2.00001 -100000.0
        ))"#;
        const WKT: &str = "POLYGON Z ((-49.880240 0.500000 -75993.341684, -1.500000 -0.999990 -100000.000000, 0.000000 0.500000 -0.333333, -49.880240 0.500000 -75993.341684), (-65.887123 2.000010 -100000.000000, 0.333333 -53.017711 -79471.332949, 180.000000 0.000000 1852.616704, -65.887123 2.000010 -100000.000000))";

        let exp = cql2::geom_expression(G);
        // tracing::debug!("exp = {:?}", exp);
        let spa = exp.expect("Failed parsing Polygon WKT");
        let g = match spa {
            E::Spatial(G::Polygon(x)) => x,
            _ => panic!("Not a Polygon..."),
        };
        // should be a 3D polygon...
        assert_eq!(g.is_2d(), false);

        let wkt = g.to_wkt_fmt(6);
        assert_eq!(WKT, wkt);
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_to_geos() -> Result<(), Box<dyn Error>> {
        let g = G::try_from("POINT(17.03 45.87)")?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Point(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "Point");
        assert_eq!(gg.get_x()?, 17.03);
        assert_eq!(gg.get_y()?, 45.87);
        assert!(!gg.has_z()?);

        let g = G::try_from("LINESTRING(-49.85 0.5, -1.5 -0.999, 0.0 0.5, -49.88 0.5)")?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Line(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "LineString");
        assert_eq!(gg.get_num_points()?, 4);
        assert_eq!(gg.get_start_point()?.get_x()?, -49.85);
        assert_eq!(gg.get_end_point()?.get_y()?, 0.5);

        let g = G::try_from(
            r#"PolyGon ((
            -0.333333   89.0, 
            -102.723546 -0.5, 
            -179.0     -89.0, 
            -1.9        89.0, 
            -0.0        89.0, 
            2.00001     -1.9, 
            -0.333333   89.0))"#,
        )?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Polygon(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "Polygon");
        assert_eq!(gg.get_num_interior_rings()?, 0);
        assert_eq!(gg.get_exterior_ring()?.get_num_coordinates()?, 7);

        // multi-stuff

        let g = G::try_from("MULTIPOINT(17.03 45.87, -0.33 89.02)")?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Points(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "MultiPoint");
        assert_eq!(gg.get_num_geometries()?, 2);
        assert_eq!(gg.get_geometry_n(0)?.get_x()?, 17.03);
        assert_eq!(gg.get_geometry_n(1)?.get_y()?, 89.02);

        let g = G::try_from(
            r#"MULTILINESTRING(
            (-49.85 0.5, -1.5 -0.999, 0.0 0.5), 
            (34.3 3.2, 0.1 0.2))"#,
        )?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Lines(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "MultiLineString");
        assert_eq!(gg.get_num_geometries()?, 2);
        assert_eq!(gg.get_geometry_n(0)?.get_start_point()?.get_x()?, -49.85);
        assert_eq!(gg.get_geometry_n(1)?.get_end_point()?.get_y()?, 0.2);

        let g = G::try_from(
            r#"MULTIPOLYGON (
            ((
                180.0 -16.0671326636424,
                180.0 -16.5552165666392,
                179.364142661964 -16.8013540769469,
                178.725059362997 -17.012041674368,
                178.596838595117 -16.63915,
                179.096609362997 -16.4339842775474,
                179.413509362997 -16.3790542775474,
                180.0 -16.0671326636424
            )),((
                178.12557 -17.50481,
                178.3736 -17.33992,
                178.71806 -17.62846,
                178.55271 -18.15059,
                177.93266 -18.28799,
                177.38146 -18.16432,
                177.28504 -17.72465,
                177.67087 -17.38114,
                178.12557 -17.50481
            )),((
                -179.793320109049 -16.0208822567412,
                -179.917369384765 -16.5017831356494,
                -180 -16.5552165666392,
                -180 -16.0671326636424,
                -179.793320109049 -16.0208822567412
            ))
        )"#,
        )?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Polygons(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "MultiPolygon");
        assert_eq!(gg.get_num_geometries()?, 3);
        let p1 = gg.get_geometry_n(0)?;
        assert_eq!(p1.get_type()?, "Polygon");
        assert_eq!(p1.get_exterior_ring()?.get_num_coordinates()?, 8);
        assert_eq!(p1.get_num_interior_rings()?, 0);

        let g = G::try_from(
            r#"GEOMETRYCOLLECTION(
            POINT(17.03 45.87), 
            LINESTRING(-49.85 0.5, -1.5 -0.999, 0.0 0.5, -49.88 0.5)
        )"#,
        )?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Vec(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "GeometryCollection");
        assert_eq!(gg.get_num_geometries()?, 2);
        Ok(())
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_geos() -> Result<(), Box<dyn Error>> {
        const G: &str = r#"MultiLineString(
        (-49.85 0.5, -1.5   -0.999,  0.0 0.5, -49.88 0.5 ),
        (-65.87 2.01, 0.33 -53.07, 180.0 0)
        )"#;

        let exp = cql2::geom_expression(G);
        // tracing::debug!("exp = {:?}", exp);
        let spa = exp.expect("Failed parsing Polygon WKT");
        let g = match spa {
            E::Spatial(G::Lines(x)) => x,
            _ => panic!("Not a Lines..."),
        };
        assert_eq!(g.is_2d(), true);
        assert_eq!(g.num_lines(), 2);

        let geos = g.to_geos().expect("Failed converting to GEOS geometry");
        assert_eq!(geos.get_num_geometries()?, g.num_lines());
        let l1 = geos.get_geometry_n(0)?;
        assert_eq!(l1.get_num_coordinates()?, 4);
        let l2 = geos.get_geometry_n(1)?;
        assert_eq!(l2.get_num_coordinates()?, 3);

        Ok(())
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_new_from_wkt() -> Result<(), Box<dyn Error>> {
        const PT: &str = "POINT (-46.03556 -7.5325)";
        const LS: &str = "LINESTRING (-180 -45, 0 -45)";
        const P: &str = "POLYGON ((-180 -90, -90 -90, -90 90, -180 90, -180 -90), (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50))";
        const MPT: &str = "MULTIPOINT ((7.02 49.92), (90 180))";
        // const MPT2: &str = "MULTIPOINT (7.02 49.92, 90 180)";
        const MLS: &str = "MULTILINESTRING ((-180 -45, 0 -45), (0 45, 180 45))";
        const MP: &str = r#"MULTIPOLYGON(
            ((-180 -90, -90 -90, -90 90, -180 90, -180 -90),
             (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)),
            ((0 0, 10 0, 10 10, 0 10, 0 0))
        )"#;
        const MG: &str = r#"GEOMETRYCOLLECTION(
            POINT(7.02 49.92),
            POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))
        )"#;

        let pt = Geometry::new_from_wkt(PT);
        assert!(pt.is_ok());
        assert_eq!(pt?.to_wkt()?, PT);

        let ls = Geometry::new_from_wkt(LS);
        assert!(ls.is_ok());
        // tracing::debug!("ls = {}", ls?.to_wkt()?);
        assert_eq!(ls?.to_wkt()?, LS);

        let poly = Geometry::new_from_wkt(P);
        assert!(poly.is_ok());
        // tracing::debug!("poly = {}", poly?.to_wkt()?);
        assert_eq!(poly?.to_wkt()?, P);

        let points = Geometry::new_from_wkt(MPT);
        assert!(points.is_ok());
        // tracing::debug!("points = {}", points?.to_wkt()?);
        assert_eq!(points?.to_wkt()?, MPT);

        let lines = Geometry::new_from_wkt(MLS);
        assert!(lines.is_ok());
        // tracing::debug!("lines = {}", lines?.to_wkt()?);
        assert_eq!(lines?.to_wkt()?, MLS);

        let polys = Geometry::new_from_wkt(MP);
        assert!(polys.is_ok());
        // tracing::debug!("polys = {}", polys?.to_wkt()?);
        assert_eq!(polys?.get_type()?, "MultiPolygon");

        let geometries = Geometry::new_from_wkt(MG);
        assert!(geometries.is_ok());
        assert_eq!(geometries?.get_type()?, "GeometryCollection");

        Ok(())
    }

    #[test]
    fn test_point_in_polygon() -> Result<(), Box<dyn Error>> {
        const WKT1: &str = "POINT(-46.03556 -7.5325)";
        const WKT2: &str =
            "POLYGON((-65.887123 2.00001, 0.333333 -53.017711, 180.0 0.0, -65.887123 2.00001))";

        let pt = Geometry::new_from_wkt(WKT1).expect("Failed parsing point");
        let polygon = Geometry::new_from_wkt(WKT2).expect("Failed parsing polygon");

        pt.within(&polygon)?;
        // so is the inverse...
        polygon.contains(&pt)?;

        Ok(())
    }

    #[test]
    fn test_try_from_wkt() -> Result<(), Box<dyn Error>> {
        // Test Vector triplet consisting of (a) a test vector input, (b) expected
        // WKT output, and (c) number of decimal digits in fraction to use.
        #[rustfmt::skip]
        const TV: [(&str, &str, usize); 8] = [
            (
                "POINT(-46.035560 -7.532500)",
                "POINT (-46.03556 -7.53250)",
                5
            ), (
                "LINESTRING   (-180 -45,   0 -45)",
                "LINESTRING (-180.0 -45.0, 0.0 -45.0)", 
                1
            ), (
                r#"POLYGON (
                    (-180 -90, -90 -90, -90 90, -180 90, -180 -90),
                    (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)
                )"#,
                "POLYGON ((-180 -90, -90 -90, -90 90, -180 90, -180 -90), (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50))",
                0
            ), (
                "MULTIPOINT ((7.02 49.92), (90 180))",
                "MULTIPOINT (7.02 49.92, 90.00 180.00)",
                2
            ), (
                "MULTILINESTRING ((-180 -45, 0 -45), (0 45, 180 45))",
                "MULTILINESTRING ((-180.0 -45.0, 0.0 -45.0), (0.0 45.0, 180.0 45.0))",
                1
            ), (
                r#"MULTIPOLYGON((
                    (-180 -90, -90 -90, -90 90, -180 90, -180 -90),
                    (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)
                ), (
                    (0 0, 10 0, 10 10, 0 10, 0 0)
                ))"#,
                "MULTIPOLYGON (((-180 -90, -90 -90, -90 90, -180 90, -180 -90), (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50)), ((0 0, 10 0, 10 10, 0 10, 0 0)))",
                0
            ), (
                "GEOMETRYCOLLECTION(POINT(7.02 49.92),POLYGON((0 0, 10 0, 10 10, 0 10, 0 0)))",
                "GEOMETRYCOLLECTION (POINT (7.0 49.9), POLYGON ((0.0 0.0, 10.0 0.0, 10.0 10.0, 0.0 10.0, 0.0 0.0)))",
                1
            ), (
                "BBOX(51.43,2.54,55.77,6.40)",
                "BBOX (51.43, 2.54, 55.77, 6.40)",
                2
            ),
        ];

        for (ndx, (wkt, expected, precision)) in TV.iter().enumerate() {
            // if let Ok(g) = G::try_from_wkt(wkt) {
            if let Ok(g) = G::try_from(*wkt) {
                let actual = g.to_wkt_fmt(*precision);
                assert_eq!(actual, *expected);
            } else {
                panic!("Failed parsing WKT at index #{ndx}")
            }
        }

        Ok(())
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_geos_envelope() -> Result<(), Box<dyn Error>> {
        let mut geom = geos::Geometry::new_from_wkt("LINESTRING(0 0, 1 3)")?;
        geom.set_srid(3587);

        let envelope = geom.envelope()?;
        let srid = envelope.get_srid()?;
        tracing::debug!("envelope SRS id = {srid}");
        assert_eq!(envelope.to_wkt()?, "POLYGON ((0 0, 1 0, 1 3, 0 3, 0 0))");

        Ok(())
    }

    #[test]
    #[ignore = "GEOS possible bug"]
    fn test_geos_wkt() -> Result<(), Box<dyn Error>> {
        let expected = "POINT (1.0 3.0)";

        let geom = geos::Geometry::new_from_wkt("POINT(1 3)")?;
        let actual = geom.to_wkt_precision(0)?;

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    #[ignore = "GEOS possible bug"]
    fn test_geos_wkt_writer() -> Result<(), Box<dyn Error>> {
        let expected = "POINT (1.00 3.00)";

        let geom = geos::Geometry::new_from_wkt("POINT(1 3)")?;
        let mut writer = geos::WKTWriter::new()?;
        writer.set_rounding_precision(2);
        let actual = writer.write(&geom)?;

        assert_eq!(actual, expected);
        Ok(())
    }
}
