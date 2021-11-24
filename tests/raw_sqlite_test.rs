#[derive(Default, Debug, sqlx::FromRow)]
struct Car {
    pub car_id: i32,
    pub car_name: String,
}
#[derive(Default, Debug, sqlx::FromRow)]
struct CreateCar {
    pub car_name: String,
}

impl Car {
    pub async fn insert(
        &self,
        pool: &sqlx::SqlitePool,
        table: &str,
    ) -> eyre::Result<sqlx::sqlite::SqliteQueryResult> {
        let sql = self.insert_query(table);
        let res = sqlx::query(&sql).execute(pool).await?;
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
async fn test_macro_sqlite_insert_raw() {
    let car = Car {
        car_id: 33,
        car_name: "Skoda".to_string(),
    };

    // bug: https://github.com/launchbadge/sqlx/issues/530
    // let pool = sqlx::SqlitePool::connect("sqlite:memory:")
    //     .await
    //     .expect("Not possible to create pool");

    //let mut conn = pool.acquire().await.unwrap();

    let url = "sqlite:%3Amemory:";

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(url)
        .await
        .expect("Not possible to create pool");

    // let mut conn = sqlx::sqlite::SqliteConnectOptions::from_str("sqlite:memory:")
    //     .expect("Constructing from string")
    //     .connect()
    //     .await
    //     .expect("Not possible to create connection");

    // let mut conn = pool.acquire

    let create_table = "create table cars (
        car_id INTEGER PRIMARY KEY,
        car_name TEXT NOT NULL
    )";

    sqlx::query(create_table)
        .execute(&pool)
        .await
        .expect("Not possible to cretae table");

    car.insert(&pool, "cars")
        .await
        .expect("Not possible to insert into dabase");

    // let sql = car.insert_query("cars");

    // assert_eq!(sql, "insert into cars ( car_id, car_name ) values ( \'33\',\'Skoda\' )");

    // let res = sqlx::query(&sql)
    //         .execute(&mut conn)// (&mut conn)
    //         .await
    //         .unwrap();

    let rows = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&pool)
        .await
        .expect("Not possible to fetch");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].car_name, "Skoda");
}
