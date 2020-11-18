# Changelog

## 0.3.0-alpha.0 - 2020-11-18

- `SqliteInsert` `insert` method changed into `insert_raw` ( it does not return `Result<T>` but only `Result<sqlx::sqlite::SqliteDone>`
- Renamed typo `PqInsert` into correct `PgInsert`
- Not finished: generic method for `insert<T>`


## 0.2.2 - 2020-11-14

- chagned to `runtime-actix-rustls` from `tokio` because there is problem with sqlite under actix with tokio runtime.
    - [sqlx issue - Remove sqlx_rt::blocking!() and change runtime-actix](https://github.com/launchbadge/sqlx/issues/793)

## 0.2.1 - 2020-11-14

- Updated reamde.md

## 0.2.0 - 2020-11-14

- Updated to `sqlx 0.4`
