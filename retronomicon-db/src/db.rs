use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket_db_pools::{Connection, Database, Initializer};
use tracing::info;

pub mod ssl_pool;

#[derive(Database)]
#[database("retronomicon_db")]
pub struct RetronomiconDbPool(ssl_pool::Pool);

impl RetronomiconDbPool {
    pub fn init() -> Initializer<Self> {
        info!("Initializing database");
        Database::init()
    }
}

pub type Db = Connection<RetronomiconDbPool>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn run_migrations(database_url: &str) {
    use diesel::Connection;

    let mut connection = PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    let all_migrations = connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");

    for migration in all_migrations {
        info!("Migration: {}", migration);
    }
}
