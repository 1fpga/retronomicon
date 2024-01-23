use backend::routes::v1;
use backend::{db, routes};
use cucumber::given;
use retronomicon_dto as dto;
use rocket::local::asynchronous::Client;
use rocket_db_pools::Database;

#[derive(cucumber::World, Debug)]
#[world(init = World::new)]
struct World {
    s3_url: String,
    database_url: String,
    pub client: Client,
}

impl World {
    async fn new() -> Self {
        let rocket = rocket::build()
            .mount("/api", routes::routes())
            .mount("/api/v1", v1::routes())
            .attach(db::RetronomiconDb::init())
            .launch()
            .await
            .expect("Rocket failed to launch");
        let client = Client::tracked(rocket).await.expect("Rocket client failed");

        Self {
            s3_url: std::env::var("AWS_REGION").unwrap_or("http://localhost:9000".to_string()),
            database_url: std::env::var("DATABASE_URL").unwrap_or(
                "postgres://local_user:mysecretpassword@localhost:5432/local_retronomicon"
                    .to_string(),
            ),
            client,
        }
    }
}

// #[given(expr = "a user named {user}")]
// async fn given_a_user(w: &mut World, user: String) {
//     w.client.
// }
