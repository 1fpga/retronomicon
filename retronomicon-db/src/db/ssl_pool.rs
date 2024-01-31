use diesel::{ConnectionError, ConnectionResult};
use rocket::figment::Figment;
use rocket::futures::future::BoxFuture;
use rocket::futures::FutureExt;
use rocket::tokio;
use rocket_db_pools::diesel::AsyncPgConnection;
use rustls::Certificate;
use std::path::Path;

fn establish_connection<'a>(
    url: &'a str,
    additional_certs: &'a [Certificate],
) -> BoxFuture<'a, ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        // We first set up the way we want rustls to work.
        let rustls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_certs(additional_certs))
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(url, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                rocket::error!("Database connection error: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}

fn load_certificates_from_pem(path: impl AsRef<Path>) -> std::io::Result<Vec<rustls::Certificate>> {
    let file = std::fs::File::open(path.as_ref())?;
    let mut reader = std::io::BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader);
    certs
        .into_iter()
        .map(|r| Ok(rustls::Certificate(r?.to_vec())))
        .collect()
}

fn root_certs(additional_certs: &[Certificate]) -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs().expect("Certs not loadable!");
    let certs: Vec<_> = certs.into_iter().map(|cert| cert.0).collect();
    roots.add_parsable_certificates(&certs);

    roots.add_parsable_certificates(additional_certs);

    roots
}

/// A connection pool for the database, using rustls for SSL connections if necessary.
/// The default diesel-async (and by extension, rocket_db_pool::PgPool) does not support
/// SSL by default (it uses NoTls when connecting). This is not a problem locally but
/// when using the databases in production we want to be able to use a proper SSL connection.
pub struct Pool {
    url: String,
    /// A list of additional certificates to trust when connecting to the database.
    additional_certs: Vec<Certificate>,
}

#[rocket::async_trait]
impl rocket_db_pools::Pool for Pool {
    type Connection = AsyncPgConnection;

    type Error = ConnectionError;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let url = figment.extract_inner::<String>("url").unwrap();
        let certs: Vec<Certificate> = figment
            .extract_inner::<Vec<String>>("certs")
            .unwrap_or_default()
            .into_iter()
            .map(load_certificates_from_pem)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .flatten()
            .collect();

        // Establish the connection once (to check).
        establish_connection(&url, &certs).await?;
        Ok(Self {
            url,
            additional_certs: certs,
        })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        establish_connection(&self.url, &self.additional_certs).await
    }

    async fn close(&self) {}
}
