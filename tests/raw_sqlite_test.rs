// extern crate we're testing, same as any other code will do.
//extern crate gmacro;

use sqlx::prelude::SqliteQueryAs;
use sqlx::Connect;
use sqlx::Executor;

// #[derive(Default, Debug, sqlx::FromRow)]
#[derive(Default, Debug, sqlx::FromRow)]
struct Car {
    pub car_id: i32,
    pub car_name: String,
}

impl Car {
    pub async fn insert<T>(&self, mut conn: &mut T, table: &str) -> eyre::Result<u64>
    where
        T: sqlx::Connect,
        T: sqlx::Executor,
        // T: std::marker::Copy,
    {
        let sql = self.insert_query(table);
        let res = sqlx::query(&sql).execute(&mut conn).await.unwrap();
        Ok(res)
    }
    fn insert_query(&self, table: &str) -> String {
        format!(
            "insert into {} ( car_id, car_name) values ( '{}', '{}' )",
            table, self.car_id, self.car_name
        )
    }
}

#[tokio::test]
async fn test_gmacro() {
    let car = Car {
        car_id: 33,
        car_name: "Skoda".to_string(),
    };

    // bug: https://github.com/launchbadge/sqlx/issues/530
    let mut conn = sqlx::SqliteConnection::connect("sqlite:%3Amemory:")
        .await
        .unwrap();
    // .expect("Not possible to create connection");

    let create_table = "create table cars (
        car_id INTEGER PRIMARY KEY,
        car_name TEXT NOT NULL
    )";

    conn.execute(sqlx::query(create_table))
        .await
        .expect("Not possible to execute");

    car.insert(&mut conn, "cars")
        .await
        .expect("Not possible to insert into dabase");

    // let sql = car.insert_query("cars");

    // assert_eq!(sql, "insert into cars ( car_id, car_name ) values ( \'33\',\'Skoda\' )");

    // let res = sqlx::query(&sql)
    //         .execute(&mut conn)// (&mut conn)
    //         .await
    //         .unwrap();

    let rows = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&mut conn)
        .await
        .expect("Not possible to fetch");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].car_name, "Skoda");
}
