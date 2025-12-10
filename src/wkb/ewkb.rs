// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Representation of a PostGIS EWKB encoded geometry binary.
//!

use crate::{
    G, Line, Lines, MyError, Point, Points, Polygon, Polygons, SRID,
    wkb::{ByteOrder, line, lines, point, points, polygon, polygons, signed, unsigned},
};

/// PostGIS Extended Well Known Binary encoded geometry.
pub struct PostGisBinary {
    geom: G,
    srid: SRID,
}

impl PostGisBinary {
    /// Return the inner geometry object.
    pub fn geom(self) -> G {
        self.geom
    }

    /// Return the SRID of this.
    #[allow(dead_code)]
    pub fn srid(&self) -> &SRID {
        &self.srid
    }
}

impl TryFrom<&[u8]> for PostGisBinary {
    type Error = MyError;

    fn try_from(ba: &[u8]) -> Result<Self, Self::Error> {
        let mut pos = 0;

        // 1st byte is byte-order...
        let b1 = ba[0] & 0x07;
        let bo = &ByteOrder::from(b1 & 0x01);
        pos += 1;

        // next 4 bytes are the geometry type...
        // IMPORTANT (rsn) 20251123 - only the least significant 12 bits --i
        // couldn't find any documentation on what values like 0x06000020 mean!
        // the only document i could find was: "06-103r4 Implementation Specification
        // for Geographic Information - Simple feature access - Part 1 Common
        // Architecture v1.2.1-1" which lists ZM geometries w/ a maximum type
        // identifier of (decimal) 3016 (12 bits) and would expect values of 6,
        // 1006, 2006, or 3006 for a polygon with optional Z and M attributes.
        let gt = unsigned(bo, ba, pos)? & 0x0F_FF;
        pos += 4;

        // next 4 bytes are the SRID...
        let srs_id = signed(bo, ba, pos)?;
        let srid = SRID::try_from(srs_id)?;
        pos += 4;

        let (geom, _span) = parse_ewkb(gt, srid, bo, ba, pos)?;

        Ok(Self { geom, srid })
    }
}

// given a geometry type, an SRID, a byte order indicator, consume enough bytes
// from a given byte array.
fn parse_ewkb(
    gt: u32,
    srid: SRID,
    bo: &ByteOrder,
    ba: &[u8],
    pos: usize,
) -> Result<(G, usize), MyError> {
    match gt {
        1 => {
            let (xy, span) = point(bo, ba, pos)?;
            let g = Point::from_xy_and_srid(xy, srid);
            Ok((G::Point(g), span))
        }
        2 => {
            let (xy, span) = line(bo, ba, pos)?;
            let g = Line::from_xy_and_srid(xy, srid);
            Ok((G::Line(g), span))
        }
        3 => {
            let (xy, span) = polygon(bo, ba, pos)?;
            let g = Polygon::from_xy_and_srid(xy, srid);
            Ok((G::Polygon(g), span))
        }
        4 => {
            let (xy, span) = points(bo, ba, pos)?;
            let g = Points::from_xy_and_srid(xy, srid);
            Ok((G::Points(g), span))
        }
        5 => {
            let (xy, span) = lines(bo, ba, pos)?;
            let g = Lines::from_xy_and_srid(xy, srid);
            Ok((G::Lines(g), span))
        }
        6 => {
            let (xy, span) = polygons(bo, ba, pos)?;
            let g = Polygons::from_xy_and_srid(xy, srid);
            Ok((G::Polygons(g), span))
        }
        7 => {
            let (xy, span) = collection(bo, ba, pos)?;
            let g = crate::Geometries::from_items_and_srid(xy, srid);
            Ok((G::Vec(g), span))
        }
        x => unreachable!("Unsupported ({x}) geometry type"),
    }
}

