// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Expressions evaluation context.
//!

use crate::{Context, QString};
use core::fmt;
use geos::{Geom, Geometry};
use jiff::{Zoned, tz::TimeZone};
use std::any::Any;

/// Externally visible data type variants for arguments and result types used
/// and referenced by user-defined and registered functions invoked in filter
/// expressions.
#[derive(Debug)]
pub enum ExtDataType {
    /// A Unicode UTF-8 string.
    Str,
    /// A numeric value including integers and floating points.
    Num,
    /// A boolean value.
    Bool,
    /// An _Instant_ with a granularity of a second or smaller. Timestamps are
    /// always in the time zone UTC ("Z").
    Timestamp,
    /// An _Instant_ with a granularity of a day. Dates are local without an
    /// associated time zone.
    Date,
    /// A spatial (geometry) value.
    Geom,
}

/// Type alias for a generic Function that may be invoked in the process of
/// evaluating a CQL2 expression.
type GenericFn = Box<dyn Fn(Vec<Box<dyn Any>>) -> Option<Box<dyn Any>> + Send + Sync + 'static>;

/// A struct that holds metadata about a Function.
pub struct FnInfo {
    pub(crate) closure: GenericFn,
    pub(crate) arg_types: Vec<ExtDataType>,
    pub(crate) result_type: ExtDataType,
}

impl fmt::Debug for FnInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnInfo")
            // .field("z_fn", &self.z_fn)
            .field("arg_types", &self.arg_types)
            .field("return_type", &self.result_type)
            .finish()
    }
}

