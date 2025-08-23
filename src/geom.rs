// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]
#![allow(dead_code)]

//! Basic spatial type facades visible from this library.
//!

use crate::{DEFAULT_PRECISION, MyError, crs::CRS, text::cql2::wkt};
use core::fmt;
use geos::{CoordSeq, Geom, Geometry};
use tracing::error;

/// Ensure a float only has a fixed number of decimal digits in its fractional
/// part.
fn ensure_precision(x: &f64) -> f64 {
    let d = 10.0_f64.powi(
        DEFAULT_PRECISION
            .try_into()
            .expect("Failed coercing DEFAULT_PRECISION"),
    );
    (x * d).round() / d
}

/// Geometry type variants handled by this library.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum G {
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

pub(crate) trait GTrait {
    /// Return TRUE if coordinates are 2D. Return FALSE otherwise.
    fn is_2d(&self) -> bool;

    /// Generate a WKT string representing this.
    fn to_wkt(&self) -> String;

    /// Check if all geometry coordinates fall w/in a given CRS's Area-of-Use,
    /// aka Extent-of-Validity.
    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError>;

    /// Try creating a `geos` [Geometry] instance from this.
    fn to_geos(&self) -> Result<Geometry, MyError>;
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
        }
    }

    fn to_wkt(&self) -> String {
        match self {
            G::Point(x) => x.to_wkt(),
            G::Line(x) => x.to_wkt(),
            G::Polygon(x) => x.to_wkt(),
            G::Points(x) => x.to_wkt(),
            G::Lines(x) => x.to_wkt(),
            G::Polygons(x) => x.to_wkt(),
            G::Vec(x) => x.to_wkt(),
            G::BBox(x) => x.to_wkt(),
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
        }
    }

    fn to_geos(&self) -> Result<Geometry, MyError> {
        match self {
            G::Point(x) => x.to_geos(),
            G::Line(x) => x.to_geos(),
            G::Polygon(x) => x.to_geos(),
            G::Points(x) => x.to_geos(),
            G::Lines(x) => x.to_geos(),
            G::Polygons(x) => x.to_geos(),
            G::Vec(x) => x.to_geos(),
            G::BBox(x) => x.to_geos(),
        }
    }
}

impl G {
    /// Parse the input string as a WKT and if valid return a geometry instance
    /// representing it.
    pub fn try_from_wkt(s: &str) -> Result<Self, MyError> {
        let g = wkt(s).map_err(|x| MyError::Runtime(format!("Not WKT: {x}").into()))?;
        Ok(g)
    }
}

impl fmt::Display for G {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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

// ----- Point ----------------------------------------------------------------

/// 2D or 3D point geometry.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Point {
    coord: Vec<f64>,
}

impl GTrait for Point {
    fn is_2d(&self) -> bool {
        self.coord.len() == 2
    }

    fn to_wkt(&self) -> String {
        if self.is_2d() {
            format!("POINT ({})", Self::coords_as_txt(self.as_2d()))
        } else {
            format!("POINT Z ({})", Self::coords_as_txt(self.as_3d()))
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        crs.check_point(&self.coord)
    }

    fn to_geos(&self) -> Result<Geometry, MyError> {
        Self::to_geos_impl(&self.coord)
    }
}

impl Point {
    pub(crate) fn from(coord: Vec<f64>) -> Self {
        // shape the input coordinates to a fixed precision; i.e. a fixed
        // number of decimals so `geos` can reliably assert equality of
        // coordinate values.
        let coord = Self::ensure_precision_impl(&coord);
        Point { coord }
    }

    fn to_geos_impl(xy: &[f64]) -> Result<Geometry, MyError> {
        let xy = CoordSeq::new_from_vec(&[xy]).map_err(MyError::Geos)?;
        Geometry::create_point(xy).map_err(MyError::Geos)
    }

    /// Outputs given coordinates sequentially seperated by a space.
    /// Values will have [`DEFAULT_PRECISION`] fractional decimal digits.
    pub(crate) fn coords_as_txt(coord: &[f64]) -> String {
        if coord.len() == 2 {
            format!("{:.2$} {:.2$}", coord[0], coord[1], DEFAULT_PRECISION)
        } else {
            format!(
                "{:.3$} {:.3$} {:.3$}",
                coord[0], coord[1], coord[2], DEFAULT_PRECISION
            )
        }
    }

