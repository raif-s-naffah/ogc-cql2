# UNPUBLISHED (2025-12-18)

* DataSource trait now exposes an srid() method to return the SRID of the layer
  if known/discovered at construction.
* Clarify context (GeoPackage or PostGIS) WKB mentions refer to.
* Fully qualify referenced crates in macros.
* Fixed README spelling mistakes.
* Use latest secondary dependent crates.

# Version 0.4.0 (2025-12-10)

* Fixed bug when generating a MULTIPOLYGON from a BBOX.
* Fixed bug affecting GeoPackage SRID detection.
* Added Criterion benchmarks.
* Modified GeoPackage implementation to minimize differences of
  generated SQL statements w/ that of PostGIS.
* Added environment variables to configure PostgreSQL database URL + connection
  parameters.
* Implemented StreamableDS for PostGIS tables.
* Upgrade `tracing` to 0.1.43.
* Upgrade `serde_with` to 3.16.1.
* Use SQL syntax common to both SQLite and PostgreSQL when possible.
* Hid internal macros, edited documentation + fixed spelling mistakes.

# Version 0.3.0 (2025-11-20)

* Tripled most of the normative tests to exercise the new data sources and traits.
* Expressions are now "reduced" before being evaluated, or converted into SQL.
* Added 'Iterable' and 'Streamable' traits.
* Added 'DataSource' trait w/ 'CSV' and 'GeoPackage' implementations.
* Added the _GeoPackage_ test data DB/file included in the specs and used it in
  `test_xxx_gpkg` and `test_xxx_sql` unit tests.
* Renamed some types.
* Upgrade `serde_with` to 3.16.0.
* Upgrade `jiff` to 0.2.16.
* Upgrade `regress` to 0.10.5.
* Upgrade `unicode-normalization` to 0.1.25.
* Upgrade `thiserror` to 2.0.17.
* Upgrade `serde` to 1.0.228.
* Upgrade `serde_json` to 1.0.145.
* Fixed some documentation spelling.
* Use latest secondary dependent crates.
* Formatting.

# Version 0.2.0 (2025-09-03)

* Added ability to control precision when generating WKT.
* Made the `geom::GTrait` public.
* Reduced visibility of `config::Config` to crate-private.
* Removed convenience method (`context::Context::new_shared()`) to make it explicit, when constructing new instances, what CRS to use when processing geometries w/ a specific but not set CRS. Amended the calls (in `tests`) accordingly.
* Updated README + documentation. Added blurb about configuring the library.
* Fixed some documentation errors.
* Use PEG rules to parse geometry WKT.
* Upgrade `proj` to 0.31.0.
* Use latest secondary dependent crates.

# Version 0.1.1 (2025-08-23)

* Unhide the fact that Q::Str is a QString.
* Fix link to license file.

# Version 0.1.0 (2025-08-23)

* Initial push to GitHub.
