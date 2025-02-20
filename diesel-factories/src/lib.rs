//! This is an implementation the test factory pattern made to work with [Diesel][].
//!
//! [Diesel]: https://diesel.rs
//!
//! Example usage:
//!
//! ```
//! #[macro_use]
//! extern crate diesel;
//!
//! use diesel_factories::{Association, Factory};
//! use diesel::{pg::PgConnection, prelude::*};
//!
//! // Tell Diesel what our schema is
//! mod schema {
//!     table! {
//!         countries (id) {
//!             id -> Integer,
//!             name -> Text,
//!         }
//!     }
//!
//!     table! {
//!         cities (id) {
//!             id -> Integer,
//!             name -> Text,
//!             country_id -> Integer,
//!         }
//!     }
//! }
//!
//! // Our city model
//! #[derive(Clone, Queryable)]
//! struct City {
//!     pub id: i32,
//!     pub name: String,
//!     pub country_id: i32,
//! }
//!
//! #[derive(Clone, Factory)]
//! #[factory(
//!     // model type our factory inserts
//!     model = "City",
//!     // table the model belongs to
//!     table = "crate::schema::cities",
//!     // connection type you use. Defaults to `PgConnection`
//!     connection = "diesel::pg::PgConnection",
//!     // type of primary key. Defaults to `i32`
//!     id = "i32",
//! )]
//! struct CityFactory<'a> {
//!     pub name: String,
//!     // A `CityFactory` is associated to either an inserted `&'a Country` or a `CountryFactory`
//!     // instance.
//!     pub country: Association<'a, Country, CountryFactory>,
//! }
//!
//! // We make new factory instances through the `Default` trait
//! impl<'a> Default for CityFactory<'a> {
//!     fn default() -> Self {
//!         Self {
//!             name: "Copenhagen".to_string(),
//!             // `default` will return an `Association` with a `CountryFactory`. No inserts happen
//!             // here.
//!             //
//!             // This is the same as `Association::Factory(CountryFactory::default())`.
//!             country: Association::default(),
//!         }
//!     }
//! }
//!
//! // The same setup, but for `Country`
//! #[derive(Clone, Queryable)]
//! struct Country {
//!     pub id: i32,
//!     pub name: String,
//! }
//!
//! #[derive(Clone, Factory)]
//! #[factory(
//!     model = "Country",
//!     table = "crate::schema::countries",
//!     connection = "diesel::pg::PgConnection",
//!     id = "i32",
//! )]
//! struct CountryFactory {
//!     pub name: String,
//! }
//!
//! impl Default for CountryFactory {
//!     fn default() -> Self {
//!         Self {
//!             name: "Denmark".into(),
//!         }
//!     }
//! }
//!
//! // Usage
//! fn basic_usage() {
//!     let con = establish_connection();
//!
//!     let city = CityFactory::default().insert(&con);
//!     assert_eq!("Copenhagen", city.name);
//!
//!     let country = find_country_by_id(city.country_id, &con);
//!     assert_eq!("Denmark", country.name);
//!
//!     assert_eq!(1, count_cities(&con));
//!     assert_eq!(1, count_countries(&con));
//! }
//!
//! fn setting_fields() {
//!     let con = establish_connection();
//!
//!     let city = CityFactory::default()
//!         .name("Amsterdam")
//!         .country(CountryFactory::default().name("Netherlands"))
//!         .insert(&con);
//!     assert_eq!("Amsterdam", city.name);
//!
//!     let country = find_country_by_id(city.country_id, &con);
//!     assert_eq!("Netherlands", country.name);
//!
//!     assert_eq!(1, count_cities(&con));
//!     assert_eq!(1, count_countries(&con));
//! }
//!
//! fn multiple_models_with_same_association() {
//!     let con = establish_connection();
//!
//!     let netherlands = CountryFactory::default()
//!         .name("Netherlands")
//!         .insert(&con);
//!
//!     let amsterdam = CityFactory::default()
//!         .name("Amsterdam")
//!         .country(&netherlands)
//!         .insert(&con);
//!
//!     let hague = CityFactory::default()
//!         .name("The Hague")
//!         .country(&netherlands)
//!         .insert(&con);
//!
//!     assert_eq!(amsterdam.country_id, hague.country_id);
//!
//!     assert_eq!(2, count_cities(&con));
//!     assert_eq!(1, count_countries(&con));
//! }
//! #
//! # fn main() {
//! #     basic_usage();
//! #     setting_fields();
//! #     multiple_models_with_same_association();
//! # }
//! # fn establish_connection() -> PgConnection {
//! #     let database_url = "postgres://localhost/diesel_factories_test";
//! #     let con = PgConnection::establish(&database_url).unwrap();
//! #     con.begin_test_transaction().unwrap();
//! #     con
//! # }
//!
//! // Utility functions just for demo'ing
//! fn count_cities(con: &PgConnection) -> i64 {
//!     use crate::schema::cities;
//!     use diesel::dsl::count_star;
//!     cities::table.select(count_star()).first(con).unwrap()
//! }
//!
//! fn count_countries(con: &PgConnection) -> i64 {
//!     use crate::schema::countries;
//!     use diesel::dsl::count_star;
//!     countries::table.select(count_star()).first(con).unwrap()
//! }
//!
//! fn find_country_by_id(input: i32, con: &PgConnection) -> Country {
//!     use crate::schema::countries::dsl::*;
//!     countries
//!         .filter(id.eq(&input))
//!         .first::<Country>(con)
//!         .unwrap()
//! }
//! ```
//!
//! ## `#[derive(Factory)]`
//!
//! ### Attributes
//!
//! | Name | Description | Example | Default |
//! |---|---|---|---|
//! | `model` | Model type your factory inserts | `"City"` | None, required |
//! | `table` | Table your model belongs to | `"crate::schema::cities"` | None, required |
//! | `connection` | The connection type your app uses | `"MysqlConnection"` | `"diesel::pg::PgConnection"` |
//! | `id` | The type of your table's primary key | `"i64"` | `"i32"` |
//!
//! ### Builder methods
//!
//! Besides implementing [`Factory`] for your struct it will also derive builder methods for easily customizing each field. The generated code looks something like this:
//!
//! ```
//! struct CountryFactory {
//!     pub name: String,
//! }
//!
//! // This is what gets generated for each field
//! impl CountryFactory {
//!     fn name<T: Into<String>>(mut self, new: T) -> Self {
//!         self.name = new.into();
//!         self
//!     }
//! }
//! #
//! # impl Default for CountryFactory {
//! #     fn default() -> Self {
//! #         CountryFactory { name: String::new() }
//! #     }
//! # }
//!
//! // So you can do this
//! CountryFactory::default().name("Amsterdam");
//! ```
//!
//! [`Factory`]: trait.Factory.html
//!
//! ### Builder methods for associations
//!
//! The builder methods generated for `Association` fields are a bit different. If you have a factory like:
//!
//! ```
//! # include!("../tests/docs_setup.rs");
//! #
//! #[derive(Clone, Factory)]
//! #[factory(
//!     model = "City",
//!     table = "crate::schema::cities",
//! )]
//! struct CityFactory<'a> {
//!     pub name: String,
//!     pub country: Association<'a, Country, CountryFactory>,
//! }
//! #
//! # impl<'a> Default for CityFactory<'a> {
//! #     fn default() -> Self {
//! #         unimplemented!()
//! #     }
//! # }
//! #
//! # fn main() {}
//! ```
//!
//! You'll be able to call `country` either with an owned `CountryFactory`:
//!
//! ```
//! # include!("../tests/docs_setup.rs");
//! #
//! # #[derive(Clone, Factory)]
//! # #[factory(
//! #     model = "City",
//! #     table = "crate::schema::cities",
//! # )]
//! # struct CityFactory<'a> {
//! #     pub name: String,
//! #     pub country: Association<'a, Country, CountryFactory>,
//! # }
//! #
//! # impl<'a> Default for CityFactory<'a> {
//! #     fn default() -> Self {
//! #         Self {
//! #             name: String::new(), country: Association::default(),
//! #         }
//! #     }
//! # }
//! #
//! # fn main() {
//! let country_factory = CountryFactory::default();
//! CityFactory::default().country(country_factory);
//! # }
//! ```
//!
//! Or a borrowed `Country`:
//!
//! ```
//! # include!("../tests/docs_setup.rs");
//! #
//! # #[derive(Clone, Factory)]
//! # #[factory(
//! #     model = "City",
//! #     table = "crate::schema::cities",
//! # )]
//! # struct CityFactory<'a> {
//! #     pub name: String,
//! #     pub country: Association<'a, Country, CountryFactory>,
//! # }
//! #
//! # impl<'a> Default for CityFactory<'a> {
//! #     fn default() -> Self {
//! #         Self {
//! #             name: String::new(), country: Association::default(),
//! #         }
//! #     }
//! # }
//! #
//! # fn main() {
//! let country = Country { id: 1, name: "Denmark".into() };
//! CityFactory::default().country(&country);
//! # }
//! ```
//!
//! This should prevent bugs where you have multiple factory instances sharing some association that you mutate halfway through a test.
//!
//! ### Optional associations
//!
//! If your model has a nullable association you can do this:
//!
//! ```
//! # include!("../tests/docs_setup_with_city_factory.rs");
//! #
//! #[derive(Clone, Factory)]
//! #[factory(
//!     model = "User",
//!     table = "crate::schema::users",
//! )]
//! struct UserFactory<'a> {
//!     pub name: String,
//!     pub country: Option<Association<'a, Country, CountryFactory>>,
//! #   pub age: i32,
//! #   pub home_city: Option<Association<'a, City, CityFactory<'a>>>,
//! #   pub current_city: Option<Association<'a, City, CityFactory<'a>>>,
//! }
//!
//! impl<'a> Default for UserFactory<'a> {
//!     fn default() -> Self {
//!         Self {
//!             name: "Bob".into(),
//!             country: None,
//! #           age: 30,
//! #           home_city: None,
//! #           current_city: None,
//!         }
//!     }
//! }
//!
//! # fn main() {
//! // Setting `country` to a `CountryFactory`
//! let country_factory = CountryFactory::default();
//! UserFactory::default().country(Some(country_factory));
//!
//! // Setting `country` to a `Country`
//! let country = Country { id: 1, name: "Denmark".into() };
//! UserFactory::default().country(Some(&country));
//!
//! // Setting `country` to `None`
//! UserFactory::default().country(Option::<CountryFactory>::None);
//! UserFactory::default().country(Option::<&Country>::None);
//! # }
//! ```