    /// Return the 2D coordinates of this point.
    fn as_2d(&self) -> &[f64; 2] {
        self.coord
            .as_slice()
            .try_into()
            .expect("Failed coercing Point to 2D")
    }

    /// Return the 3D coordinates of this point.
    fn as_3d(&self) -> &[f64; 3] {
        self.coord
            .as_slice()
            .try_into()
            .expect("Failed coercing Point to 3D")
    }

    fn ensure_precision_impl(coord: &[f64]) -> Vec<f64> {
        coord.iter().map(ensure_precision).collect()
    }

    // Return the 1st coordinate of this.
    #[cfg(test)]
    fn x(&self) -> f64 {
        self.coord[0]
    }

    // Return the 2nd coordinate of this.
    #[cfg(test)]
    fn y(&self) -> f64 {
        self.coord[1]
    }

    // Return the 3rd coordinate of this if it's a 3D one. Return `None` otherwise.
    #[cfg(test)]
    fn z(&self) -> Option<f64> {
        if self.coord.len() == 2 {
            None
        } else {
            Some(self.coord[2])
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pt (...)")
    }
}

// ----- Line -----------------------------------------------------------------

/// 2D or 3D line-string geometry.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Line {
    coord: Vec<Vec<f64>>,
}

impl GTrait for Line {
    fn is_2d(&self) -> bool {
        self.coord[0].len() == 2
    }

    fn to_wkt(&self) -> String {
        if self.is_2d() {
            format!("LINESTRING {}", Self::coords_as_txt(&self.coord))
        } else {
            format!("LINESTRING Z {}", Self::coords_as_txt(&self.coord))
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        crs.check_line(&self.coord)
    }

    fn to_geos(&self) -> Result<Geometry, MyError> {
        Self::to_geos_impl(&self.coord)
    }
}

impl Line {
    pub(crate) fn from(coord: Vec<Vec<f64>>) -> Self {
        let coord = Self::ensure_precision_impl(&coord);
        Line { coord }
    }

    fn to_geos_impl(xy: &[Vec<f64>]) -> Result<Geometry, MyError> {
        let vertices: Vec<&[f64]> = xy.iter().map(|x| x.as_slice()).collect();
        let xy = CoordSeq::new_from_vec(&vertices).map_err(MyError::Geos)?;
        Geometry::create_line_string(xy).map_err(MyError::Geos)
    }

    fn ensure_precision_impl(coord: &[Vec<f64>]) -> Vec<Vec<f64>> {
        coord
            .iter()
            .map(|x| Point::ensure_precision_impl(x))
            .collect()
    }

    pub(crate) fn coords_as_txt(coord: &[Vec<f64>]) -> String {
        let points: Vec<String> = coord.iter().map(|x| Point::coords_as_txt(x)).collect();
        format!("({})", points.join(", "))
    }

    // Return the number of vertices.
    fn size(&self) -> usize {
        self.coord.len()
    }

    // Return TRUE if the first and last vertices coincide. Return FALSE
    // otherwise.
    fn is_closed(&self) -> bool {
        self.coord.first() == self.coord.last()
    }

    // Return TRUE if this cnsists of at least 4 points w/ the first and last
    // ones coinciding. Return FALSE otherwise.
    fn is_ring(&self) -> bool {
        self.size() > 3 && self.is_closed()
    }

    fn first(&self) -> Option<&Vec<f64>> {
        self.coord.first()
    }

    fn last(&self) -> Option<&Vec<f64>> {
        self.coord.last()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LINE (...)")
    }
}

// ----- Polygon --------------------------------------------------------------

/// 2D or 3D polygon geometry.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Polygon {
    rings: Vec<Vec<Vec<f64>>>,
}

impl GTrait for Polygon {
    fn is_2d(&self) -> bool {
        self.rings[0][0].len() == 2
    }

    fn to_wkt(&self) -> String {
        if self.is_2d() {
            format!("POLYGON {}", Self::coords_as_txt(&self.rings))
        } else {
            format!("POLYGON Z {}", Self::coords_as_txt(&self.rings))
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        crs.check_polygon(&self.rings)
    }

    fn to_geos(&self) -> Result<Geometry, MyError> {
        Self::to_geos_impl(&self.rings)
    }
}

impl Polygon {
    pub(crate) fn from(rings: Vec<Vec<Vec<f64>>>) -> Self {
        let rings = Self::ensure_precision_impl(&rings);
        Polygon { rings }
    }

