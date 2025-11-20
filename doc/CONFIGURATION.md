# Configuring this library

This library, so far, relies on 3 environment variables `DEFAULT_CRS`, `DEFAULT_PRECISION`, and `RUST_LOG`.

The file `.env.template` contains those variables w/ their defaults. To adapt it to your environment make a copy, rename it `.env` and change the values as required.


## `DEFAULT_CRS`
This environment variable defines the implicit _Coordinate Reference System_ (CRS) code to use when checking if coordinates fall w/in a geometry's CRS validity extent (a.k.a Area Of Use). It defaults to `EPSG:4326` if undefined.

The [standard](https://docs.ogc.org/is/21-065r2/21-065r2.html#basic-spatial-data-types) mentions this in...

> _Since WKT and GeoJSON do not provide a capability to specify the CRS of a geometry literal, the server has to determine the CRS of the geometry literals in a filter expression through another mechanism._

This value is fed to a `Context` when created using the `new()` constructor and will trickle down and be used when parsing _Expressions_ containing geometry queryables and literals. For example...

```text
    let shared_ctx = Context::new().freeze();
```

Because the _Conformance Tests_ expect `EPSG:4326` to be indeed the implicit CRS when using included (in the standard) test data, this library allows overriding the global implicit CRS when constructing a `Context` before freezing and handing it over to _Evaluators_. Here's an example when used in most of the _Conformance Tests_...

```text
    let shared_ctx = Context::try_with_crs("EPSG:4326")?.freeze();
```


## `DEFAULT_PRECISION`
By _Precision_ I mean the number of digits after the decimal point.

This environment variable controls 3 things: (a) the _precision_ to keep when ingesting geometry coordinates, (b) the _precision_ to use when rendering geometry WKT output using the `to_wkt()` generic method, and (c) the _Precision_ to use when invoking certain spatial _ST_ functions such as `ST_Within` and others.

The default value of `7` ensures that coordinates in _WGS 84_ (which is the default implicit CRS) are compared w/ an accuracy of `1.11` cm.

For now only integers greater than or equal to `0` and less than or equal to `32` are allowed.

The `GTrait` made public since vesion 0.2.0 and implemented for all geometry variants allows for fine-tuning the WKT output by offering the following method...

```text
    fn to_wkt_fmt(&self, precision: usize) -> String;
```

## `RUST_LOG`
See <https://docs.rs/env_logger/latest/env_logger/#enabling-logging> for details.
