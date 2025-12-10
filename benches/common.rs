// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Common artifacts for benchamrking.
//!

use futures::{StreamExt, TryStreamExt};
use ogc_cql2::prelude::*;
use serde::Deserialize;
use sqlx::FromRow;
use std::{collections::HashMap, error::Error};
use tokio::runtime::{Builder, Runtime};

const COUNTRIES_CSV: &str = "./tests/samples/data/ne_110m_admin_0_countries.csv";
const GPKG_URL: &str = "sqlite:tests/samples/data/ne110m4cql2.gpkg";
const COUNTRIES_TBL: &str = "ne_110m_admin_0_countries";
const PG_DB_NAME: &str = "cql2";

// code to read and group all text-encoded samples in an array.
fn text_samples_to_vec() -> Result<(), Box<dyn std::error::Error>> {
    let mut it = vec![];
    let mut count = 0;
    for entry in walkdir::WalkDir::new("tests/samples/text") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        let src = std::fs::read_to_string(entry.path()).expect("Failed reading sample text");
        it.push(src);

        count += 1;
    }

    assert_eq!(count, 120);
    assert_eq!(it.len(), 120);
    println!("it = {it:#?}");
    Ok(())
}

pub(crate) const TEXT_SAMPLES: [&str; 120] = [
    "T_DURING(INTERVAL(starts_at, ends_at), INTERVAL('2005-01-10', '2010-02-10'))\n",
    "depth BETWEEN 100.0 and 150.0\n",
    "S_WITHIN(POLYGON Z ((-49.88024 0.5 -75993.341684, -1.5 -0.99999 -100000.0, 0.0 0.5 -0.333333, -49.88024 0.5 -75993.341684), (-65.887123 2.00001 -100000.0, 0.333333 -53.017711 -79471.332949, 180.0 0.0 1852.616704, -65.887123 2.00001 -100000.0)), \"geometry\")\n",
    "\"value\" <= (2 ^ \"foo\")\n",
    "\"value\" <= 10\n",
    "\"value\" <> (22.1 * \"foo\")\n",
    "    eo:cloud_cover >= 0.1\nAND eo:cloud_cover <= 0.2\nAND landsat:wrs_row=28\nAND landsat:wrs_path=203\n",
    "(owner LIKE 'mike%' OR owner LIKE 'Mike%') AND floors<4\n",
    "floors>5 AND S_WITHIN(geometry,BBOX(-118,33.8,-117.9,34))\n",
    "eo:cloud_cover IN (0.1,0.2)\n",
    "owner NOT LIKE '%Mike%'\n",
    "Foo(\"geometry\") = TRUE\n",
    "\"value\" = (2 / \"foo\")\n",
    "ACCENTI(\"owner\") = ACCENTI('Beyonce\u{301}')\n",
    "eo:cloud_cover < 0.1 AND landsat:wrs_row=28 AND landsat:wrs_path=203\n",
    "A_EQUALS(('a', TRUE, 1.0, 8), \"values\")\n",
    "name LIKE 'Smith%'\n",
    "\"value\" NOT IN ('a', 'b', 'c')\n",
    "T_DURING(INTERVAL(touchdown, liftOff), INTERVAL('1969-07-16T13:32:00Z', '1969-07-24T16:50:35Z'))\n",
    "swimming_pool=true AND (floors>5 \n                    OR  material LIKE 'brick%'\n                    OR  material LIKE '%brick')\n",
    "avg(windSpeed) < 4\n",
    "S_CROSSES(road,POLYGON((43.7286 -79.2986, 43.7311 -79.2996, 43.7323 -79.2972,\n                        43.7326 -79.2971, 43.7350 -79.2981, 43.7350 -79.2982,\n                        43.7352 -79.2982, 43.7357 -79.2956, 43.7337 -79.2948,\n                        43.7343 -79.2933, 43.7339 -79.2923, 43.7327 -79.2947,\n                        43.7320 -79.2942, 43.7322 -79.2937, 43.7306 -79.2930,\n                        43.7303 -79.2930, 43.7299 -79.2928, 43.7286 -79.2986)))\n",
    "eo:instrument LIKE 'OLI%'\n                AND S_INTERSECTS(footprint,POLYGON((43.5845 -79.5442,\n                                                    43.6079 -79.4893,\n                                                    43.5677 -79.4632,\n                                                    43.6129 -79.3925,\n                                                    43.6223 -79.3238,\n                                                    43.6576 -79.3163,\n                                                    43.7945 -79.1178,\n                                                    43.8144 -79.1542,\n                                                    43.8555 -79.1714,\n                                                    43.7509 -79.6390,\n                                                    43.5845 -79.5442)))\n",
    "T_MEETS(INTERVAL('2005-01-10', '2010-02-10'), INTERVAL(starts_at, ends_at))\n",
    "S_WITHIN(POLYGON ((-49.88024 0.5 -75993.341684, -1.5 -0.99999 -100000.0, 0.0 0.5 -0.333333, -49.88024 0.5 -75993.341684), (-65.887123 2.00001 -100000.0, 0.333333 -53.017711 -79471.332949, 180.0 0.0 1852.616704, -65.887123 2.00001 -100000.0)), \"geometry\")\n",
    "\"name\" NOT LIKE 'foo%' AND \"value\" > 10\n",
    "\"value\" BETWEEN 10 AND 20\n",
    "value = - foo * 2.0 + \"bar\" / 6.1234 - \"x\" ^ 2.0\n",
    "NOT \"value\" IS NULL\n",
    "taxes <= 500\n",
    "S_CROSSES(LINESTRING(43.72992 -79.2998, 43.73005 -79.2991, 43.73006 -79.2984,\n                     43.73140 -79.2956, 43.73259 -79.2950, 43.73266 -79.2945,\n                     43.73320 -79.2936, 43.73378 -79.2936, 43.73486 -79.2917),\n        POLYGON((43.7286 -79.2986, 43.7311 -79.2996, 43.7323 -79.2972, 43.7326 -79.2971,\n                 43.7350 -79.2981, 43.7350 -79.2982, 43.7352 -79.2982, 43.7357 -79.2956,\n                 43.7337 -79.2948, 43.7343 -79.2933, 43.7339 -79.2923, 43.7327 -79.2947,\n                 43.7320 -79.2942, 43.7322 -79.2937, 43.7306 -79.2930, 43.7303 -79.2930,\n                 43.7299 -79.2928, 43.7286 -79.2986)))\n",
    "(floors>5 AND material='brick') OR swimming_pool=true\n",
    "NOT \"name\" LIKE 'foo%'\n",
    "ACCENTI(etat_vol) = ACCENTI('débárquér')\n",
    "floors>5\n",
    "\"value\" IN (1.0, 2.0, 3.0)\n",
    "A_CONTAINS(layer:ids, ('layers-ca','layers-us'))\n",
    "T_EQUALS(\"updated_at\", DATE('1851-04-29'))\n",
    "T_FINISHES(INTERVAL(starts_at, ends_at), INTERVAL('1991-10-07', '2010-02-10T05:29:20.073225Z'))\n",
    "S_INTERSECTS(geometry,POLYGON((-10.0 -10.0,10.0 -10.0,10.0 10.0,-10.0 -10.0)))\n",
    "S_EQUALS(MULTIPOINT ((180.0 -0.5), (179.0 -47.121701), (180.0 -0.0), (33.470475 -0.99999), (179.0 -15.333062)), \"geometry\")\n",
    "T_STARTS(INTERVAL(starts_at, ends_at), INTERVAL('1991-10-07T08:21:06.393262Z', '..'))\n",
    "T_BEFORE(built, DATE('2015-01-01'))\n",
    "S_TOUCHES(\"geometry\", MULTILINESTRING ((-1.9 -0.99999, 75.292574 1.5, -0.5 -4.016458, -31.708594 -74.743801, 179.0 -90.0),(-1.9 -1.1, 1.5 8.547371)))\n",
    "owner LIKE '%Jones%'\n",
    "\"name\" LIKE CASEI('FOO%')\n",
    "S_EQUALS(GEOMETRYCOLLECTION (POINT (1.9 2.00001), POINT (0.0 -2.00001), MULTILINESTRING ((-2.00001 -0.0, -77.292642 -0.5, -87.515626 -0.0, -180.0 12.502773, 21.204842 -1.5, -21.878857 -90.0)), POINT (1.9 0.5), LINESTRING (179.0 1.179148, -148.192487 -65.007816, 0.5 0.333333)), \"geometry\")\n",
    "NOT (floors<5) OR swimming_pool=true\n",
    "floors>5 AND swimming_pool=true\n",
    "S_INTERSECTS(geometry,POINT(36.319836 32.288087))\n",
    "T_CONTAINS(INTERVAL('2000-01-01T00:00:00Z', '2005-01-10T01:01:01.393216Z'), INTERVAL(starts_at, ends_at))\n",
    "T_INTERSECTS(event_time, INTERVAL('1969-07-16T05:32:00Z', '1969-07-24T16:50:35Z'))\n",
    "    eo:cloud_cover BETWEEN 0.1 AND 0.2\nAND landsat:wrs_row=28\nAND landsat:wrs_path=203\n",
    "ACCENTI(etat_vol) = ACCENTI('débárquér')\n",
    "swimming_pool = true\n",
    "T_INTERSECTS(INTERVAL(starts_at, ends_at), INTERVAL('1991-10-07T08:21:06.393262Z', '2010-02-10T05:29:20.073225Z'))\n",
    "T_FINISHEDBY(INTERVAL(starts_at, ends_at), INTERVAL('1991-10-07T08:21:06.393262Z', '2010-02-10T05:29:20.073225Z'))\n",
    "S_OVERLAPS(\"geometry\", BBOX(-179.912109, 1.9, 180.0, 16.897016))\n",
    "1 = (\"foo\" div 2)\n",
    "T_BEFORE(\"updated_at\", TIMESTAMP('2012-08-10T05:30:00.000000Z'))\n",
    "\"id\" = 'fa7e1920-9107-422d-a3db-c468cbc5d6df'\n",
    "cityName IN ('Toronto','Frankfurt','Tokyo','New York')\n",
    "A_OVERLAPS(\"values\", (TIMESTAMP('2012-08-10T05:30:00.000000Z'), DATE('2010-02-10'), FALSE))\n",
    "S_CONTAINS(\"geometry\", POINT (-3.508362 -1.754181))\n",
    "T_AFTER(\"updated_at\", DATE('2010-02-10'))\n",
    "T_DURING(INTERVAL(starts_at, ends_at), INTERVAL('2017-06-10T07:30:00Z', '2017-06-11T10:30:00Z'))\n",
    "CASEI(road_class) IN (CASEI('Οδος'),CASEI('Straße'))\n",
    "\"value\" = ((((-1 * \"foo\") * 2.0) + (\"bar\" / 6.1234)) - (\"x\" ^ 2.0))\n",
    "\"value\" IS NULL\n",
    "\"value\" < 10\n",
    "0 = (\"foo\" % 2)\n",
    "\"value\" > 10\n",
    "S_INTERSECTS(\"geometry\", BBOX(-128.098193, -1.1, -99999.0, 180.0, 90.0, 100000.0))\n",
    "owner LIKE 'Mike%'\n",
    "T_CONTAINS(INTERVAL('2000-01-01T00:00:00.000000Z', '2005-01-10T01:01:01.393216Z'), INTERVAL(starts_at, ends_at))\n",
    "    beamMode='ScanSAR Narrow'\nAND swathDirection='ascending'\nAND polarization='HH+VV+HV+VH'\nAND s_intersects(footprint,POLYGON((-77.117938 38.936860,\n                                    -77.040604 39.995648,\n                                    -76.910536 38.892912,\n                                    -77.039359 38.791753,\n                                    -77.047906 38.841462,\n                                    -77.034183 38.840655,\n                                    -77.033142 38.857490,\n                                    -77.117938 38.936860)))\n",
    "(NOT \"name\" LIKE 'foo%' AND \"value\" > 10)\n",
    "geometry IS NOT NULL\n",
    "eo:cloud_cover=0.1 OR eo:cloud_cover=0.2\n",
    "\"id\" <> 'fa7e1920-9107-422d-a3db-c468cbc5d6df'\n",
    "CASEI(\"owner\") = CASEI('somebody else')\n",
    "T_STARTEDBY(INTERVAL('1991-10-07T08:21:06.393262Z', '2010-02-10T05:29:20.073225Z'), INTERVAL(starts_at, ends_at))\n",
    "CASEI(road_class) IN (CASEI('Οδος'),CASEI('Straße'))\n",
    "\"name\" LIKE 'foo%'\n",
    "S_DISJOINT(\"geometry\", MULTIPOLYGON (((144.022387 45.176126, -1.1 0.0, 180.0 47.808086, 144.022387 45.176126))))\n",
    "balance-150.0 > 0\n",
    "NOT \"value\" IN ('a', 'b', 'c')\n",
    "city='Toronto'\n",
    "S_WITHIN(road,Buffer(geometry,10,'m'))\n",
    "S_CROSSES(\"geometry\", LINESTRING (172.03086 1.5, 1.1 -90.0, -159.757695 0.99999, -180.0 0.5, -12.111235 81.336403, -0.5 64.43958, 0.0 81.991815, -155.93831 90.0))\n",
    "S_WITHIN(location,BBOX(-118,33.8,-117.9,34))\n",
    "landsat:scene_id = 'LC82030282019133LGN00'\n",
    "\"value\" < (\"foo\" - 10)\n",
    "T_DURING(INTERVAL('1969-07-20T20:17:40Z', '1969-07-21T17:54:00Z'), INTERVAL('1969-07-16T13:32:00Z', '1969-07-24T16:50:35Z'))\n",
    "A_CONTAINEDBY(\"values\", ('a', 'b', 'c'))\n",
    "\"value\" NOT BETWEEN 10 AND 20\n",
    "category NOT IN (1,2,3,4)\n",
    "eo:instrument LIKE 'OLI%'\n",
    "T_METBY(INTERVAL('2010-02-10T05:29:20.073225Z', '2010-10-07'), INTERVAL(starts_at, ends_at))\n",
    "T_OVERLAPPEDBY(INTERVAL('1991-10-07T08:21:06.393262Z', '2010-02-10T05:29:20.073225Z'), INTERVAL(starts_at, ends_at))\n",
    "\"value\" > (\"foo\" + 10)\n",
    "FALSE <> Bar(\"geometry\", 100, 'a', 'b', FALSE)\n",
    "\"value\" >= 10\n",
    "S_EQUALS(POLYGON ((-0.333333 89.0, -102.723546 -0.5, -179.0 -89.0, -1.9 89.0, -0.0 89.0, 2.00001 -1.9, -0.333333 89.0)), \"geometry\")\n",
    "CASEI(geophys:SURVEY_NAME) LIKE CASEI('%calcutta%')\n",
    "T_AFTER(built,DATE('2012-06-05'))\n",
    "\"value\" IS NULL OR \"value\" BETWEEN 10 AND 20\n",
    "vehicle_height > (bridge_clearance-1)\n",
    "(\"value\" IS NULL OR \"value\" BETWEEN 10 AND 20)\n",
    "A_CONTAINS(\"values\", ('a', 'b', 'c'))\n",
    "T_DISJOINT(INTERVAL('..', '2005-01-10T01:01:01.393216Z'), INTERVAL(starts_at, ends_at))\n",
    "landsat:wrs_path IN ('153','154','15X')\n",
    "\"value\" IS NOT NULL\n",
    "T_OVERLAPS(INTERVAL(starts_at, ends_at), INTERVAL('1991-10-07T08:21:06.393262Z', '1992-10-09T08:08:08.393473Z'))\n",
    "avg(windSpeed)\n",
    "S_EQUALS(\n    POLYGON (\n        (-0.333333 89.0, -102.723546 -0.5, -179.0 -89.0, -1.9 89.0, -0.0 89.0, 2.00001 -1.9, -0.333333 89.0)\n    ),\n    \"geometry\"\n)\n",
    "\"name\" NOT LIKE 'foo%'\n",
    "updated >= date('1970-01-01')\n",
    "NOT \"value\" BETWEEN 10 AND 20\n",
    "T_BEFORE(updated_at, TIMESTAMP('2012-08-10T05:30:00Z'))\n",
];

