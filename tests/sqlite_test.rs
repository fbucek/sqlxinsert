// extern crate we're testing, same as any other code will do.
//extern crate gmacro;

// #[derive(Default, Debug, sqlx::FromRow)]
#[derive(Default, Debug, Clone, sqlx::FromRow, sqlxinsert::SqliteInsert)]
struct Car {
    pub id: i32,
    pub car_name: String,
}

#[tokio::test]
async fn test_macro_sqlite_insert() {
    let car = Car {
        id: 33,
        car_name: "Skoda".to_string(),
    };

    // bug: https://github.com/launchbadge/sqlx/issues/530
    let url = "sqlite::memory:";

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(url)
        .await
        .expect("Not possible to create pool");

    let create_table = "create table cars (
        id INTEGER PRIMARY KEY,
        car_name TEXT NOT NULL
    )";
    sqlx::query(create_table)
        .execute(&pool)
        .await
        .expect("Not possible to execute");

    let res = car.insert_raw(&pool, "cars").await.unwrap();

    assert_eq!(res.last_insert_rowid(), 33);

    let rows = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&pool)
        .await
        .expect("Not possible to fetch");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].car_name, "Skoda");
}

#[tokio::test]
async fn test_macro_sqlite_update() {
    let car = Car {
        id: 33,
        car_name: "Skoda".to_string(),
    };

    // bug: https://github.com/launchbadge/sqlx/issues/530
    let url = "sqlite::memory:";

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(url)
        .await
        .expect("Not possible to create pool");

    let create_table = "create table cars (
        id INTEGER PRIMARY KEY,
        car_name TEXT NOT NULL
    )";
    sqlx::query(create_table)
        .execute(&pool)
        .await
        .expect("Not possible to execute");

    let res = car.insert_raw(&pool, "cars").await.unwrap();

    assert_eq!(res.last_insert_rowid(), 33);

    let rows = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&pool)
        .await
        .expect("Not possible to fetch");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].car_name, "Skoda");

    let mut audi = car.clone();
    audi.car_name = "Audi".to_string();

    let sql = audi.update_query("cars");
    assert_eq!(sql, "update cars set car_name = $2 where id = $1");

    let _ = audi.update_raw(&pool, "cars").await.unwrap();

    let rows = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&pool)
        .await
        .expect("Not possible to fetch");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].car_name, "Audi");
}