    fn ensure_precision_impl(rings: &[Vec<Vec<f64>>]) -> Vec<Vec<Vec<f64>>> {
        rings
            .iter()
            .map(|r| Line::ensure_precision_impl(r))
            .collect()
    }

    pub(crate) fn coords_as_txt(rings: &[Vec<Vec<f64>>]) -> String {
        let rings: Vec<String> = rings.iter().map(|x| Line::coords_as_txt(x)).collect();
        format!("({})", rings.join(", "))
    }

    fn to_geos_impl(rings: &[Vec<Vec<f64>>]) -> Result<Geometry, MyError> {
        let vertices: Vec<&[f64]> = rings[0].iter().map(|x| x.as_slice()).collect();
        let xy = CoordSeq::new_from_vec(&vertices).map_err(MyError::Geos)?;
        let exterior = Geometry::create_linear_ring(xy).map_err(MyError::Geos)?;

        let mut interiors = vec![];
        for hole in &rings[1..] {
            let vertices: Vec<&[f64]> = hole.iter().map(|x| x.as_slice()).collect();
            let xy = CoordSeq::new_from_vec(&vertices).map_err(MyError::Geos)?;
            let hole = Geometry::create_linear_ring(xy).map_err(MyError::Geos)?;
            interiors.push(hole);
        }

        Geometry::create_polygon(exterior, interiors).map_err(MyError::Geos)
    }

    // Return the outer (always the first) linear ring contour of this.
    fn outer(&self) -> &Vec<Vec<f64>> {
        self.rings[0].as_ref()
    }

    fn outer_as_ring(&self) -> Line {
        Line::from(self.outer().to_vec())
    }

    // Return TRUE if this has holes; i.e. more than 1 linear ring. Return
    // FALSE otherwise.
    fn has_holes(&self) -> bool {
        self.rings.len() > 1
    }

    // Return the array of inner (holes) linear rings of this.
    fn inners(&self) -> &[Vec<Vec<f64>>] {
        &self.rings.as_slice()[1..]
    }
}

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "POLYGON (...)")
    }
}

// ----- Points ---------------------------------------------------------------

/// Collection of point geometries.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Points {
    points: Vec<Vec<f64>>,
}

impl GTrait for Points {
    fn is_2d(&self) -> bool {
        self.points[0].len() == 2
    }

    fn to_wkt(&self) -> String {
        if self.is_2d() {
            format!("MULTIPOINT {}", Self::coords_as_txt(&self.points))
        } else {
            format!("MULTIPOINT Z {}", Self::coords_as_txt(&self.points))
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

    fn to_geos(&self) -> Result<Geometry, MyError> {
        let mut points: Vec<Geometry> = vec![];
        for p in &self.points {
            let g = Point::to_geos_impl(p)?;
            points.push(g);
        }
        Geometry::create_multipoint(points).map_err(MyError::Geos)
    }
}

impl Points {
    pub(crate) fn from(points: Vec<Vec<f64>>) -> Self {
        let points = points
            .iter()
            .map(|x| Point::ensure_precision_impl(x))
            .collect();
        Points { points }
    }

    pub(crate) fn coords_as_txt(points: &[Vec<f64>]) -> String {
        let points: Vec<String> = points.iter().map(|x| Point::coords_as_txt(x)).collect();
        format!("({})", points.join(", "))
    }

    #[cfg(test)]
    // Return the number of points in this instance.
    pub(crate) fn size(&self) -> usize {
        self.points.len()
    }
}

impl fmt::Display for Points {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "POINTS (...)")
    }
}

// ----- Lines ----------------------------------------------------------------

/// Collection of line-string geometries.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Lines {
    lines: Vec<Vec<Vec<f64>>>,
}

impl GTrait for Lines {
    fn is_2d(&self) -> bool {
        self.lines[0][0].len() == 2
    }

