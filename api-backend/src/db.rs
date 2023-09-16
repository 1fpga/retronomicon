use rocket_db_pools::diesel::PgPool;
use rocket_db_pools::{Connection, Database};

#[derive(Database)]
#[database("retronomicon_db")]
pub struct RetronomiconDb(PgPool);

pub type Db = Connection<RetronomiconDb>;
