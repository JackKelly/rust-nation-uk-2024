use anyhow::Result;
use axum::routing::{get, post};
use axum::Extension;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

#[derive(Debug, FromRow, Serialize, Deserialize)]
struct BlogPost {
    id: i32,
    date: String,
    title: String,
    body: String,
    author: String,
}

async fn get_connection_pool(url: &str) -> Result<sqlx::SqlitePool> {
    let connection_poll = sqlx::SqlitePool::connect(url).await?;
    Ok(connection_poll)
}

async fn run_migrations(pool: sqlx::SqlitePool) -> Result<()> {
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}

async fn get_blog_post(pool: sqlx::SqlitePool, id: i32) -> Result<BlogPost> {
    let post = sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    Ok(post)
}

async fn get_blog_posts(pool: sqlx::SqlitePool) -> Result<Vec<BlogPost>> {
    let posts = sqlx::query_as::<_, BlogPost>("SELECT * FROM blog_posts")
        .fetch_all(&pool)
        .await?;
    Ok(posts)
}

async fn add_blog_post(
    pool: sqlx::SqlitePool,
    date: String,
    title: String,
    body: String,
    author: String,
) -> Result<i32> {
    let id = sqlx::query("INSERT INTO blog_posts (date, title, body, author) VALUES (?, ?, ?, ?); SELECT last_insert_rowid();")
        .bind(date)
        .bind(title)
        .bind(body)
        .bind(author)
        .fetch_one(&pool)
        .await?
        .get(0);
    Ok(id)
}

async fn update_blog_post(
    pool: sqlx::SqlitePool,
    id: i32,
    date: String,
    title: String,
    body: String,
    author: String,
) -> Result<()> {
    sqlx::query("UPDATE blog_posts SET date = ?, title = ?, body = ?, author = ? WHERE id = ?")
        .bind(date)
        .bind(title)
        .bind(body)
        .bind(author)
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

async fn say_hello() -> &'static str {
    "Hello, World!"
}

async fn get_blog_posts_handler(
    Extension(pool): Extension<sqlx::SqlitePool>,
) -> axum::Json<Vec<BlogPost>> {
    let posts = get_blog_posts(pool).await.unwrap();
    axum::Json(posts)
}

async fn get_blog_post_handler(
    Extension(pool): Extension<sqlx::SqlitePool>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> axum::Json<BlogPost> {
    let post = get_blog_post(pool, id).await.unwrap();
    axum::Json(post)
}

async fn update_blog_post_handler(
    Extension(pool): Extension<sqlx::SqlitePool>,
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Json(post): axum::extract::Json<BlogPost>,
) -> axum::Json<()> {
    update_blog_post(pool, id, post.date, post.title, post.body, post.author)
        .await
        .unwrap();
    axum::Json(())
}

async fn delete_blog_post_handler(
    Extension(pool): Extension<sqlx::SqlitePool>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> axum::Json<()> {
    delete_blog_post(pool, id).await.unwrap();
    axum::Json(())
}

async fn delete_blog_post(pool: sqlx::SqlitePool, id: i32) -> Result<()> {
    sqlx::query("DELETE FROM blog_posts WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Read the .env file and apply it
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    println!("Connecting to: {database_url}");

    // Setup the database
    let pool = get_connection_pool(&database_url).await?;
    println!("Running migrations!");
    run_migrations(pool.clone()).await?;

    println!("{:?}", get_blog_posts(pool.clone()).await?);

    println!("{:?}", get_blog_post(pool.clone(), 1).await?);

    // let mut post = BlogPost {
    //     id: 0, // This will be overwritten.
    //     date: "2022-01-01".to_string(),
    //     title: "Foo".to_string(),
    //     body: "Bar".to_string(),
    //     author: "Jack".to_string(),
    // };
    // let new_id = add_blog_post(pool.clone(), &mut post).await?;
    // println!(
    //     "New blog post: {:?}",
    //     get_blog_post(pool.clone(), new_id).await?
    // );

    // TCP Listener
    let listen_address = std::env::var("LISTEN_ADDRESS")?;
    println!("Listening on: {listen_address}");
    let listener = tokio::net::TcpListener::bind(&listen_address).await?;

    // Build Axum Router and run it
    let app = axum::Router::new()
        .route("/hello", get(say_hello))
        .route("/", get(get_blog_posts_handler))
        .route("/:id", get(get_blog_post_handler))
        .route("/add", post(get_blog_post_handler))
        .route("/delete/:id", post(delete_blog_post_handler))
        .layer(Extension(pool.clone()));
    axum::serve(listener, app).await?;

    Ok(())
}