// likewise for JSON-encoded ones...
fn json_samples_to_vec() -> Result<(), Box<dyn std::error::Error>> {
    let mut it = vec![];
    let mut count = 0;
    for entry in walkdir::WalkDir::new("tests/samples/json") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        if entry.path().ends_with("validate.sh") {
            continue;
        }
        let src = std::fs::read_to_string(entry.path()).expect("Failed reading sample text");
        it.push(src);

        count += 1;
    }

    assert_eq!(count, 109);
    assert_eq!(it.len(), 109);
    println!("it = {it:#?}");
    Ok(())
}

pub(crate) const JSON_SAMPLES: [&str; 109] = [
    "{\n  \"op\": \"<=\",\n  \"args\": [\n    { \"property\": \"value\" },\n    {\n      \"op\": \"^\",\n      \"args\": [ 2, { \"property\": \"foo\" } ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"t_finishes\",\n  \"args\": [\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] },\n    { \"interval\": [ \"1991-10-07\", \"2010-02-10T05:29:20.073225Z\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"beamMode\" },\n        \"ScanSAR Narrow\"\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"swathDirection\" },\n        \"ascending\"\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"polarization\" },\n        \"HH+VV+HV+VH\"\n      ]\n    },\n    {\n      \"op\": \"s_intersects\",\n      \"args\": [\n        {\n          \"property\": \"footprint\"\n        },\n        {\n          \"type\": \"Polygon\",\n          \"coordinates\": [\n            [ [ -77.117938, 38.936860 ],\n              [ -77.040604, 39.995648 ],\n              [ -76.910536, 38.892912 ],\n              [ -77.039359, 38.791753 ],\n              [ -77.047906, 38.841462 ],\n              [ -77.034183, 38.840655 ],\n              [ -77.033142, 38.857490 ],\n              [ -77.117938, 38.936860 ]\n            ]\n          ]\n        }\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"property\": \"value\" },\n    {\n      \"op\": \"-\",\n      \"args\": [\n        {\n          \"op\": \"+\",\n          \"args\": [\n            {\n              \"op\": \"*\",\n              \"args\": [\n                {\n                  \"op\": \"*\",\n                  \"args\": [ -1, { \"property\": \"foo\" } ]\n                },\n                2.0\n              ]\n            },\n            {\n              \"op\": \"/\",\n              \"args\": [ { \"property\": \"bar\" }, 6.1234 ]\n            }\n          ]\n        },\n        {\n          \"op\": \"^\",\n          \"args\": [ { \"property\": \"x\" }, 2.0 ]\n        }\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"s_crosses\",\n  \"args\": [\n    {\n      \"type\": \"LineString\",\n      \"coordinates\": [\n        [ 43.72992, -79.2998 ], [ 43.73005, -79.2991 ], [ 43.73006, -79.2984 ],\n        [ 43.73140, -79.2956 ], [ 43.73259, -79.2950 ], [ 43.73266, -79.2945 ],\n        [ 43.73320, -79.2936 ], [ 43.73378, -79.2936 ], [ 43.73486, -79.2917 ]\n      ]\n    },\n    {\n      \"type\": \"Polygon\",\n      \"coordinates\": [\n        [\n          [ 43.7286, -79.2986 ], [ 43.7311, -79.2996 ], [ 43.7323, -79.2972 ],\n          [ 43.7326, -79.2971 ], [ 43.7350, -79.2981 ], [ 43.7350, -79.2982 ],\n          [ 43.7352, -79.2982 ], [ 43.7357, -79.2956 ], [ 43.7337, -79.2948 ],\n          [ 43.7343, -79.2933 ], [ 43.7339, -79.2923 ], [ 43.7327, -79.2947 ],\n          [ 43.7320, -79.2942 ], [ 43.7322, -79.2937 ], [ 43.7306, -79.2930 ],\n          [ 43.7303, -79.2930 ], [ 43.7299, -79.2928 ], [ 43.7286, -79.2986 ]\n        ]\n      ]\n    }\n  ]\n}\n",
    "{ \"op\": \"avg\", \"args\": [ { \"property\": \"windSpeed\" } ] }\n",
    "{\n  \"op\": \"not\",\n  \"args\": [\n    {\n      \"op\": \"in\",\n      \"args\": [\n        { \"property\": \"value\" },\n        [ \"a\", \"b\", \"c\" ]\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"t_contains\",\n  \"args\": [\n    { \"interval\": [ \"2000-01-01T00:00:00Z\", \"2005-01-10T01:01:01.393216Z\" ] },\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] }\n    \n  ]\n}\n",
    "{\n  \"op\": \"like\",\n  \"args\": [\n    { \"property\": \"name\" },\n    \"Smith%\"\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"swimming_pool\" },\n        true\n      ]\n    },\n    {\n      \"op\": \"or\",\n      \"args\": [\n        {\n          \"op\": \">\",\n          \"args\": [\n            { \"property\": \"floors\" },\n            5\n          ]\n        },\n        {\n          \"op\": \"like\",\n          \"args\": [\n            { \"property\": \"material\" },\n            \"brick%\"\n          ]\n        },\n        {\n          \"op\": \"like\",\n          \"args\": [\n            { \"property\": \"material\" },\n            \"%brick\"\n          ]\n        }\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"like\",\n  \"args\": [\n    { \"property\": \"owner\" },\n    \"Mike%\"\n  ]\n}\n",
    "{\n  \"op\": \"t_disjoint\",\n  \"args\": [\n    { \"interval\": [ \"..\", \"2005-01-10T01:01:01.393216Z\" ] },\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] }\n  ]\n}\n",
    "{\n  \"op\": \"<=\",\n  \"args\": [\n    { \"property\": \"taxes\" },\n    500\n  ]\n}\n",
    "{\n  \"op\": \"t_meets\",\n  \"args\": [\n    { \"interval\": [ \"2005-01-10\", \"2010-02-10\" ] },\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] }\n  ]\n}\n",
    "{\n  \"op\": \"t_metBy\",\n  \"args\": [\n    { \"interval\": [ \"2010-02-10T05:29:20.073225Z\", \"2010-10-07\" ] },\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] }\n  ]\n}\n",
    "{\n  \"op\": \"not\",\n  \"args\": [\n    {\n      \"op\": \"isNull\",\n      \"args\": [ { \"property\": \"value\" } ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"<\",\n  \"args\": [\n    {\n      \"op\": \"avg\",\n      \"args\": [ { \"property\": \"windSpeed\" } ]\n    },\n    4\n  ]\n}\n",
    "{\n  \"op\": \"s_crosses\",\n  \"args\": [\n    { \"property\": \"road\" },\n    {\n      \"type\": \"Polygon\",\n      \"coordinates\": [\n        [\n          [ 43.7286, -79.2986 ], [ 43.7311, -79.2996 ], [ 43.7323, -79.2972 ],\n          [ 43.7326, -79.2971 ], [ 43.7350, -79.2981 ], [ 43.7350, -79.2982 ],\n          [ 43.7352, -79.2982 ], [ 43.7357, -79.2956 ], [ 43.7337, -79.2948 ],\n          [ 43.7343, -79.2933 ], [ 43.7339, -79.2923 ], [ 43.7327, -79.2947 ],\n          [ 43.7320, -79.2942 ], [ 43.7322, -79.2937 ], [ 43.7306, -79.2930 ],\n          [ 43.7303, -79.2930 ], [ 43.7299, -79.2928 ], [ 43.7286, -79.2986 ]\n        ]\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"between\",\n  \"args\": [\n    { \"property\": \"depth\" },\n    100.0,\n    150.0\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    1,\n    {\n      \"op\": \"div\",\n      \"args\": [ { \"property\": \"foo\" }, 2 ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"property\": \"value\" },\n    {\n      \"op\": \"/\",\n      \"args\": [\n        2,\n        { \"property\": \"foo\" }\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"not\",\n  \"args\": [\n    {\n      \"op\": \"like\",\n      \"args\": [\n        { \"property\": \"owner\" },\n        \"%Mike%\"\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"a_contains\",\n  \"args\": [\n    { \"property\": \"values\" },\n    [ \"a\", \"b\", \"c\" ]\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \"or\",\n      \"args\": [\n        {\n          \"op\": \"like\",\n          \"args\": [\n            { \"property\": \"owner\" },\n            \"mike%\"\n          ]\n        },\n        {\n          \"op\": \"like\",\n          \"args\": [\n            { \"property\": \"owner\" },\n            \"Mike%\"\n          ]\n        }\n      ]\n    },\n    {\n      \"op\": \"<\",\n      \"args\": [\n        { \"property\": \"floors\" },\n        4\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \">\",\n      \"args\": [\n        { \"property\": \"floors\" },\n        5\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"swimming_pool\" },\n        true\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"t_intersects\",\n  \"args\": [\n    { \"property\": \"event_time\" },\n    { \"interval\": [ \"1969-07-16T05:32:00Z\", \"1969-07-24T16:50:35Z\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"like\",\n  \"args\": [\n    { \"property\": \"owner\" },\n    \"%Jones%\"\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \"between\",\n      \"args\": [\n        { \"property\": \"eo:cloud_cover\" },\n        0.1, 0.2\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"landsat:wrs_row\" },\n        28\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"landsat:wrs_path\" },\n        203\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"t_during\",\n  \"args\": [\n    { \"interval\": [ \"1969-07-20T20:17:40Z\", \"1969-07-21T17:54:00Z\" ] },\n    { \"interval\": [ \"1969-07-16T13:32:00Z\", \"1969-07-24T16:50:35Z\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"t_overlaps\",\n  \"args\": [\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] },\n    { \"interval\": [ \"1991-10-07T08:21:06.393262Z\", \"1992-10-09T08:08:08.393473Z\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"a_containedBy\",\n  \"args\": [\n    { \"property\": \"values\" },\n    [ \"a\", \"b\", \"c\" ]\n  ]\n}\n",
    "{\n  \"op\": \"not\",\n  \"args\": [\n    {\n      \"op\": \"isNull\",\n      \"args\": [ { \"property\": \"geometry\" } ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \">=\",\n  \"args\": [\n    { \"property\": \"updated\" },\n    { \"date\": \"1970-01-01\" }\n  ]\n}\n",
    "{\n  \"op\": \"t_before\",\n  \"args\": [\n    { \"property\": \"updated_at\" },\n    { \"timestamp\": \"2012-08-10T05:30:00Z\" }\n  ]\n}\n",
    "{\n  \"op\": \"s_equals\",\n  \"args\": [\n    {\n      \"type\": \"MultiPoint\",\n      \"coordinates\": [ [ 180.0, -0.5 ],\n                       [ 179.0, -47.121701 ],\n                       [ 180.0, -0.0 ],\n                       [ 33.470475, -0.99999 ],\n                       [ 179.0, -15.333062 ] ]\n    },\n    { \"property\": \"geometry\" }\n  ]\n}\n",
    "{\n  \"op\": \">\",\n  \"args\": [\n    { \"property\": \"floors\" },\n    5\n  ]\n}\n",
    "{\n  \"op\": \"s_disjoint\",\n  \"args\": [\n    { \"property\": \"geometry\" },\n    {\n      \"type\": \"MultiPolygon\",\n      \"coordinates\": [ [ [ [ 144.022387, 45.176126 ],\n                           [ -1.1, 0.0 ],\n                           [ 180.0, 47.808086 ],\n                           [ 144.022387, 45.176126 ] ] ] ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"<>\",\n  \"args\": [\n    false,\n    {\n      \"op\": \"Bar\",\n      \"args\": [ { \"property\": \"geometry\" }, 100, \"a\", \"b\", false ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"isNull\",\n  \"args\": [ { \"property\": \"value\" } ]\n}\n",
    "{\n  \"op\": \"in\",\n  \"args\": [\n    { \"property\": \"landsat:wrs_path\" },\n    [ \"153\", \"154\", \"15X\" ]\n  ]\n}\n",
    "{\n  \"op\": \"like\",\n  \"args\": [\n    { \"property\": \"name\" },\n    { \"op\": \"casei\", \"args\": [ \"FOO%\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \"not\",\n      \"args\": [\n        {\n          \"op\": \"like\",\n          \"args\": [\n            { \"property\": \"name\" },\n            \"foo%\"\n          ]\n        }\n      ]\n    },\n    {\n      \"op\": \">\",\n      \"args\": [\n        { \"property\": \"value\" },\n        10\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"s_intersects\",\n  \"args\": [\n    { \"property\": \"geometry\" },\n    {\n      \"type\": \"Point\",\n      \"coordinates\": [ 36.319836, 32.288087 ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"or\",\n  \"args\": [\n    {\n      \"op\": \"and\",\n      \"args\": [\n        {\n          \"op\": \">\",\n          \"args\": [\n            { \"property\": \"floors\" },\n            5\n          ]\n        },\n        {\n          \"op\": \"=\",\n          \"args\": [\n            { \"property\": \"material\" },\n            \"brick\"\n          ]\n        }\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"swimming_pool\" },\n        true\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"s_overlaps\",\n  \"args\": [\n    { \"property\": \"geometry\" },\n    { \"bbox\": [ -179.912109, 1.9, 180.0, 16.897016 ] }\n  ]\n}\n",
    "{\n  \"op\": \"or\",\n  \"args\": [\n    {\n      \"op\": \"isNull\",\n      \"args\": [ { \"property\": \"value\" } ]\n    },\n    {\n      \"op\": \"between\",\n      \"args\": [\n        { \"property\": \"value\" },\n        10, 20\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"not\",\n  \"args\": [\n    {\n      \"op\": \"between\",\n      \"args\": [\n        { \"property\": \"value\" },\n        10, 20\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"property\": \"landsat:scene_id\" },\n    \"LC82030282019133LGN00\"\n  ]\n}\n",
    "{\n  \"op\": \"a_overlaps\",\n  \"args\": [\n    { \"property\": \"values\" },\n    [ { \"timestamp\": \"2012-08-10T05:30:00Z\" }, { \"date\": \"2010-02-10\" }, false ]\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"property\": \"city\" },\n    \"Toronto\"\n  ]\n}\n",
    "{\n  \"op\": \"in\",\n  \"args\": [\n    { \"property\": \"cityName\" },\n    [ \"Toronto\", \"Frankfurt\", \"Tokyo\", \"New York\" ]\n  ]\n}\n",
    "{\n  \"op\": \"in\",\n  \"args\": [\n    { \"property\": \"value\" },\n    [ 1.0, 2.0, 3.0 ]\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"op\": \"accenti\", \"args\": [ { \"property\": \"owner\" } ] },\n    { \"op\": \"accenti\", \"args\": [ \"Beyonce\u{301}\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"<>\",\n  \"args\": [\n    { \"property\": \"id\" },\n    \"fa7e1920-9107-422d-a3db-c468cbc5d6df\"\n  ]\n}\n",
    "{\n  \"op\": \"t_during\",\n  \"args\": [\n    { \"interval\": [ { \"property\": \"touchdown\" }, { \"property\": \"liftOff\" } ] },\n    { \"interval\": [ \"1969-07-16T13:32:00Z\", \"1969-07-24T16:50:35Z\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"or\",\n  \"args\": [\n    {\n      \"op\": \"not\",\n      \"args\": [\n        {\n          \"op\": \"<\",\n          \"args\": [\n            { \"property\": \"floors\" },\n            5\n          ]\n        }\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"swimming_pool\" },\n        true\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"like\",\n  \"args\": [\n    { \"property\": \"eo:instrument\" },\n    \"OLI%\"\n  ]\n}\n",
    "{\n  \"op\": \">=\",\n  \"args\": [\n    { \"property\": \"value\" },\n    10\n  ]\n}\n",
    "{\n  \"op\": \"t_during\",\n  \"args\": [\n    {\"interval\": [{ \"property\": \"starts_at\" }, { \"property\": \"ends_at\" }]},\n    {\"interval\": [\"2005-01-10\", \"2010-02-10\"]\n    }\n  ]\n}\n",
    "{\n  \"op\": \">\",\n  \"args\": [\n    { \"property\": \"value\" },\n    {\n      \"op\": \"+\",\n      \"args\": [\n        { \"property\": \"foo\" },\n        10\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"not\",\n  \"args\": [\n    {\n      \"op\": \"like\",\n      \"args\": [\n        { \"property\": \"name\" },\n        \"foo%\"\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"t_after\",\n  \"args\": [\n    { \"property\": \"updated_at\" },\n    { \"date\": \"2010-02-10\" }\n  ]\n}\n",
    "{\n  \"op\": \">\",\n  \"args\": [\n    { \"property\": \"vehicle_height\" },\n    {\n      \"op\": \"-\",\n      \"args\": [\n        { \"property\": \"bridge_clearance\" },\n        1\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"s_contains\",\n  \"args\": [\n    { \"property\": \"geometry\" },\n    {\n      \"type\": \"Point\",\n      \"coordinates\": [ -3.508362, -1.754181 ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"t_after\",\n  \"args\": [\n    { \"property\": \"built\" },\n    { \"date\": \"2012-06-05\" }\n  ]\n}\n",
    "{\n  \"op\": \"s_equals\",\n  \"args\": [\n    {\n      \"type\": \"Polygon\",\n      \"coordinates\": [ [ [ -0.333333, 89.0 ],\n                         [ -102.723546, -0.5 ],\n                         [ -179.0, -89.0 ],\n                         [ -1.9, 89.0 ],\n                         [ -0.0, 89.0 ],\n                         [ 2.00001, -1.9 ],\n                         [ -0.333333, 89.0 ] ] ]\n    },\n    { \"property\": \"geometry\" }\n  ]\n}\n",
    "{\n  \"op\": \"or\",\n  \"args\": [\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"eo:cloud_cover\" },\n        0.1\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"eo:cloud_cover\" },\n        0.2\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"<=\",\n  \"args\": [\n    { \"property\": \"value\" },\n    10\n  ]\n}\n",
    "{\n  \"op\": \"t_before\",\n  \"args\": [\n    { \"property\": \"built\" },\n    { \"date\": \"2015-01-01\" }\n  ]\n}\n",
    "{\n  \"op\": \"s_intersects\",\n  \"args\": [\n    { \"property\": \"geometry\" },\n    {\n      \"type\": \"Polygon\",\n      \"coordinates\": [ [ [ -10, -10 ], [ 10, -10 ], [ 10, 10 ], [ -10, -10 ] ] ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    {\n      \"op\": \"accenti\",\n      \"args\": [ { \"property\": \"etat_vol\" } ]\n    },\n    {\n      \"op\": \"accenti\",\n      \"args\": [ \"débárquér\" ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \">=\",\n      \"args\": [\n        { \"property\": \"eo:cloud_cover\" },\n        0.1\n      ]\n    },\n    {\n      \"op\": \"<=\",\n      \"args\": [\n        { \"property\": \"eo:cloud_cover\" },\n        0.2\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"landsat:wrs_row\" },\n        28\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"landsat:wrs_path\" },\n        203\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"like\",\n  \"args\": [\n    { \"op\": \"casei\", \"args\": [ { \"property\": \"geophys:SURVEY_NAME\" } ] },\n    { \"op\": \"casei\", \"args\": [ \"%calcutta%\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"a_equals\",\n  \"args\": [\n    [ \"a\", true, 1.0, 8 ],\n    { \"property\": \"values\" }\n  ]\n}\n",
    "{\n  \"op\": \"t_finishedBy\",\n  \"args\": [\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] },\n    { \"interval\": [ \"1991-10-07T08:21:06.393262Z\", \"2010-02-10T05:29:20.073225Z\" ] }\n  ]\n}\n",
    "{\n  \"op\": \">\",\n  \"args\": [\n    { \"property\": \"value\" },\n    10\n  ]\n}\n",
    "{\n  \"op\": \"s_intersects\",\n  \"args\": [\n    { \"property\": \"geometry\" },\n    { \"bbox\": [ -128.098193, -1.1, -99999.0, 180.0, 90.0, 100000.0 ] }\n  ]\n}\n",
    "{\n  \"op\": \"<\",\n  \"args\": [\n    { \"property\": \"value\" },\n    10\n  ]\n}\n",
    "{\n  \"op\": \"t_startedBy\",\n  \"args\": [\n    { \"interval\": [ \"1991-10-07T08:21:06.393262Z\", \"2010-02-10T05:29:20.073225Z\" ] },\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] }\n  ]\n}\n",
    "{\n  \"op\": \"s_within\",\n  \"args\": [\n    { \"property\": \"location\" },\n    { \"bbox\": [ -118, 33.8, -117.9, 34 ] }\n  ]\n}\n",
    "{\n  \"op\": \"<\",\n  \"args\": [\n    { \"property\": \"value\" },\n    {\n      \"op\": \"-\",\n      \"args\": [\n        { \"property\": \"foo\" },\n        10\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"s_equals\",\n  \"args\": [\n    {\n      \"type\": \"GeometryCollection\",\n      \"geometries\": [\n        {\n          \"type\": \"Point\",\n          \"coordinates\": [ 1.9, 2.00001 ]\n        },\n        {\n          \"type\": \"Point\",\n          \"coordinates\": [ 0.0, -2.00001 ]\n        },\n        {\n          \"type\": \"MultiLineString\",\n          \"coordinates\": [ [ [ -2.00001, -0.0 ],\n                             [ -77.292642, -0.5 ],\n                             [ -87.515626, -0.0 ],\n                             [ -180.0, 12.502773 ],\n                             [ 21.204842, -1.5 ],\n                             [ -21.878857, -90.0 ] ] ]\n        },\n        {\n          \"type\": \"Point\",\n          \"coordinates\": [ 1.9, 0.5 ]\n        },\n        {\n          \"type\": \"LineString\",\n          \"coordinates\": [ [ 179.0, 1.179148 ],\n                           [ -148.192487, -65.007816 ],\n                           [ 0.5, 0.333333 ] ]\n        }\n      ]\n    },\n    { \"property\": \"geometry\" }\n  ]\n}\n",
    "{\n  \"op\": \"t_intersects\",\n  \"args\": [\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] },\n    { \"interval\": [ \"1991-10-07T08:21:06.393262Z\", \"2010-02-10T05:29:20.073225Z\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"t_equals\",\n  \"args\": [\n    { \"property\": \"updated_at\" },\n    { \"date\": \"1851-04-29\" }\n  ]\n}\n",
    "{\n  \"op\": \"between\",\n  \"args\": [\n    { \"property\": \"value\" },\n    10, 20\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"op\": \"casei\", \"args\": [ { \"property\": \"owner\" } ] },\n    { \"op\": \"casei\", \"args\": [ \"somebody else\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"s_crosses\",\n  \"args\": [\n    { \"property\": \"geometry\" },\n    {\n      \"type\": \"LineString\",\n      \"coordinates\": [ [ 172.03086, 1.5 ],\n                       [ 1.1, -90.0 ],\n                       [ -159.757695, 0.99999 ],\n                       [ -180.0, 0.5 ],\n                       [ -12.111235, 81.336403 ],\n                       [ -0.5, 64.43958 ],\n                       [ 0.0, 81.991815 ],\n                       [ -155.93831, 90.0 ] ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"op\": \"accenti\", \"args\": [ { \"property\": \"etat_vol\" } ] },\n    { \"op\": \"accenti\", \"args\": [ \"débárquér\" ] }\n  ]\n}\n",
    "{\n  \"op\": \">\",\n  \"args\": [\n    {\n      \"op\": \"-\",\n      \"args\": [\n        { \"property\": \"balance\" },\n        150.0\n      ]\n    },\n    0\n  ]\n}\n",
    "{\n  \"op\": \"a_contains\",\n  \"args\": [\n    { \"property\": \"layer:ids\" },\n    [ \"layers-ca\", \"layers-us\" ]\n  ]\n}\n",
    "{\n  \"op\": \"in\",\n  \"args\": [\n    { \"property\": \"eo:cloud_cover\" },\n    [ 0.1, 0.2 ]\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \"like\",\n      \"args\": [\n        { \"property\": \"eo:instrument\" },\n        \"OLI%\"\n      ]\n    },\n    {\n      \"op\": \"s_intersects\",\n      \"args\": [\n        { \"property\": \"footprint\" },\n        {\n          \"type\": \"Polygon\",\n          \"coordinates\": [\n            [ [ 43.5845, -79.5442 ],\n              [ 43.6079, -79.4893 ],\n              [ 43.5677, -79.4632 ],\n              [ 43.6129, -79.3925 ],\n              [ 43.6223, -79.3238 ],\n              [ 43.6576, -79.3163 ],\n              [ 43.7945, -79.1178 ],\n              [ 43.8144, -79.1542 ],\n              [ 43.8555, -79.1714 ],\n              [ 43.7509, -79.639  ],\n              [ 43.5845, -79.5442 ]\n            ]\n          ]\n        }\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"<>\",\n  \"args\": [\n    { \"property\": \"value\" },\n    {\n      \"op\": \"*\",\n      \"args\": [\n        22.1,\n        { \"property\": \"foo\" }\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"s_touches\",\n  \"args\": [\n    { \"property\": \"geometry\" },\n    {\n      \"type\": \"MultiLineString\",\n      \"coordinates\": [ [ [ -1.9, -0.99999 ],\n                         [ 75.292574, 1.5 ],\n                         [ -0.5, -4.016458 ],\n                         [ -31.708594, -74.743801 ],\n                         [ 179.0, -90.0 ] ],\n                       [ [ -1.9, -1.1 ],\n                         [ 1.5, 8.547371 ] ] ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    0,\n    {\n      \"op\": \"%\",\n      \"args\": [ { \"property\": \"foo\" }, 2 ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"like\",\n  \"args\": [\n    { \"property\": \"name\" },\n    \"foo%\"\n  ]\n}\n",
    "{\n  \"op\": \"s_within\",\n  \"args\": [\n    {\n      \"type\": \"Polygon\",\n      \"coordinates\": [ [ [ -49.88024, 0.5, -75993.341684 ],\n                         [ -1.5, -0.99999, -100000.0 ],\n                         [ 0.0, 0.5, -0.333333 ],\n                         [ -49.88024, 0.5, -75993.341684 ] ],\n                       [ [ -65.887123, 2.00001, -100000.0 ],\n                         [ 0.333333, -53.017711, -79471.332949 ],\n                         [ 180.0, 0.0, 1852.616704 ],\n                         [ -65.887123, 2.00001, -100000.0 ] ] ]\n    },\n    { \"property\": \"geometry\" }\n  ]\n}\n",
    "{\n  \"op\": \"t_during\",\n  \"args\": [\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] },\n    { \"interval\": [ \"2017-06-10T07:30:00Z\", \"2017-06-11T10:30:00Z\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"property\": \"swimming_pool\" },\n    true\n  ]\n}\n",
    "{\n  \"op\": \"s_within\",\n  \"args\": [\n    { \"property\": \"road\" },\n    {\n      \"op\": \"Buffer\",\n      \"args\": [\n        { \"property\": \"geometry\" },\n        10,\n        \"m\"\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"t_starts\",\n  \"args\": [\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] },\n    { \"interval\": [ \"1991-10-07T08:21:06.393262Z\", \"..\" ] }\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \"<\",\n      \"args\": [\n        { \"property\": \"eo:cloud_cover\" },\n        0.1\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"landsat:wrs_row\" },\n        28\n      ]\n    },\n    {\n      \"op\": \"=\",\n      \"args\": [\n        { \"property\": \"landsat:wrs_path\" },\n        203\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"in\",\n  \"args\": [\n    {\n      \"op\": \"casei\",\n      \"args\": [ { \"property\": \"road_class\" } ]\n    },\n    [\n      { \"op\": \"casei\", \"args\": [ \"Οδος\" ] },\n      { \"op\": \"casei\", \"args\": [ \"Straße\" ] }\n    ]\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    {\n      \"op\": \"Foo\",\n      \"args\": [ { \"property\": \"geometry\" } ]\n    },\n    true\n  ]\n}\n",
    "{\n  \"op\": \"t_overlappedBy\",\n  \"args\": [\n    { \"interval\": [ \"1991-10-07T08:21:06.393262Z\", \"2010-02-10T05:29:20.073225Z\" ] },\n    { \"interval\": [ { \"property\": \"starts_at\" }, { \"property\": \"ends_at\" } ] }\n  ]\n}\n",
    "{\n  \"op\": \"and\",\n  \"args\": [\n    {\n      \"op\": \">\",\n      \"args\": [\n        { \"property\": \"floors\" },\n        5\n      ]\n    },\n    {\n      \"op\": \"s_within\",\n      \"args\": [\n        { \"property\": \"geometry\" },\n        { \"bbox\": [ -118, 33.8, -117.9, 34 ] }\n      ]\n    }\n  ]\n}\n",
    "{\n  \"op\": \"in\",\n  \"args\": [\n    { \"op\": \"casei\", \"args\": [ { \"property\": \"road_class\" } ] },\n    [\n      { \"op\": \"casei\", \"args\": [ \"Οδος\" ] },\n      { \"op\": \"casei\", \"args\": [ \"Straße\" ] }\n    ]\n  ]\n}\n",
    "{\n  \"op\": \"=\",\n  \"args\": [\n    { \"property\": \"id\" },\n    \"fa7e1920-9107-422d-a3db-c468cbc5d6df\"\n  ]\n}\n",
    "{\n  \"op\": \"not\",\n  \"args\": [\n    {\n      \"op\": \"in\",\n      \"args\": [\n        { \"property\": \"category\" },\n        [ 1, 2, 3, 4 ]\n      ]\n    }\n  ]\n}\n",
];

