use diesel::{ConnectionError, ConnectionResult, PgConnection};
use rocket_db_pools::diesel::{AsyncPgConnection, PgPool};
use rocket_db_pools::{Connection, Database};

#[derive(Database)]
#[database("retronomicon_db")]
pub struct RetronomiconDb(MyPool);

pub type Db = Connection<RetronomiconDb>;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket::futures::future::BoxFuture;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

fn establish_connection(
    config: &str,
) -> BoxFuture<ConnectionResult<rocket_db_pools::diesel::pg::AsyncPgConnection>> {
    let fut = async {
        // We first set up the way we want rustls to work.
        let rustls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_certs())
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Database connection: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}

fn load_certificates_from_pem(path: &str) -> std::io::Result<Vec<rustls::Certificate>> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader);

    certs
        .into_iter()
        .map(|r| Ok(rustls::Certificate(r?.to_vec())))
        .collect()
}

fn root_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs().expect("Certs not loadable!");
    let certs: Vec<_> = certs.into_iter().map(|cert| cert.0).collect();
    roots.add_parsable_certificates(&certs);

    // Add the digitalocean root certificate.
    // let certs = load_certificates_from_pem("").unwrap();

    let mut reader = std::io::BufReader::new(std::io::Cursor::new(include_bytes!(
        "../../digitalocean.crt"
    )));
    let certs = rustls_pemfile::certs(&mut reader);
    let certs: Vec<_> = certs
        .into_iter()
        .map(|r| r.map(|x| rustls::Certificate(x.to_vec())))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    roots.add_parsable_certificates(&certs);

    roots
}

use rocket::figment::Figment;
use rocket::futures::FutureExt;
use rocket::tokio;
use rocket_db_pools::Pool;

pub struct MyPool(String);
#[rocket::async_trait]
impl Pool for MyPool {
    type Connection = AsyncPgConnection;

    type Error = ConnectionError;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config = figment.extract_inner::<String>("url").unwrap();

        // Establish the connection once (to check).
        establish_connection(&config).await?;
        Ok(Self(config))
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        establish_connection(&self.0).await
    }

    async fn close(&self) {}
}

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