// parse a series of (a) 1-byte representing the byte order to use when
// recognizing numbers, (b) a 4-byte geometry type identifier, (c) a 4-byte
// SRID in which the geometry coordinates are expressed, and (d) the elements
// of that geometry type.
// return the discovered geometry structure and the index of the last
// consumed byte from the input slice.
fn wkb_geometry(ba: &[u8], start: usize) -> Result<(G, usize), MyError> {
    let mut pos = start;

    // 1st byte is byte-order...
    let b1 = ba[pos] & 0x07;
    let bo = &ByteOrder::from(b1 & 0x01);
    pos += 1;

    // next 4 bytes are the geometry type...
    // IMPORTANT (rsn) 20251123 - only the least significant 12 bits --i
    // couldn't find any documentation on what values like 0x06000020 mean!
    // the only document i could find was: "06-103r4 Implementation Specification
    // for Geographic Information - Simple feature access - Part 1 Common
    // Architecture v1.2.1-1" which lists ZM geometries w/ a maximum type
    // identifier of (decimal) 3016 (12 bits) and would expect values of 6,
    // 1006, 2006, or 3006 for a polygon with optional Z and M attributes.
    let gt = unsigned(bo, ba, pos)? & 0x0FFF;
    pos += 4;

    // next 4 bytes are the SRID...
    let srs_id = signed(bo, ba, pos)?;
    let srid = SRID::try_from(srs_id)?;
    pos += 4;

    parse_ewkb(gt, srid, bo, ba, pos)
}

fn collection(bo: &ByteOrder, ba: &[u8], start: usize) -> Result<(Vec<G>, usize), MyError> {
    let num_geometries = unsigned(bo, ba, start)?;
    let mut span = 4;
    let mut xy: Vec<G> = Vec::with_capacity(usize::try_from(num_geometries)?);
    for _ in 0..num_geometries {
        let (g, offset) = wkb_geometry(ba, start + span)?;
        xy.push(g);
        span += offset;
    }
    Ok((xy, span))
}

#[cfg(test)]
mod tests {
    use super::PostGisBinary as EWKB;
    use crate::GTrait;
    use std::error::Error;

