# Functions

The CQL2 Standard makes provisions for externally defined _Functions_ in addition to few specific ones defined in the specs.

This project implements **all** the required standard functions. It also offers:

* Support for few ones called _builtins_ that can be used in _Filter Expressions_.
* A mechanism for externally defined functions, implemented as [Rust Closures](https://doc.rust-lang.org/book/ch13-01-closures.html), that can be _registered_ in a [Context] which is then passed to an [Evaluator] implementation (such as [EvaluatorImpl]) for processing an [Expression] against one or more [Resource].

Examples of both types, and the plumbing to wire them, abound in the `tests` folder. Here is a working simple example:

```rust
use ogc_cql2::prelude::*;
# use std::error::Error;
# fn test() -> Result<(), Box<dyn Error>> {

// define a function that adds 2 numbers together...
let sum = |x: f64, y: f64| x + y;

// create a Context and register that function and its metadata...
let mut ctx = Context::new();
ctx.register(
    "sum",
    vec![ExtDataType::Num, ExtDataType::Num],
    ExtDataType::Num,
    move |args| {
        let a1 = args.first()?.downcast_ref::<f64>()?;
        let a2 = args.get(1)?.downcast_ref::<f64>()?;
        Some(Box::new(sum(*a1, *a2)))
    },
 );

// freeze the Context (make it read-only) so we can share it safely... 
let shared_ctx = ctx.freeze();

// parse an Expression from a text string...
let expression = Expression::try_from_text("3 = sum(1, 2)")?;

// instantiate an Evaluator instance and feed it the Context...
let mut evaluator = EvaluatorImpl::new(shared_ctx);

// now set up that Evaluator for evaluating Resources...
evaluator.setup(expression)?;

// since our Expression does not need any queryable Resource property,
// use an empty one...
let feature = Resource::new();

// evaluate the Expression...
let res = evaluator.evaluate(&feature)?;

// assert the outcome is TRUE...
assert!(matches!(res, Outcome::T));

// tear down the Evaluator...
evaluator.teardown()?;

# Ok(())
# }
```

# Data types

This library supports a subset of data types available in a Rust environment for use with function arguments and results. The [ExtDataType] variants embody those types. Each variant maps to a specific yet opaque (for now) Rust type...

| `ExtDataType` variant | Symbol | inner type                                                       |
|-----------------------|--------|------------------------------------------------------------------|
| `Num`                 |  `N`   | f64                                                              |
| `Str`                 |  `S`   | [QString] (**only the plain variant**)                           |
| `Bool`                |  `B`   | bool                                                             |
| `Timestamp`           |  `Z`   | [jiff::Zoned](https://docs.rs/jiff/0.2.15/jiff/struct.Zoned.html)|
| `Date`                |  `Z`   | jiff::Zoned                                                      |
| `Geom`                |  `G`   | [geos::Geom](https://docs.rs/geos/10.0.0/geos/trait.Geom.html)   |


# Numeric (`Num`) builtins

| Name   | Argument(s)   | Result | Description                                           | See      |
|--------|---------------|--------|-------------------------------------------------------|----------|
| `abs`  | x: `N`        | `N`    | Compute absolute value of `x`.                        |[See][101]|
| `acos` | x: `N`        | `N`    | Compute arccosine of `x`. Result is in radians.       |[See][102]|
| `asin` | x: `N`        | `N`    | Compute arcsine of `x`. Result is in radians.         |[See][103]|
| `atan` | x: `N`        | `N`    | Compute arctangent of `x`. Result is in radians.      |[See][104]|
| `cbrt` | x: `N`        | `N`    | Compute cube root of `x`.                             |[See][105]|
| `ceil` | x: `N`        | `N`    | Compute smallest integer greater than or equal to `x`.|[See][106]|
| `cos`  | x: `N`        | `N`    | Compute cosine of `x` (in radians).                   |[See][107]|
| `floor`| x: `N`        | `N`    | Compute largest integer less than or equal to `x`.    |[See][108]|
| `ln`   | x: `N`        | `N`    | Compute natural logarithm of `x`.                     |[See][109]|
| `sin`  | x: `N`        | `N`    | Compute sine of `x` (in radians).                     |[See][110]|
| `sqrt` | x: `N`        | `N`    | Compute square root of `x`.                           |[See][111]|
| `tan`  | x: `N`        | `N`    | Compute tangent of `x` (in radians).                  |[See][112]|
| `max`  | x: `N`, y: `N`| `N`    | Compute maximum of `x` and `y`.                       |[See][113]|
| `avg`  | x: `N`, y: `N`| `N`    | Compute midpoint (average) between `x` and `y`.       |[See][114]|
| `min`  | x: `N`, y: `N`| `N`    | Compute minimum of `x` and `y`.                       |[See][115]|


# String (`Str`) builtins

| Name         | Argument(s)   | Result | Description                                         | See      |
|--------------|---------------|--------|-----------------------------------------------------|----------|
| `trim`       | x: `S`        | `S`    | Remove leading and trailing whitespaces from `x`.   |[See][201]|
| `len`        | x: `S`        | `N`    | Compute length of `x` in **bytes**.                 |[See][202]|
| `concat`     | x: `S`, y: `S`| `S`    | Append `y` to the end of `x`.                       |[See][203]|
| `starts_with`| x: `S`, y: `S`| `B`    | Return TRUE if `y` is a prefix of `x`. FALSE otherwise.|[See][204]|
| `ends_with`  | x: `S`, y: `S`| `B`    | Return TRUE if `y` is a suffix of `x`. FALSE otherwise.|[See][205]|


# Temporal builtins

| Name   | Argument(s) | Result | Description                                           |
|--------|-------------|--------|-------------------------------------------------------|
| `now`  |             | `Z`    | Return the current timestamp in UTC time-zone.        |
| `today`|             | `Z`    | Return today's date in UTC time-zone.                 |


# Geometry (`Geom`) builtins

| Name       | Argument(s)   | Result | Description                                                |
|------------|---------------|--------|------------------------------------------------------------|
| `boundary` | x: `G`        | `G`    | Return the closure of combinatorial boundary of `x`.       |
| `buffer`   | x: `G`, y: `N`| `G`    | Return a geometry representing all points whose distance from `x` is less than or equal to `y`.|
| `envelope` | x: `G`        | `G`    | Return the minimum bouding box of `x`.                     |
| `centroid` | x: `G`        | `G`    | Return the geometric centre of `x`.                        |
|`convex_hull`| x: `G`       | `G`    | Return minimum convex geometry that encloses all geometries within `x`.|
| `get_x`    | x: `G`        | `N`    | Return the _X_ coordinate of `x` if it's a Point.          |
| `get_y`    | x: `G`        | `N`    | Return the _Y_ coordinate of `x` if it's a Point.          |
| `get_z`    | x: `G`        | `N`    | Return the _Z_ coordinate of `x` if it's a Point and is 3D.|
| `wkt`      | x: `G`        | `S`    | Return a WKT representation of `x`.                        |


[101]: <https://doc.rust-lang.org/std/primitive.f64.html#method.abs>
[102]: <https://doc.rust-lang.org/std/primitive.f64.html#method.acos>
[103]: <https://doc.rust-lang.org/std/primitive.f64.html#method.asin>
[104]: <https://doc.rust-lang.org/std/primitive.f64.html#method.atan>
[105]: <https://doc.rust-lang.org/std/primitive.f64.html#method.cbrt>
[106]: <https://doc.rust-lang.org/std/primitive.f64.html#method.ceil>
[107]: <https://doc.rust-lang.org/std/primitive.f64.html#method.cos>
[108]: <https://doc.rust-lang.org/std/primitive.f64.html#method.floor>
[109]: <https://doc.rust-lang.org/std/primitive.f64.html#method.ln>
[110]: <https://doc.rust-lang.org/std/primitive.f64.html#method.sin>
[111]: <https://doc.rust-lang.org/std/primitive.f64.html#method.sqrt>
[112]: <https://doc.rust-lang.org/std/primitive.f64.html#method.tan>
[113]: <https://doc.rust-lang.org/std/primitive.f64.html#method.max>
[114]: <https://doc.rust-lang.org/std/primitive.f64.html#method.midpoint>
[115]: <https://doc.rust-lang.org/std/primitive.f64.html#method.min>

[201]: <https://doc.rust-lang.org/std/primitive.str.html#method.trim>
[202]: <https://doc.rust-lang.org/std/string/struct.String.html#method.len>
[203]: <https://doc.rust-lang.org/std/string/struct.String.html#method.push_str>
[204]: <https://doc.rust-lang.org/std/primitive.str.html#method.starts_with>
[205]: <https://doc.rust-lang.org/std/primitive.str.html#method.ends_with>
