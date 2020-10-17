extern crate proc_macro;
use self::proc_macro::TokenStream;

use quote::quote;

use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

/// 2 -> ( $1,$2 )
fn dollar_values(max: usize) -> String {
    let itr = 1..max + 1;
    itr.into_iter()
        .map(|s| format!("${}", s))
        .collect::<Vec<String>>()
        .join(",")
}

/// Create method for inserting struts into Sqlite database
///
/// ```rust,ignore
/// # #[tokio::main]
/// # async fn main() -> eyre::Result<()>{
/// #[derive(Default, Debug, sqlx::FromRow, gmacro::SqliteInsert)]
/// struct Car {
///     pub car_id: i32,
///     pub car_name: String,
/// }
///
/// let car = Car {
///     car_id: 33,
///     car_name: "Skoda".to_string(),
/// };
/// // bug: https://github.com/launchbadge/sqlx/issues/530
/// let url = "sqlite:%3Amemory:";
/// let pool = sqlx::SqlitePool::builder().build(&url).await.unwrap();
///
/// let res = car.insert(&pool, "cars").await.unwrap();
/// # Ok(())
/// # }
/// ```
///
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

    // Attributes -> field names
    let field_name = fields.iter().map(|field| &field.ident);
    let field_name2 = fields.iter().map(|field| &field.ident);

    let struct_name = &input.ident;

    let field_length = field_name.len();
    // ( $1, $2)
    let values = dollar_values(field_length);

    let fields_list = quote! {
        #( #field_name ),*
    };
    let columns = format!("{}", fields_list);

    TokenStream::from(quote! {

        impl #struct_name {
            pub fn insert_query(&self, table: &str) -> String
            {
                let sqlquery = format!("insert into {} ( {} ) values ( {} )", table, #columns, #values); //self.values );
                sqlquery
            }

            pub async fn insert(&self, mut pool: &sqlx::Pool<sqlx::SqliteConnection>, table: &str) -> eyre::Result<u64>
            {
                let sql = self.insert_query(table);
                let mut pool = pool;
                Ok(sqlx::query(&sql)
                #(
                    .bind(&self.#field_name2)//         let #field_name: #field_type = Default::default();
                )*
                    .execute(&mut pool)// (&mut conn)
                    .await?
                )
            }
        }
    })
}

/// Create method for inserting struts into Postgres database
///
/// ```rust,ignore
/// # #[tokio::main]
/// # async fn main() -> eyre::Result<()>{
///
/// #[derive(Default, Debug, std::cmp::PartialEq, sqlx::FromRow)]
/// struct Car {
///     pub id: i32,
///     pub name: String,
/// }
///
/// #[derive(Default, Debug, sqlx::FromRow, sqlxinsert::PqInsert)]
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
/// let pool = sqlx::SqlitePool::builder().build(&url).await.unwrap();
///
/// let car_skoda = CreateCar::new("Skoda");
/// let res: Car = car_skoda.insert::<Car>(&pool, "cars").await?;
/// # Ok(())
/// # }
/// ```
///
#[proc_macro_derive(PqInsert)]
pub fn derive_from_struct_psql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let field_name = fields.iter().map(|field| &field.ident);
    let field_name_values = fields.iter().map(|field| &field.ident);

    let field_length = field_name.len();
    // struct Car { id: i32, name: String }
    // -> ( $1,$2 )
    let values = dollar_values(field_length);

    // struct Car ...
    // -> Car
    let struct_name = &input.ident;

    // struct { id: i32, name: String }
    // -> ( id, name )
    let columns = format!(
        "{}",
        quote! {
            #( #field_name ),*
        }
    );

    TokenStream::from(quote! {
        impl #struct_name {
            fn insert_query(&self, table: &str) -> String
            {
                let sqlquery = format!("insert into {} ( {} ) values ( {} ) returning *", table, #columns, #values); // self.value_list()); //self.values );
                sqlquery
            }

            pub async fn insert<T>(&self, mut pool: &sqlx::Pool<sqlx::PgConnection>, table: &str) -> eyre::Result<T>
            where
                T: Send,
                T: for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow<'c>>
            {
                let sql = self.insert_query(table);

                let mut pool = pool;
                let res: T = sqlx::query_as::<_,T>(&sql)
                #(
                    .bind(&self.#field_name_values)//         let #field_name: #field_type = Default::default();
                )*
                    .fetch_one(&mut pool)
                    .await?;

                Ok(res)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_test() {
        let itr = 1..4;
        let res = itr
            .into_iter()
            .map(|s| format!("${}", s))
            .collect::<Vec<String>>()
            .join(",");

        assert_eq!(res, "$1,$2,$3");
    }

    #[test]
    fn dollar_value_tes() {
        let res = dollar_values(3);
        assert_eq!(res, "$1,$2,$3");
    }
}
