use constata_lib::models::Site;

#[tokio::main]
async fn main() {
  let result = sqlx::migrate!("db/migrations")
    .run(&Site::from_stdin_password().await.unwrap().db.pool)
    .await;

  if let Err(e) = result {
    println!("Migration error {:?}", e);
  }
}
