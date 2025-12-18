// SPDX-License-Identifier: Apache-2.0

#![warn(missing_docs)]

//! Geospatial data stored in CSV files.
//!

use crate::ds::DataSource;
use std::path::PathBuf;

/// [`DataSource`] of _Features_ and [Resources][crate::Resource] mapped from CSV rows/records.
#[derive(Debug)]
pub struct CSVDataSource {
    path: PathBuf,
}

impl DataSource for CSVDataSource {
    fn srid(&self) -> Option<u32> {
        None
    }
}

impl CSVDataSource {
    /// Constructor given the file system location of an accessible CSV file.
    pub fn from(s: &str) -> Self {
        Self { path: s.into() }
    }

    /// Return this CSV data source path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Macro to generate a concrete [CSVDataSource].
///
/// Caller must provide the following parameters:
/// * `$vis`: Visibility specifier of the generated artifacts; e.g. `pub`.
/// * `$name`: Prefix of the concrete data source structure name to materialize.
///   The final name will have a 'CSV' suffix appended; eg. `Foo` -> `FooCSV`.
/// * `$path`: Path to a readable CSV file.
/// * `$feature`: `serde` deserializable structure that maps rows to _Features_.
#[macro_export]
macro_rules! gen_csv_ds {
    ($vis:vis, $name:expr, $path:expr, $feature:expr) => {
        ::paste::paste! {
            /// Concrete data source.
            #[derive(Debug)]
            $vis struct [<$name CSV>](CSVDataSource);

            impl [<$name CSV>] {
                /// Construct a new CSV data source.
                $vis fn new() -> Self {
                    Self(CSVDataSource::from($path))
                }

                /// Return a file reader that deserializes rows into features.
                $vis fn reader(&self) -> Result<::csv::Reader<::std::fs::File>, MyError> {
                    let file = ::std::fs::File::open(self.0.path())?;
                    Ok(::csv::Reader::from_reader(file))
                }
            }

            impl ::core::fmt::Display for [<$name CSV>] {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}CSV({})", $name, $path)
                }
            }

            impl IterableDS for [<$name CSV>] {
                type Item = $feature;
                type Err = MyError;

                fn iter(&self) -> Result<impl Iterator<Item = Result<$feature, Self::Err>>, Self::Err> {
                    let file = ::std::fs::File::open(&self.0.path())?;
                    let rdr = ::csv::Reader::from_reader(file);
                    let it = rdr.into_deserialize().map(|res| res.map_err(MyError::from));
                    Ok(it)
                }
            }
        }
    }
}
