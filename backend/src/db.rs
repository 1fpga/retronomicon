use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket_db_pools::{Connection, Database};

pub mod ssl_pool;

#[derive(Database)]
#[database("retronomicon_db")]
pub struct RetronomiconDb(ssl_pool::Pool);

pub type Db = Connection<RetronomiconDb>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn run_migrations() {
    use diesel::Connection;

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let mut connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");
}
