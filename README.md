# sqlxinsert




Inserting using sqlx.

```rust
let query = r#"
insert into article 
    ( title, subtitle, url, text ) 
values 
    ( $1, $2, $3, $4 ) 
returning *
"#;

let result: Article = sqlx::query_as::<_, Article>(query)
    .bind(&article.title)
    .bind(&article.subtitle)
    .bind(&article.url)
    .bind(&article.text)
    .fetch_one(&mut conn)
    .await?;
```

Using derive macro

```rust
let res = car.insert::<Car>(&pool, "cars").await?
```

##### Example with different input and output structs.

```rust
#[tokio::main]
async fn main() -> eyre::Result<()>{
[derive(Default, Debug, std::cmp::PartialEq, sqlx::FromRow)]
struct Car {
    pub id: i32,
    pub name: String,
}
#[derive(Default, Debug, sqlx::FromRow, gmacro::PqInsert)]
struct CreateCar {
    pub name: String,
    pub color: Option<String>,
}
impl CreateCar {
    pub fn new<T: Into<String>>(name: T) -> Self {
        CreateCar {
            name: name.into(),
            color: None,
        }
    }
}

let url = "postgres://user:pass@localhost:5432/test_db";
let pool = sqlx::SqlitePool::builder().build(&url).await.unwrap();
let car_skoda = CreateCar::new("Skoda");
let res: Car = car_skoda.insert::<Car>(&pool, "cars").await?;
Ok(())
}
```

## Build

`make build`

## Test

Requirements: docker installed

`make all` will crete docker