#![doc(html_root_url = "https://docs.rs/diesel-factories/0.1.1")]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use lazy_static::lazy_static;
use std::sync::atomic::{AtomicUsize, Ordering};

pub use diesel_factories_code_gen::Factory;

/// A "belongs to" association that may or may not have been inserted yet.
///
/// You will normally be using this when setting up "belongs to" associations between models in
/// factories.
#[derive(Debug, Clone)]
pub enum Association<'a, Model, Factory> {
    /// An associated model that has been inserted into the database.
    ///
    /// You shouldn't have to use this direclty but instead just `Association::default()`.
    Model(&'a Model),

    /// A factory for a model that hasn't been inserted yet into the database.
    ///
    /// You shouldn't have to use this direclty but instead just `Association::default()`.
    Factory(Factory),
}

impl<Model, Factory: Default> Default for Association<'_, Model, Factory> {
    fn default() -> Self {
        Association::Factory(Factory::default())
    }
}

impl<'a, Model, Factory> Association<'a, Model, Factory> {
    #[doc(hidden)]
    pub fn new_model(inner: &'a Model) -> Self {
        Association::Model(inner)
    }

    #[doc(hidden)]
    pub fn new_factory(inner: Factory) -> Self {
        Association::Factory(inner)
    }
}

impl<M, F> Association<'_, M, F>
where
    F: Factory<Model = M> + Clone,
{
    #[doc(hidden)]
    pub fn insert_returning_id(&self, con: &F::Connection) -> F::Id {
        match self {
            Association::Model(model) => F::id_for_model(&model).clone(),
            Association::Factory(factory) => {
                let model = factory.clone().insert(con);
                F::id_for_model(&model).clone()
            }
        }
    }
}