pub(crate) fn async_runtime() -> Runtime {
    Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
}

// ----- CSV stuff ------------------------------------------------------------

#[rustfmt::skip]
#[derive(Debug, Default, Deserialize)]
pub(crate) struct CSVFeature {
    /*  0 */ fid: i32,
    /*  1 */ geom: String,
    #[serde(skip)] type_: String,
    #[serde(skip)] adm0_a3: String,
    /*  4 */ #[serde(rename(deserialize = "NAME"))] name: String,
    #[serde(skip)] name_long: String,
    #[serde(skip)] abbrev: String,
    #[serde(skip)] postal: String,
    #[serde(skip)] formal_en: String,
    #[serde(skip)] name_sort: String,
    /* 10 */ #[serde(rename(deserialize = "POP_EST"))] pop_est: f64,
    #[serde(skip)] enonomy: String,
    #[serde(skip)] income_grp: String,
    #[serde(skip)] continent: String,
    #[serde(skip)] region_un: String,
    #[serde(skip)] subregion: String,
    #[serde(skip)] region_wb: String,
    #[serde(skip)] wikidataid: String,
    #[serde(skip)] name_de: String,
    #[serde(skip)] name_en: String,
    #[serde(skip)] name_el: String,
}

impl TryFrom<CSVFeature> for Resource {
    type Error = MyError;

