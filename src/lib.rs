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

            pub async fn insert_raw(&self, pool: &sqlx::SqlitePool, table: &str) -> sqlx::Result<sqlx::sqlite::SqliteQueryResult>
            {
                let sql = self.insert_query(table);
                Ok(sqlx::query(&sql)
                #(
                    .bind(&self.#field_name2)//         let #field_name: #field_type = Default::default();
                )*
                    .execute(pool)// (&mut conn)
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
/// # async fn main() -> sqlx::Result<()>{
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

            pub async fn insert<'e,E>(&self, executor: E, table: &str) -> sqlx::Result<()>
            where
                E: sqlx::Executor<'e,Database = sqlx::Postgres>
            {
                let sql = self.insert_query(table);

                // let mut pool = pool;
                 sqlx::query(&sql)
                #(
                    .bind(&self.#field_name_values)//         let #field_name: #field_type = Default::default();
                )*
                    .execute(executor)
                    .await?;

                Ok(())
            }
        }
    })
}

#[proc_macro_derive(PgUpdate)]
pub fn derive_update_from_struct_psql(input: TokenStream) -> TokenStream {
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
            pub fn update_query_string(&self, table: &str, where_field: &str) -> String
            {

                let fields_for_update = #columns.trim().split(", ").collect::<Vec<&str>>();
                 let where_field_tuple = fields_for_update.iter().enumerate().find(|(i,&item)| item == where_field).expect("Where field does not exist as field on struc");
                let where_clause = format!("WHERE {} = ${}", where_field_tuple.1, where_field_tuple.0 + 1);

                let update_set_values = fields_for_update.iter().enumerate().filter(|(i,&item)| item !=where_field).map(|(i,item)| format!("{} = ${}", item, i + 1))
                    .collect::<Vec<String>>().join(",");

                let sqlquery = format!("UPDATE {} SET {} {}  returning *", table, update_set_values, where_clause ); // self.value_list()); //self.values );
                sqlquery
            }

            pub async fn update<'e,E>(&self, executor: E, table: &str, where_field: &str) -> sqlx::Result<()>
            where
                E: sqlx::Executor<'e,Database = sqlx::Postgres>
            {
                let sql = self.update_query_string(table, where_field);

                // let mut pool = pool;
                 sqlx::query(&sql)
                #(
                    .bind(&self.#field_name_values)//         let #field_name: #field_type = Default::default();
                )*
                    .execute(executor)
                    .await?;

                Ok(())
            }

        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, std::cmp::PartialEq, sqlx::FromRow)]
    struct Car {
        pub id: i32,
        pub name: String,
    }

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
