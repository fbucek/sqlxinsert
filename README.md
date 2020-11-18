# sqlxinsert

Warning: used in private projects

Derive macro for inserting struct into sql.

Instead of manually creating insert query and binding all attributes:

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

Using derive macro if simplifies to this: 

```rust
let res = car.insert::<Car>(&pool, "cars").await?
```

##### Example with different input and output structs.

```rust
#[derive(Default, Debug, std::cmp::PartialEq, sqlx::FromRow)]
struct Car {
    pub id: i32,
    pub name: String,
    pub color: Option<String>
}
#[derive(Default, Debug, sqlx::FromRow, sqlxinsert::PgInsert)]
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

#[tokio::main]
async fn main() -> eyre::Result<()>{
    let url = "postgres://user:pass@localhost:5432/test_db";
    let pool = sqlx::postgres::PgPoolOptions::new().connect(&url).await.unwrap();
    let car_skoda = CreateCar::new("Skoda");
    let res: Car = car_skoda.insert::<Car>(pool, "cars").await?;
    Ok(())
}
```

## Build

`make build`

## Test

Requirements: `docker` installed

`make testall`
