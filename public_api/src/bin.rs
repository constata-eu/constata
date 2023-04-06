use public_api::{server, Site};

#[rocket::launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
  // It's ok to unwrap here as it will panic when the process launches, which helps us know and fix it right away.
  server(Site::from_stdin_password().await.unwrap())
}
