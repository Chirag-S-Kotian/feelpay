use sqlx::Row;

#[tokio::test]
#[ignore = "requires DATABASE_URL to real postgres and migrated schema"]
async fn db_connectivity_and_basic_query() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("db connect failed");

    let row = sqlx::query("SELECT 1 as one")
        .fetch_one(&pool)
        .await
        .expect("query failed");
    let one: i32 = row.get("one");
    assert_eq!(one, 1);
}

