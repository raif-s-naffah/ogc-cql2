// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! OGC CQL2 evaluator trait and implementations...
//!

use crate::{
    E, Expression, GTrait, MyError, Outcome, Q, Resource, SharedContext, text::cql2::expression,
};
use tracing::{debug, error};

/// Capability of processing OGC CQL2 [expressions][Expression], both text- and json-encoded.
pub trait Evaluator {
    /// Setup an instance to operate with a given [Expression].
    fn setup(&mut self, expr: Expression) -> Result<(), MyError>;

    /// Evaluate a given [Resource] returning an [Outcome], or raise a
    /// [MyError] if an unexpected error occurs in the process.
    fn evaluate(&self, f: &Resource) -> Result<Outcome, MyError>;
}

/// A concrete [evaluator][Evaluator] that does the work w/o relying on any external
/// source or capability that may be available in high-level data sources such as a
/// database engine endowed w/ spatial and other operators.
#[derive(Debug)]
pub struct ExEvaluator {
    /// Runtime context w/in which [Resource]s will be evaluated.
    shared_ctx: SharedContext,
    /// Valid/parsed OGC CQL2 expression.
    exp: E,
}

impl ExEvaluator {
    /// Create a new instance using the given [SharedContext].
    pub fn new(ctx: SharedContext) -> Self {
        Self {
            shared_ctx: ctx,
            exp: E::default(),
        }
    }
}

impl Evaluator for ExEvaluator {
    fn setup(&mut self, input: Expression) -> Result<(), MyError> {
        // if we're JSON-encoded, convert to Text-encoded.
        let mut exp = match input {
            Expression::Text(text) => text.0,
            Expression::Json(json) => {
                // attempt parsing the to_string() output of inner object...
                let text = json.0.to_string();
                debug!("About to parse '{text}'");
                expression(&text).map_err(MyError::Text)?
            }
        };
        let it = E::reduce(&mut exp)?;
        tracing::trace!("setup (redux): {it}");
        self.exp = it;
        Ok(())
    }

    // #[tracing::instrument(level="trace", skip_all, ret)]
    fn evaluate(&self, feature: &Resource) -> Result<Outcome, MyError> {
        // let _ev_ = tracing::span!(tracing::Level::DEBUG, "L2").entered();
        // let _ev_start = _ev_.enter();

        let ctx = &self.shared_ctx;
        match self.exp.eval(ctx, feature)? {
            Q::Null => Ok(Outcome::N),
            Q::Bool(x) => match x {
                true => Ok(Outcome::T),
                false => Ok(Outcome::F),
            },
            Q::Num(x) => {
                error!("Unexpected number: {x}");
                Ok(Outcome::N)
            }
            Q::Str(x) => {
                error!("Unexpected string: '{}'", &x);
                Ok(Outcome::N)
            }
            Q::Geom(x) => {
                error!("Unexpected geometry: {}", x.to_wkt());
                Ok(Outcome::N)
            }
            Q::Instant(x) => {
                error!("Unexpected instant: {x}");
                Ok(Outcome::N)
            }
            Q::Interval(x, y) => {
                error!("Unexpected interval: [{x}..{y}]");
                Ok(Outcome::N)
            }
            Q::List(x) => {
                error!("Unexpected list: {x:?}");
                Ok(Outcome::N)
            }
        }
    }
}