// FIXME (rsn) 20250820 - rewrite w/ a macro...
pub(crate) fn add_builtins(ctx: &mut Context) {
    // numeric functions as closures...
    let abs = |x: f64| x.abs();
    ctx.register(
        "abs",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(abs(*x)))
        },
    );

    let acos = |x: f64| x.acos();
    ctx.register(
        "acos",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(acos(*x)))
        },
    );

    let asin = |x: f64| x.asin();
    ctx.register(
        "asin",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(asin(*x)))
        },
    );

    let atan = |x: f64| x.atan();
    ctx.register(
        "atan",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(atan(*x)))
        },
    );

    let cbrt = |x: f64| x.cbrt();
    ctx.register(
        "cbrt",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(cbrt(*x)))
        },
    );

    let ceil = |x: f64| x.ceil();
    ctx.register(
        "ceil",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(ceil(*x)))
        },
    );

    let cos = |x: f64| x.cos();
    ctx.register(
        "cos",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(cos(*x)))
        },
    );

    let floor = |x: f64| x.floor();
    ctx.register(
        "floor",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(floor(*x)))
        },
    );

    let ln = |x: f64| x.ln();
    ctx.register(
        "ln",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(ln(*x)))
        },
    );

    let sin = |x: f64| x.sin();
    ctx.register(
        "sin",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(sin(*x)))
        },
    );

    let sqrt = |x: f64| x.sqrt();
    ctx.register(
        "sqrt",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(sqrt(*x)))
        },
    );

    let tan = |x: f64| x.tan();
    ctx.register(
        "tan",
        vec![ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            Some(Box::new(tan(*x)))
        },
    );

    let max = |x: f64, y: f64| x.max(y);
    ctx.register(
        "max",
        vec![ExtDataType::Num, ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            let y = args.get(1)?.downcast_ref::<f64>()?;
            Some(Box::new(max(*x, *y)))
        },
    );

    let avg = |x: f64, y: f64| x.midpoint(y);
    ctx.register(
        "avg",
        vec![ExtDataType::Num, ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            let y = args.get(1)?.downcast_ref::<f64>()?;
            Some(Box::new(avg(*x, *y)))
        },
    );

    let min = |x: f64, y: f64| x.min(y);
    ctx.register(
        "min",
        vec![ExtDataType::Num, ExtDataType::Num],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<f64>()?;
            let y = args.get(1)?.downcast_ref::<f64>()?;
            Some(Box::new(min(*x, *y)))
        },
    );

    // string builtins...
    let trim = |x: &QString| x.as_str().trim().to_owned();
    ctx.register(
        "trim",
        vec![ExtDataType::Str],
        ExtDataType::Str,
        move |args| {
            let x = args.first()?.downcast_ref::<QString>()?;
            Some(Box::new(trim(x)))
        },
    );

    let len = |x: &QString| x.as_str().len();
    ctx.register(
        "len",
        vec![ExtDataType::Str],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<QString>()?;
            Some(Box::new(len(x)))
        },
    );

    let concat = |x: &QString, y: &QString| format!("{}{}", x.as_str(), y.as_str());
    ctx.register(
        "concat",
        vec![ExtDataType::Str, ExtDataType::Str],
        ExtDataType::Str,
        move |args| {
            let x = args.first()?.downcast_ref::<QString>()?;
            let y = args.get(1)?.downcast_ref::<QString>()?;
            Some(Box::new(concat(x, y)))
        },
    );

    let starts_with = |x: &str, y: &str| x.starts_with(y);
    ctx.register(
        "starts_with",
        vec![ExtDataType::Str, ExtDataType::Str],
        ExtDataType::Bool,
        move |args| {
            let x = args.first()?.downcast_ref::<QString>()?.as_str();
            let y = args.get(1)?.downcast_ref::<QString>()?.as_str();
            Some(Box::new(starts_with(x, y)))
        },
    );

    let ends_with = |x: &str, y: &str| x.ends_with(y);
    ctx.register(
        "ends_with",
        vec![ExtDataType::Str, ExtDataType::Str],
        ExtDataType::Bool,
        move |args| {
            let x = args.first()?.downcast_ref::<QString>()?.as_str();
            let y = args.get(1)?.downcast_ref::<QString>()?.as_str();
            Some(Box::new(ends_with(x, y)))
        },
    );

    // temporal builtins...
    let now = || Zoned::now().with_time_zone(TimeZone::UTC);
    ctx.register("now", vec![], ExtDataType::Timestamp, move |_| {
        Some(Box::new(now()))
    });

    let today = || {
        let noon = Zoned::now()
            .with()
            .hour(12)
            .minute(0)
            .second(0)
            .subsec_nanosecond(0)
            .build()
            .expect("Failed finding today's date");
        noon.with_time_zone(TimeZone::UTC)
    };
    ctx.register("today", vec![], ExtDataType::Timestamp, move |_| {
        Some(Box::new(today()))
    });

    // spatial builtins...
    let boundary = |x: &Geometry| x.boundary().expect("Failed finding boundary");
    ctx.register(
        "boundary",
        vec![ExtDataType::Geom],
        ExtDataType::Geom,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            Some(Box::new(boundary(x)))
        },
    );

    let buffer = |x: &Geometry, y: &f64| x.buffer(*y, 8).expect("Failed computing buffer");
    ctx.register(
        "buffer",
        vec![ExtDataType::Geom, ExtDataType::Num],
        ExtDataType::Geom,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            let y = args.get(1)?.downcast_ref::<f64>()?;
            Some(Box::new(buffer(x, y)))
        },
    );

    let envelope = |x: &Geometry| x.envelope().expect("Failed finding envelope");
    ctx.register(
        "envelope",
        vec![ExtDataType::Geom],
        ExtDataType::Geom,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            Some(Box::new(envelope(x)))
        },
    );

    let centroid = |x: &Geometry| x.get_centroid().expect("Failed finding centroid");
    ctx.register(
        "centroid",
        vec![ExtDataType::Geom],
        ExtDataType::Geom,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            Some(Box::new(centroid(x)))
        },
    );

    let convex_hull = |x: &Geometry| x.convex_hull().expect("Failed finding convex hull");
    ctx.register(
        "convex_hull",
        vec![ExtDataType::Geom],
        ExtDataType::Geom,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            Some(Box::new(convex_hull(x)))
        },
    );

    let get_x = |x: &Geometry| x.get_x().expect("Failed isolating X");
    ctx.register(
        "get_x",
        vec![ExtDataType::Geom],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            Some(Box::new(get_x(x)))
        },
    );

    let get_y = |x: &Geometry| x.get_y().expect("Failed isolating Y");
    ctx.register(
        "get_y",
        vec![ExtDataType::Geom],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            Some(Box::new(get_y(x)))
        },
    );

    let get_z = |x: &Geometry| x.get_y().expect("Failed isolating Z");
    ctx.register(
        "get_z",
        vec![ExtDataType::Geom],
        ExtDataType::Num,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            Some(Box::new(get_z(x)))
        },
    );

    let wkt = |x: &Geometry| x.to_wkt().expect("Failed generating WKT");
    ctx.register(
        "wkt",
        vec![ExtDataType::Geom],
        ExtDataType::Str,
        move |args| {
            let x = args.first()?.downcast_ref::<Geometry>()?;
            Some(Box::new(wkt(x)))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use std::error::Error;

    #[test]
    // #[tracing_test::traced_test]
    fn test_unregistered() -> Result<(), Box<dyn Error>> {
        let shared_ctx = Context::new().freeze();

        let expr = Expression::try_from_text("sum(a, b)")?;
        let mut eval = EvaluatorImpl::new(shared_ctx);
        eval.setup(expr)?;

        let feat = Resource::new();
        let res = eval.evaluate(&feat)?;
        // tracing::debug!("res = {res:?}");
        assert!(matches!(res, Outcome::N));

        eval.teardown()?;

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_literals() -> Result<(), Box<dyn Error>> {
        let sum = |x: f64, y: f64| x + y;

        let mut ctx = Context::new();
        ctx.register(
            "sum",
            vec![ExtDataType::Num, ExtDataType::Num],
            ExtDataType::Num,
            move |args| {
                let a1 = args.get(0)?.downcast_ref::<f64>()?;
                let a2 = args.get(1)?.downcast_ref::<f64>()?;
                Some(Box::new(sum(*a1, *a2))) // Call the user-defined closure
            },
        );
        let shared_ctx = ctx.freeze();

        let expr = Expression::try_from_text("3 = sum(1, 2)")?;
        let mut eval = EvaluatorImpl::new(shared_ctx);
        eval.setup(expr)?;

        let feat = Resource::new();

        let res = eval.evaluate(&feat)?;
        // tracing::debug!("res = {res:?}");
        assert!(matches!(res, Outcome::T));

        eval.teardown()?;

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_queryables() -> Result<(), Box<dyn Error>> {
        let sum = |x: f64, y: f64| x + y;

        let mut ctx = Context::new();
        ctx.register(
            "sum",
            vec![ExtDataType::Num, ExtDataType::Num],
            ExtDataType::Num,
            move |args| {
                let a1 = args.get(0)?.downcast_ref::<f64>()?;
                let a2 = args.get(1)?.downcast_ref::<f64>()?;
                Some(Box::new(sum(*a1, *a2))) // Call the user-defined closure
            },
        );
        let shared_ctx = ctx.freeze();

        let expr = Expression::try_from_text("30 = sum(a, b)")?;
        let mut eval = EvaluatorImpl::new(shared_ctx);
        eval.setup(expr)?;

        let feat = Resource::from([
            ("fid".into(), Q::try_from(1)?),
            ("a".into(), Q::try_from(10)?),
            ("b".into(), Q::try_from(20.0)?),
        ]);

        let res = eval.evaluate(&feat)?;
        // tracing::debug!("res = {res:?}");
        assert!(matches!(res, Outcome::T));

        eval.teardown()?;

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_wrong_data_type() -> Result<(), Box<dyn Error>> {
        let sum = |x: f64, y: f64| x + y;

        let mut ctx = Context::new();
        ctx.register(
            "sum",
            vec![ExtDataType::Num, ExtDataType::Num],
            ExtDataType::Num,
            move |args| {
                let a1 = args.get(0)?.downcast_ref::<f64>()?;
                let a2 = args.get(1)?.downcast_ref::<f64>()?;
                Some(Box::new(sum(*a1, *a2))) // Call the user-defined closure
            },
        );
        let shared_ctx = ctx.freeze();

        let expr = Expression::try_from_text("30 = sum(a, b)")?;
        let mut eval = EvaluatorImpl::new(shared_ctx);
        eval.setup(expr)?;

        let feat = Resource::from([
            ("fid".into(), Q::try_from(1)?),
            ("a".into(), Q::try_from(10)?),
            ("b".into(), Q::new_plain_str("20.0")),
        ]);

        let res = eval.evaluate(&feat);
        // tracing::debug!("res = {res:?}");
        assert!(res.is_err());

        eval.teardown()?;

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_num_builtins() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::new();
        ctx.register_builtins();
        let shared_ctx = ctx.freeze();

        let expr = Expression::try_from_text("min(a, b) + max(a, b) = 2 * avg(a, b)")?;
        let mut eval = EvaluatorImpl::new(shared_ctx);
        eval.setup(expr)?;

        let feat = Resource::from([
            ("fid".into(), Q::try_from(1)?),
            ("a".into(), Q::try_from(10)?),
            ("b".into(), Q::try_from(20)?),
        ]);

        let res = eval.evaluate(&feat)?;
        // tracing::debug!("res = {res:?}");
        assert!(matches!(res, Outcome::T));

        Ok(())
    }

    #[test]
    // #[tracing_test::traced_test]
    fn test_geom_builtins() -> Result<(), Box<dyn Error>> {
        // IMPORTANT (rsn) 20250901 - if we rely on Context::new() we leave
        // the context subject to the global configuration which may be using
        // an implicit CRS code that's unexpected for the test.  specifically
        // the pre-conditions of this test expect WGS-84 coordinates...
        let mut ctx = Context::try_with_crs("epsg:4326")?;
        ctx.register_builtins();
        let shared_ctx = ctx.freeze();

        let expr = Expression::try_from_text(
            "wkt(centroid(envelope(MULTIPOINT(0 90, 90 0)))) = 'POINT (45 45)'",
        )?;
        let mut eval = EvaluatorImpl::new(shared_ctx);
        eval.setup(expr)?;

        let feat = Resource::new();

        let res = eval.evaluate(&feat)?;
        // tracing::debug!("res = {res:?}");
        assert!(matches!(res, Outcome::T));

        Ok(())
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_str_builtins() -> Result<(), Box<dyn Error>> {
        let mut ctx = Context::new();
        ctx.register_builtins();
        let shared_ctx = ctx.freeze();

        let expr = Expression::try_from_text("starts_with(concat('foo', 'bar'), 'fo')")?;
        let mut eval = EvaluatorImpl::new(shared_ctx);
        eval.setup(expr)?;

        let feat = Resource::new();

        let res = eval.evaluate(&feat)?;
        tracing::debug!("res = {res:?}");
        assert!(matches!(res, Outcome::T));

        Ok(())
    }
}