    fn to_wkt(&self) -> String {
        if self.is_2d() {
            format!(
                "MULTILINESTRING {}",
                Self::coords_as_txt(self.lines.as_slice())
            )
        } else {
            format!(
                "MULTILINESTRING Z {}",
                Self::coords_as_txt(self.lines.as_slice())
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

    fn to_geos(&self) -> Result<Geometry, MyError> {
        let mut lines: Vec<Geometry> = vec![];
        for l in &self.lines {
            let g = Line::to_geos_impl(l)?;
            lines.push(g);
        }
        Geometry::create_multiline_string(lines).map_err(MyError::Geos)
    }
}

impl Lines {
    pub(crate) fn from(lines: Vec<Vec<Vec<f64>>>) -> Self {
        let lines = lines
            .iter()
            .map(|x| Line::ensure_precision_impl(x))
            .collect();
        Lines { lines }
    }

    pub(crate) fn coords_as_txt(lines: &[Vec<Vec<f64>>]) -> String {
        let lines: Vec<String> = lines
            .iter()
            .map(|x| Line::coords_as_txt(x.as_slice()))
            .collect();
        format!("({})", lines.join(", "))
    }

    // Return the number of lines in this instance.
    #[cfg(test)]
    fn size(&self) -> usize {
        self.lines.len()
    }

    // Return the lines in this as a slice.
    #[cfg(test)]
    fn lines(&self) -> &[Vec<Vec<f64>>] {
        self.lines.as_slice()
    }
}

impl fmt::Display for Lines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LINES (...)")
    }
}

// ----- Polygons -------------------------------------------------------------

/// Collection of polygon geometries.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Polygons {
    polygons: Vec<Vec<Vec<Vec<f64>>>>,
}

impl GTrait for Polygons {
    fn is_2d(&self) -> bool {
        self.polygons[0][0][0].len() == 2
    }

    fn to_wkt(&self) -> String {
        if self.is_2d() {
            format!(
                "MULTIPOLYGON {}",
                Self::coords_as_txt(self.polygons.as_slice())
            )
        } else {
            format!(
                "MULTIPOLYGON Z {}",
                Self::coords_as_txt(self.polygons.as_slice())
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

    fn to_geos(&self) -> Result<Geometry, MyError> {
        let mut polygons: Vec<Geometry> = vec![];
        for p in &self.polygons {
            let g = Polygon::to_geos_impl(p)?;
            polygons.push(g);
        }
        Geometry::create_multipolygon(polygons).map_err(MyError::Geos)
    }
}

impl Polygons {
    pub(crate) fn from(polygons: Vec<Vec<Vec<Vec<f64>>>>) -> Self {
        let polygons = polygons
            .iter()
            .map(|x| Polygon::ensure_precision_impl(x))
            .collect();
        Polygons { polygons }
    }

    pub(crate) fn coords_as_txt(polygons: &[Vec<Vec<Vec<f64>>>]) -> String {
        let polygons: Vec<String> = polygons
            .iter()
            .map(|x| Polygon::coords_as_txt(x.as_slice()))
            .collect();
        format!("({})", polygons.join(", "))
    }
}

impl fmt::Display for Polygons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "POLYGONS (...)")
    }
}

// ----- Geometries -----------------------------------------------------------

/// Collection of mixed geometries.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Geometries {
    items: Vec<G>,
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

    fn to_wkt(&self) -> String {
        let items: Vec<String> = self
            .items
            .iter()
            .map(|x| match x {
                G::Point(x) => x.to_wkt(),
                G::Line(x) => x.to_wkt(),
                G::Polygon(x) => x.to_wkt(),
                G::Points(x) => x.to_wkt(),
                G::Lines(x) => x.to_wkt(),
                G::Polygons(x) => x.to_wkt(),
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

    fn to_geos(&self) -> Result<Geometry, MyError> {
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
        Geometry::create_geometry_collection(items?).map_err(MyError::Geos)
    }
}

impl Geometries {
    pub(crate) fn from(items: Vec<G>) -> Self {
        Geometries { items }
    }
}

impl fmt::Display for Geometries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GEOMETRIES (...)")
    }
}

// ----- BBox -----------------------------------------------------------------

/// 2D or 3D bounding box.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BBox {
    w: f64,             // west bound longitude
    s: f64,             // south bound latitude
    z_min: Option<f64>, // minimum elevation
    e: f64,             // east bound longitude
    n: f64,             // north bound latitude
    z_max: Option<f64>, // maximum elevation
}

impl GTrait for BBox {
    fn is_2d(&self) -> bool {
        self.z_min.is_none()
    }

