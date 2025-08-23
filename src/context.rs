// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Expressions evaluation context.
//!

use crate::{ExtDataType, FnInfo, add_builtins, crs::CRS};
use core::fmt;
use std::{any::Any, collections::HashMap, rc::Rc};

/// A Context object we will be handing to evaluators so they are aware of
/// external registered _Functions_.
#[derive(Default)]
pub struct Context {
    crs: CRS,
    pub(crate) functions: HashMap<String, FnInfo>,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("crs", &self.crs)
            .field("functions", &self.functions)
            .finish()
    }
}

impl Context {
    /// Create a new instance w/ no registered functions.
    pub fn new() -> Self {
        Context {
            crs: CRS::default(),
            functions: HashMap::with_capacity(5),
        }
    }

    /// Combine creation w/ sharing in one convenience method.
    pub fn new_shared() -> SharedContext {
        let default_ctx = Context::default();
        Rc::new(default_ctx)
    }

    /// Register a Function (Rust Closure) by name with expected argument(s)
    /// and result types.
    pub fn register<F>(
        &mut self,
        name: &str,
        arg_types: Vec<ExtDataType>,
        result_type: ExtDataType,
        closure: F,
    ) where
        F: Fn(Vec<Box<dyn Any>>) -> Option<Box<dyn Any>> + Send + Sync + 'static,
    {
        self.functions.insert(
            name.to_string(),
            FnInfo {
                closure: Box::new(closure),
                arg_types,
                result_type,
            },
        );
    }

    /// Return a safe share-able read-only version of this frozen at the time
    /// of the call.
    pub fn freeze(self) -> SharedContext {
        Rc::new(self) //.clone()
    }

    /// Return a reference to the currently set CRS w/in this.
    pub fn crs(&self) -> &CRS {
        &self.crs
    }

    /// Return meta-information about a Function already registered in this.
    pub fn fn_info(&self, name: &str) -> Option<&FnInfo> {
        self.functions.get(name)
    }

    /// Register all builtin functions we support.
    pub fn register_builtins(&mut self) {
        add_builtins(self);
    }
}

/// What we share between [Evaluator][crate::Evaluator]s.
pub type SharedContext = Rc<Context>;
