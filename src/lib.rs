// lib.rs
mod common;

extern crate proc_macro;
use self::proc_macro::TokenStream;

use quote::quote;

use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

use crate::common::dollar_values;

/// Create method for inserting struts into Sqlite database
///
/// ```rust
/// # #[tokio::main]
/// # async fn main() -> sqlx::Result<()>{
/// #[derive(Default, Debug, sqlx::FromRow, sqlxinsert::SqliteInsert)]
/// struct Car {
///     pub car_id: i32,
///     pub car_name: String,
/// }
///
/// let car = Car {
///     car_id: 33,
///     car_name: "Skoda".to_string(),
/// };
///
/// let url = "sqlite::memory:";
/// let pool = sqlx::sqlite::SqlitePoolOptions::new().connect(url).await.unwrap();
///
/// let create_table = "create table cars ( car_id INTEGER PRIMARY KEY, car_name TEXT NOT NULL )";
/// sqlx::query(create_table).execute(&pool).await.expect("Not possible to execute");
///
/// let res = car.insert_raw(&pool, "cars").await.unwrap(); // returning id
/// # Ok(())
/// # }
/// ```
///
#[cfg(feature = "sqlite")]
#[proc_macro_derive(SqliteInsert)]
pub fn derive_from_struct_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    // COMMON Atrributes
    let struct_name = &input.ident;

    // INSERT Attributes -> field names
    let attributes = fields.iter().map(|field| &field.ident);
    let attributes_vec: Vec<String> = fields
        .iter()
        .map(|field| {
            field
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect();

    // ( id, name, hostname .. )
    let columns = attributes_vec.join(",");
    // ( $1, $2)
    let dollars = dollar_values(attributes_vec.len());

    // UPDATE Attributes -> field names for
    let attributes_update = fields.iter().map(|field| &field.ident);
    // name = $2, hostname = $3
    let pairs: String = attributes_vec
        .iter()
        .enumerate()
        .skip(1) // Skip the first element
        .map(|(index, value)| {
            let number = index + 1; // Start with $2
            format!("{} = ${}", value, number)
        })
        .collect::<Vec<String>>()
        .join(",");

    TokenStream::from(quote! {

        impl #struct_name {
            pub fn insert_query(&self, table: &str) -> String
            {
                let sqlquery = format!("insert into {} ( {} ) values ( {} )", table, #columns, #dollars);
                sqlquery
            }

            pub async fn insert_raw(&self, pool: &sqlx::SqlitePool, table: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
                let sql = self.insert_query(table);
                sqlx::query(&sql)
                    #(
                        .bind(&self.#attributes)
                    )*
                    .execute(pool)
                    .await
            }

            pub fn update_query(&self, table: &str) -> String
            {
                let sqlquery = format!("update {} set {} where id = $1", table, #pairs);
                sqlquery
            }

            pub async fn update_raw(&self, pool: &sqlx::SqlitePool, table: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
                let sql = self.update_query(table);
                sqlx::query(&sql)
                    #(
                        .bind(&self.#attributes_update)
                    )*
                    .execute(pool)
                    .await
            }
        }
    })
}

/// Create method for inserting struts into Postgres database
///
/// ```rust,ignore
/// # #[tokio::main]
/// # async fn main() -> sqlx::Result<()> {
///
/// #[derive(Default, Debug, std::cmp::PartialEq, sqlx::FromRow)]
/// struct Car {
///     pub id: i32,
///     pub name: String,
/// }
///
/// #[derive(Default, Debug, sqlx::FromRow, sqlxinsert::PgInsert)]
/// struct CreateCar {
///     pub name: String,
///     pub color: Option<String>,
/// }
/// impl CreateCar {
///     pub fn new<T: Into<String>>(name: T) -> Self {
///         CreateCar {
///             name: name.into(),
///             color: None,
///         }
///     }
/// }
/// let url = "postgres://user:pass@localhost:5432/test_db";
/// let pool = sqlx::postgres::PgPoolOptions::new().connect(&url).await.unwrap();
///
/// let car_skoda = CreateCar::new("Skoda");
/// let res: Car = car_skoda.insert::<Car>(pool, "cars").await?;
/// # Ok(())
/// # }
/// ```
///
#[cfg(feature = "postgres")]
#[proc_macro_derive(PgInsert)]
pub fn derive_from_struct_psql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    // COMMON Atrributes
    let struct_name = &input.ident;

    // INSERT Attributes -> field names
    let attributes = fields.iter().map(|field| &field.ident);
    let attributes_vec: Vec<String> = fields
        .iter()
        .map(|field| {
            field
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect();

    // ( id, name, hostname .. )
    let columns = attributes_vec.join(",");
    // ( $1, $2)
    let dollars = dollar_values(attributes_vec.len());

    // UPDATE Attributes -> field names for
    let attributes_update = fields.iter().map(|field| &field.ident);
    // name = $2, hostname = $3
    let pairs: String = attributes_vec
        .iter()
        .enumerate()
        .skip(1) // Skip the first element
        .map(|(index, value)| {
            let number = index + 1; // Start with $2
            format!("{} = ${}", value, number)
        })
        .collect::<Vec<String>>()
        .join(",");

    TokenStream::from(quote! {
        impl #struct_name {
            fn insert_query(&self, table: &str) -> String
            {
                let sqlquery = format!("insert into {} ( {} ) values ( {} ) returning *", table, #columns, #dollars); // self.value_list()); //self.values );
                sqlquery
            }

            pub async fn insert<T>(&self, pool: &sqlx::PgPool, table: &str) -> sqlx::Result<T>
            where
                T: Send,
                T: for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
                T: std::marker::Unpin
            {
                let sql = self.insert_query(table);

                // let mut pool = pool;
                let res: T = sqlx::query_as::<_,T>(&sql)
                #(
                    .bind(&self.#attributes) //         let #field_name: #field_type = Default::default();
                )*
                    .fetch_one(pool)
                    .await?;

                Ok(res)
            }

            fn update_query(&self, table: &str) -> String
            {
                let sqlquery = format!("update {} set {} where id = $1 returning *", table, #pairs);
                sqlquery
            }

            pub async fn update<T>(&self, pool: &sqlx::PgPool, table: &str) -> sqlx::Result<T>
            where
                T: Send,
                T: for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow>,
                T: std::marker::Unpin
            {
                let sql = self.update_query(table);

                // let mut pool = pool;
                let res: T = sqlx::query_as::<_,T>(&sql)
                #(
                    .bind(&self.#attributes_update)//         let #field_name: #field_type = Default::default();
                )*
                    .fetch_one(pool)
                    .await?;

                Ok(res)
            }
        }
    })
}