    #[test]
    #[tracing_test::traced_test]
    fn test_pg_polygons() -> Result<(), Box<dyn Error>> {
        #[rustfmt::skip]
        let bytes: &[u8] = &vec![
            0x01,
            0x06, 0x00, 0x00, 0x20,
            0xe6, 0x10, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00,
            // polygon #1
            0x01,
            0x03, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x66, 0x40, 0x72, 0xd6, 0x32, 0x9b, 0x2f, 0x11, 0x30, 0xc0,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x66, 0x40, 0xaa, 0xe9, 0x43, 0xac, 0x22, 0x8e, 0x30, 0xc0,
            0xdc, 0x06, 0x83, 0x0e, 0xa7, 0x6b, 0x66, 0x40, 0xcd, 0x00, 0x71, 0x8a, 0x25, 0xcd, 0x30, 0xc0,
            0x66, 0x77, 0xb1, 0xaf, 0x33, 0x57, 0x66, 0x40, 0x8d, 0x99, 0xc5, 0x29, 0x15, 0x03, 0x31, 0xc0,
            0x99, 0xe0, 0x40, 0x4d, 0x19, 0x53, 0x66, 0x40, 0x09, 0x3d, 0x9b, 0x55, 0x9f, 0xa3, 0x30, 0xc0,
            0xeb, 0xd1, 0x84, 0x6c, 0x17, 0x63, 0x66, 0x40, 0x56, 0x0b, 0xf7, 0x97, 0x19, 0x6f, 0x30, 0xc0,
            0x32, 0xd5, 0xfc, 0x77, 0x3b, 0x6d, 0x66, 0x40, 0x9a, 0x79, 0x7d, 0xb3, 0x09, 0x61, 0x30, 0xc0,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x66, 0x40, 0x72, 0xd6, 0x32, 0x9b, 0x2f, 0x11, 0x30, 0xc0,
            // polygon #2
            0x01,
            0x03, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x09, 0x00, 0x00, 0x00,
            0x7b, 0x6b, 0x60, 0xab, 0x04, 0x44, 0x66, 0x40, 0x9a, 0xb1, 0x68, 0x3a, 0x3b, 0x81, 0x31, 0xc0,
            0x24, 0xb9, 0xfc, 0x87, 0xf4, 0x4b, 0x66, 0x40, 0x9e, 0x41, 0x43, 0xff, 0x04, 0x57, 0x31, 0xc0,
            0x1b, 0x12, 0xf7, 0x58, 0xfa, 0x56, 0x66, 0x40, 0x1b, 0xd8, 0x2a, 0xc1, 0xe2, 0xa0, 0x31, 0xc0,
            0x82, 0xc5, 0xe1, 0xcc, 0xaf, 0x51, 0x66, 0x40, 0xca, 0x1a, 0xf5, 0x10, 0x8d, 0x26, 0x32, 0xc0,
            0x33, 0xc9, 0xc8, 0x59, 0xd8, 0x3d, 0x66, 0x40, 0x36, 0x93, 0x6f, 0xb6, 0xb9, 0x49, 0x32, 0xc0,
            0x6e, 0x17, 0x9a, 0xeb, 0x34, 0x2c, 0x66, 0x40, 0x27, 0x14, 0x22, 0xe0, 0x10, 0x2a, 0x32, 0xc0,
            0xa9, 0xc1, 0x34, 0x0c, 0x1f, 0x29, 0x66, 0x40, 0xe1, 0x0b, 0x93, 0xa9, 0x82, 0xb9, 0x31, 0xc0,
            0xc3, 0xbb, 0x5c, 0xc4, 0x77, 0x35, 0x66, 0x40, 0x8c, 0x32, 0x1b, 0x64, 0x92, 0x61, 0x31, 0xc0,
            0x7b, 0x6b, 0x60, 0xab, 0x04, 0x44, 0x66, 0x40, 0x9a, 0xb1, 0x68, 0x3a, 0x3b, 0x81, 0x31, 0xc0,
            // polygon #3
            0x01,
            0x03, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00,
            0xf0, 0x73, 0xda, 0xe0, 0x62, 0x79, 0x66, 0xc0, 0x2e, 0xc5, 0x21, 0x8a, 0x58, 0x05, 0x30, 0xc0,
            0x65, 0x3d, 0x0a, 0x17, 0x5b, 0x7d, 0x66, 0xc0, 0x6a, 0x4c, 0x0d, 0xdc, 0x74, 0x80, 0x30, 0xc0,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x66, 0xc0, 0xaa, 0xe9, 0x43, 0xac, 0x22, 0x8e, 0x30, 0xc0,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x66, 0xc0, 0x72, 0xd6, 0x32, 0x9b, 0x2f, 0x11, 0x30, 0xc0,
            0xf0, 0x73, 0xda, 0xe0, 0x62, 0x79, 0x66, 0xc0, 0x2e, 0xc5, 0x21, 0x8a, 0x58, 0x05, 0x30, 0xc0,
        ];

        let ewkb = EWKB::try_from(bytes)?;
        assert_eq!(ewkb.srid().as_usize()?, 4326);

        let g = ewkb.geom();
        assert!(g.is_2d());
        assert_eq!(g.type_(), "MultiPolygon");
        let polygons = g.as_polygons().unwrap();
        assert_eq!(polygons.num_polygons(), 3);

        let p1 = polygons.polygons().nth(0).expect("Expected a Polygon");
        // all polygons have 1 ring...
        assert_eq!(p1.len(), 1);
        // 1st ring of 1st polygon has 8 points...
        let r11 = &p1[0];
        assert_eq!(r11.len(), 8);
        let p2 = polygons.polygons().nth(1).expect("Expected a Polygon");
        assert_eq!(p2.len(), 1);
        // 1st ring of 2nd polygon has 9 points...
        let r21 = &p2[0];
        assert_eq!(r21.len(), 9);
        let p3 = polygons.polygons().nth(2).expect("Expected a Polygon");
        assert_eq!(p3.len(), 1);
        // 1st ring of 3rd polygon has 5 points...
        let r31 = &p3[0];
        assert_eq!(r31.len(), 5);

        Ok(())
    }
}
