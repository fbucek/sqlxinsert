// #[derive(Default, Debug, sqlx::FromRow)]
#[derive(Default, Debug, sqlx::FromRow)]
struct Car {
    pub id: i32,
    pub car_name: String,
}

impl Car {
    pub async fn insert<T>(&self, pool: &sqlx::SqlitePool, table: &str) -> sqlx::Result<T>
    where
        T: Send,
        T: for<'c> sqlx::FromRow<'c, sqlx::sqlite::SqliteRow>,
        T: std::marker::Unpin,
    {
        let sql = self.insert_query(table);
        // let res = sqlx::query(&sql).execute(pool).await?;
        let res = sqlx::query(&sql).bind(&self.car_name).execute(pool).await?;

        let sql = format!(
            "select * from {} where id={}",
            table,
            res.last_insert_rowid()
        );
        let res: T = sqlx::query_as::<_, T>(&sql)
            // .bind(&self.)
            .fetch_one(pool)
            .await?;

        Ok(res)
    }
    fn insert_query(&self, table: &str) -> String {
        format!(
            "insert into {} ( id, car_name ) values ( '{}', '{}' )",
            table, self.id, self.car_name
        )
    }
}

#[tokio::test]
async fn test_macro_sqlite_insert_generic() {
    // let car = Car {
    //     car_id: 33,
    //     car_name: "Skoda".to_string(),
    // };

    let car = Car {
        id: 34,
        car_name: "Skoda".to_string(),
    };

    let url = "sqlite::memory:";

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(url)
        .await
        .expect("Not possible to create pool");

    // Reset database
    let drop_table = "DROP TABLE IF EXISTS cars";
    sqlx::query(drop_table).execute(&pool).await.unwrap();

    let create_table = "create table cars (
        id INTEGER PRIMARY KEY,
        car_name TEXT NOT NULL
    )";

    sqlx::query(create_table)
        .execute(&pool)
        .await
        .expect("Not possible to cretae table");

    let _car = car
        .insert::<Car>(&pool, "cars")
        .await
        .expect("Not possible to insert into dabase");

    let rows = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&pool)
        .await
        .expect("Not possible to fetch");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].car_name, "Skoda");
}