    // IMPORTANT: BBOX does not have a standard WKT representation. this
    // function first creates a `geos` equivalent polygon or multi-polygon
    // Geometry (depending on whether the coordinates cross the antimeridian
    // or not), then generate the result's WKT.
    fn to_wkt(&self) -> String {
        if let Ok(p2d) = self.to_geos_impl() {
            p2d.to_wkt().unwrap()
        } else {
            panic!("Unable to convert BBOX to 2D Polygon")
        }
    }

    fn check_coordinates(&self, crs: &CRS) -> Result<(), MyError> {
        crs.check_point([self.w, self.s].as_ref())?;
        crs.check_point([self.e, self.n].as_ref())?;
        Ok(())
    }

    fn to_geos(&self) -> Result<Geometry, MyError> {
        self.to_geos_impl()
    }
}

impl BBox {
    /// (from [1]) If the vertical axis is included, the third and the sixth
    /// number are the bottom and the top of the 3-dimensional bounding box.
    ///
    /// [1]: https://docs.ogc.org/is/21-065r2/21-065r2.html#basic-spatial-data-types
    pub(crate) fn from(xy: Vec<f64>) -> Self {
        if xy.len() == 4 {
            // BBox { w: xy[0], s: xy[1], z_min: None, e: xy[2], n: xy[3], z_max: None }
            BBox {
                w: ensure_precision(&xy[0]),
                s: ensure_precision(&xy[1]),
                z_min: None,
                e: ensure_precision(&xy[2]),
                n: ensure_precision(&xy[3]),
                z_max: None,
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
            }
        }
    }