    fn try_from(value: CSVFeature) -> Result<Self, Self::Error> {
        Ok(HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkt(&value.geom)?),
            ("NAME".into(), Q::new_plain_str(&value.name)),
            ("POP_EST".into(), Q::from(value.pop_est)),
        ]))
    }
}

gen_csv_ds!(pub(crate), "Country", COUNTRIES_CSV, CSVFeature);

// ----- GeoPackage stuff -----------------------------------------------------

#[rustfmt::skip]
#[derive(Debug, FromRow)]
pub(crate) struct GPkgFeature {
    fid: i32,
    geom: Vec<u8>,
    #[sqlx(rename = "NAME")] name: String,
    #[sqlx(rename = "POP_EST")] pop_est: f64,
}

impl TryFrom<GPkgFeature> for Resource {
    type Error = MyError;

    fn try_from(value: GPkgFeature) -> Result<Self, Self::Error> {
        Ok(HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("geom".into(), Q::try_from_wkb(&value.geom)?),
            ("NAME".into(), Q::new_plain_str(&value.name)),
            ("POP_EST".into(), Q::from(value.pop_est)),
        ]))
    }
}

gen_gpkg_ds!(pub(crate), "Country", GPKG_URL, COUNTRIES_TBL, GPkgFeature);

// ----- PostGIS stuff --------------------------------------------------------

#[rustfmt::skip]
#[derive(Debug, FromRow)]
pub(crate) struct PGFeature {
    fid: i32,
    #[sqlx(rename = "NAME")] name: String,
    #[sqlx(rename = "POP_EST")] pop_est: f64,
    geom: G,
}

impl TryFrom<PGFeature> for Resource {
    type Error = MyError;

    fn try_from(value: PGFeature) -> Result<Self, Self::Error> {
        Ok(HashMap::from([
            ("fid".into(), Q::try_from(value.fid)?),
            ("NAME".into(), Q::new_plain_str(&value.name)),
            ("POP_EST".into(), Q::from(value.pop_est)),
            ("geom".into(), Q::Geom(value.geom)),
        ]))
    }
}

gen_pg_ds!(pub(crate), "Country", PG_DB_NAME, COUNTRIES_TBL, PGFeature);