/// A generic factory trait.
///
/// You shouldn't ever have to implement this trait yourself. It can be derived using
/// `#[derive(Factory)]`
///
/// See the [root module docs](/) for info on how to use `#[derive(Factory)]`.
pub trait Factory: Clone {
    /// The model type the factory inserts.
    ///
    /// For a factory named `UserFactory` this would probably be `Use`.
    type Model;

    /// The primary key type your model uses.
    ///
    /// This will normally be i32 or i64 but can be whatever you need.
    type Id: Clone;

    /// The database connection type you use such as `diesel::pg::PgConnection`.
    type Connection;

    /// Insert the factory into the database.
    ///
    /// # Panics
    /// This will panic if the insert fails. Should be fine since you want panics early in tests.
    fn insert(self, con: &Self::Connection) -> Self::Model;

    /// Get the primary id value for a model type.
    ///
    /// Just a generic wrapper around `model.id`.
    fn id_for_model(model: &Self::Model) -> &Self::Id;
}

lazy_static! {
    static ref SEQUENCE_COUNTER: AtomicUsize = { AtomicUsize::new(0) };
}

/// Utility function for generating unique ids or strings in factories.
/// Each time `sequence` gets called, the closure will receive a different number.
///
/// ```
/// use diesel_factories::sequence;
///
/// assert_ne!(
///     sequence(|i| format!("unique-string-{}", i)),
///     sequence(|i| format!("unique-string-{}", i)),
/// );
/// ```
pub fn sequence<T, F>(f: F) -> T
where
    F: Fn(usize) -> T,
{
    SEQUENCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let count = SEQUENCE_COUNTER.load(Ordering::Relaxed);
    f(count)
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_compile_pass() {
        let t = trybuild::TestCases::new();
        t.pass("tests/compile_pass/*.rs");
    }
}
