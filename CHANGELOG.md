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
