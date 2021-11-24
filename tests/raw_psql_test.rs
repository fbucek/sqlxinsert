// #[derive(Default, Debug, sqlx::FromRow)]
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
        pool: &sqlx::PgPool,
        table: &str,
    ) -> eyre::Result<sqlx::postgres::PgQueryResult> {
        let sql = self.insert_query(table);
        let res = sqlx::query(&sql).execute(pool).await.unwrap();
        Ok(res)
    }
    fn insert_query(&self, table: &str) -> String {
        format!(
            "insert into {} ( car_id, car_name) values ( '{}', '{}' ) returning *",
            table, self.car_id, self.car_name
        )
    }
}

#[tokio::test]
async fn test_macro_psql_insert() {
    let car = Car {
        car_id: 33,
        car_name: "Skoda".to_string(),
    };

    let url = "postgres://user:pass@localhost:5444/test_db";

    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(30))
        .connect(url)
        .await
        .expect("Not possible to create pool");

    // Reset database
    let drop_table = "DROP TABLE IF EXISTS cars";
    sqlx::query(drop_table).execute(&pool).await.unwrap();

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

    let rows = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&pool)
        .await
        .expect("Not possible to fetch");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].car_name, "Skoda");
}