    // Convert this to one 2D polygon, or in the case of a box that spans the
    // antimeridian, a 2D multi-polygon.
    fn to_geos_impl(&self) -> Result<Geometry, MyError> {
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
    use crate::{expr::E, text::cql2};
    use approx::assert_relative_eq;
    use std::error::Error;

    const TOLERANCE: f64 = 1.0E-3;

    #[test]
    fn test_pt_equality() {
        let p1 = Point {
            coord: vec![1., 1.],
        };
        let p2 = Point {
            coord: vec![1.0, 1.0],
        };
        let p3 = Point {
            coord: vec![1.0, 1.1],
        };
        let p4 = Point {
            coord: vec![1.1, 1.0],
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
    fn test_pt_comparison() {
        let p1 = Point {
            coord: vec![1.0, 1.0],
        };
        let p2 = Point {
            coord: vec![1.0, 1.1],
        };
        let p3 = Point {
            coord: vec![1.1, 1.0],
        };

        assert!(p1 < p2);
        assert!(p1 < p3);
        assert!(p2 < p3);
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_point() {
        const G: &str = r#"point (-3.508362 -1.754181)"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        // tracing::debug!("exp = {:?}", exp);
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
    // #[tracing_test::traced_test]
    fn test_line() {
        const G: &str = r#"LineString(43.72992 -79.2998, 43.73005 -79.2991, 43.73006 -79.2984,
                   43.73140 -79.2956, 43.73259 -79.2950, 43.73266 -79.2945,
                   43.73320 -79.2936, 43.73378 -79.2936, 43.73486 -79.2917)"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        // tracing::debug!("exp = {:?}", exp);
        let spa = exp.unwrap();
        let g = match spa {
            E::Spatial(G::Line(x)) => x,
            _ => panic!("Not a Line..."),
        };
        assert_eq!(g.is_2d(), true);
        assert_eq!(g.size(), 9);

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
    // #[tracing_test::traced_test]
    fn test_polygon2d() {
        const G: &str = r#"PolyGon ((-0.333333 89.0, -102.723546 -0.5, -179.0 -89.0, -1.9 89.0, -0.0 89.0, 2.00001 -1.9, -0.333333 89.0))"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        // tracing::debug!("exp = {:?}", exp);
        let spa = exp.unwrap();
        let g = match spa {
            E::Spatial(G::Polygon(x)) => x,
            _ => panic!("Not a Polygon..."),
        };
        assert_eq!(g.is_2d(), true);

        let outer_ring = g.outer_as_ring();
        assert!(outer_ring.is_ring());
        assert!(outer_ring.is_closed());
        assert_eq!(outer_ring.size(), 7);

        // has no holes...
        assert!(!g.has_holes());
        assert!(g.inners().is_empty());
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_polygon3d() {
        const G: &str = r#"POLYGON Z ((-49.88024 0.5 -75993.341684, -1.5 -0.99999 -100000.0, 0.0 0.5 -0.333333, -49.88024 0.5 -75993.341684), (-65.887123 2.00001 -100000.0, 0.333333 -53.017711 -79471.332949, 180.0 0.0 1852.616704, -65.887123 2.00001 -100000.0))"#;

        let exp = cql2::geom_expression(G);
        assert!(exp.is_ok());
        // tracing::debug!("exp = {:?}", exp);
        let spa = exp.unwrap();
        let g = match spa {
            E::Spatial(G::Polygon(x)) => x,
            _ => panic!("Not a Polygon..."),
        };
        assert_eq!(g.is_2d(), false);

        let outer_ring = g.outer_as_ring();
        assert!(outer_ring.is_ring());
        assert!(outer_ring.is_closed());
        assert_eq!(outer_ring.size(), 4);

        // has 1 hole...
        assert!(g.has_holes());
        assert_eq!(g.inners().len(), 1);
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_multiline() {
        const G: &str = r#"MultiLineString(
        ( -1.9 -0.99999, 75.292574 1.5,     -0.5 -4.016458, -31.708594 -74.743801, 179.0 -90.0 ),
        (-1.9 -1.1,      1.5      8.547371))"#;

        let exp = cql2::geom_expression(G);
        // tracing::debug!("exp: {:?}", exp);
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
        assert_eq!(lines.len(), g.size());
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_bbox() {
        const G1: &str = "bbox(-128.098193, -1.1, -99999.0, 180.0, 90.0, 100000.0)";
        const G2: &str = "bbox(-128.098193,-1.1, -99999.0,180.0 , \t90.0, \n 100000.0)";

        let x = cql2::geom_expression(G1);
        // tracing::debug!("x = {:?}", x);
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
        // tracing::debug!("x = {:?}", x);
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
    // #[tracing_test::traced_test]
    fn test_bbox_to_polygon() {
        const G1: &str = "BBOX(-180,-90,180,90)";
        const WKT: &str = "POLYGON ((-180 -90, 180 -90, 180 90, -180 90, -180 -90))";
        const G2: &str = "bbox(-180.0,-90.,-99999.0,180.0,90.0,100000.0)";

        let x1 = cql2::geom_expression(G1);
        // tracing::debug!("x1 = {:?}", x1);
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
        // tracing::debug!("g1 = {:?}", g1.to_wkt());
        let wkt1 = g1.to_wkt().unwrap();
        assert_eq!(wkt1, WKT);

        let x2 = cql2::geom_expression(G2);
        // tracing::debug!("x2 = {:?}", x2);
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
        // tracing::debug!("g2 = {:?}", g2.to_wkt());
        let wkt2 = g2.to_wkt().unwrap();
        assert_eq!(wkt2, WKT);
    }

    #[test]
    // #[tracing_test::traced_test]
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

        let wkt = g.to_wkt();
        // tracing::debug!("wkt = {}", wkt);
        assert_eq!(WKT, wkt);
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_to_geos() -> Result<(), Box<dyn Error>> {
        let g = G::try_from_wkt("POINT(17.03 45.87)")?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Point(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "Point");
        assert_eq!(gg.get_x()?, 17.03);
        assert_eq!(gg.get_y()?, 45.87);
        assert!(!gg.has_z()?);

        let g = G::try_from_wkt("LINESTRING(-49.85 0.5, -1.5 -0.999, 0.0 0.5, -49.88 0.5)")?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Line(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "LineString");
        assert_eq!(gg.get_num_points()?, 4);
        assert_eq!(gg.get_start_point()?.get_x()?, -49.85);
        assert_eq!(gg.get_end_point()?.get_y()?, 0.5);

        let g = G::try_from_wkt(
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

        let g = G::try_from_wkt("MULTIPOINT(17.03 45.87, -0.33 89.02)")?;
        // tracing::debug!("g = {g:?}");
        assert!(matches!(g, G::Points(_)));
        // tracing::debug!("g (wkt) = {}", g.to_wkt());
        let gg = g.to_geos()?;
        assert_eq!(gg.get_type()?, "MultiPoint");
        assert_eq!(gg.get_num_geometries()?, 2);
        assert_eq!(gg.get_geometry_n(0)?.get_x()?, 17.03);
        assert_eq!(gg.get_geometry_n(1)?.get_y()?, 89.02);

        let g = G::try_from_wkt(
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

        let g = G::try_from_wkt(
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

        let g = G::try_from_wkt(
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
    // #[tracing_test::traced_test]
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
        assert_eq!(g.size(), 2);

        let geos = g.to_geos().expect("Failed converting to GEOS geometry");
        assert_eq!(geos.get_num_geometries()?, g.size());
        let l1 = geos.get_geometry_n(0)?;
        assert_eq!(l1.get_num_coordinates()?, 4);
        let l2 = geos.get_geometry_n(1)?;
        assert_eq!(l2.get_num_coordinates()?, 3);

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_new_from_wkt() -> Result<(), Box<dyn Error>> {
        const PT: &str = "POINT (-46.03556 -7.5325)";
        const LS: &str = "LINESTRING (-180 -45, 0 -45)";
        const P: &str = "POLYGON ((-180 -90, -90 -90, -90 90, -180 90, -180 -90), (-120 -50, -100 -50, -100 -40, -120 -40, -120 -50))";
        const MPT: &str = "MULTIPOINT ((7.02 49.92), (90 180))";
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
        // tracing::debug!("polys#type = {}", polys?.get_type()?);
        assert_eq!(polys?.get_type()?, "MultiPolygon");

        let geometries = Geometry::new_from_wkt(MG);
        assert!(geometries.is_ok());
        // tracing::debug!("geometries = {}", geometries?.to_wkt()?);
        // tracing::debug!("geometries#type = {}", geometries?.get_type()?);
        assert_eq!(geometries?.get_type()?, "GeometryCollection");

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
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
    // #[tracing_test::traced_test]
    fn test_bbox_antimeridian() -> Result<(), Box<dyn Error>> {
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
    #[should_panic]
    fn test_invalid_pt_xy() {
        let pt = Point::from(vec![90.0, 180.0]);
        let crs = CRS::default();
        pt.check_coordinates(&crs).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_line_xy() {
        let line = Line::from(vec![vec![0.0, 45.0], vec![90.0, 180.0], vec![45.0, 45.0]]);
        let crs = CRS::default();
        line.check_coordinates(&crs).unwrap();
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_touches() -> Result<(), Box<dyn Error>> {
        const WKT1: &str = "POLYGON ((0 -90, 0 0, 180 0, 180 -90, 0 -90))";
        const WKT2: &str = "POLYGON ((-180 -90, -180 90, 180 90, 180 -90, -180 -90))";

        let p1 = Geometry::new_from_wkt(WKT1)?;
        let p2 = Geometry::new_from_wkt(WKT2)?;

        // although p1 and p2 share a segment of their bottom side, their
        // interiors are NOT disjoint and as such they are considered to
        // "touch" each other.
        assert!(!p1.touches(&p2)?);

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_point_precision() -> Result<(), Box<dyn Error>> {
        const XYZ: [f64; 3] = [-16.0671326636424, -17.012041674368, 179.096609362997];
        const WKT: &str = "POINT Z (-16.067133 -17.012042 179.096609)";

        let pt = Point::from(XYZ.to_vec());
        // tracing::debug!("pt = {pt:?}");
        let wkt = pt.to_wkt();
        // tracing::debug!("wkt = {wkt}");
        assert_eq!(wkt, WKT);

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_line_precision() -> Result<(), Box<dyn Error>> {
        const WKT: &str = "LINESTRING (82.400480 30.411477, 82.722734 30.365046)";

        let line_xy = vec![
            vec![82.400479770847, 30.4114773625851],
            vec![82.7227340026191, 30.3650460881709],
        ];

        let line = Line::from(line_xy);
        // tracing::debug!("line = {line:?}");
        let wkt = line.to_wkt();
        // tracing::debug!("wkt = {wkt}");
        assert_eq!(wkt, WKT);

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_bbox_precision() -> Result<(), Box<dyn Error>> {
        const WKT: &str = "POLYGON ((6.043073 50.128052, 6.242751 50.128052, 6.242751 49.902226, 6.043073 49.902226, 6.043073 50.128052))";

        let bbox_xy = vec![
            6.043073357781111,
            50.128051662794235,
            6.242751092156993,
            49.90222565367873,
        ];

        let bbox = BBox::from(bbox_xy);
        // tracing::debug!("bbox = {bbox:?}");
        let wkt = bbox.to_wkt();
        // tracing::debug!("wkt = {wkt}");
        assert_eq!(wkt, WKT);

        Ok(())
    }
}
