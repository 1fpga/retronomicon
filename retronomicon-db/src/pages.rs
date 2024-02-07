use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::sql_types::BigInt;
use retronomicon_dto as dto;
use rocket_db_pools::diesel::methods::LoadQuery;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use rocket_db_pools::Database;

pub trait Paginate: Sized {
    fn paginate(self, page: i64, per_page: i64) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: i64, per_page: i64) -> Paginated<Self> {
        Paginated {
            query: self,
            per_page,
            page,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    per_page: i64,
    page: i64,
}

impl<T> Paginated<T> {
    pub async fn load_and_count_total<'a, U>(
        self,
        conn: &rocket_db_pools::Connection<impl Database>,
    ) -> QueryResult<dto::Paginated<U>>
    where
        Self: RunQueryDsl<PgConnection> + LoadQuery<'a, PgConnection, (U, i64)> + 'a,
    {
        let results = self.load::<(U, i64)>(conn).await?;
        let total = results.first().map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();

        Ok(dto::Paginated::new(
            total as u64,
            self.page as u64,
            self.per_page as u64,
            records,
        ))
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&((self.page - 1) * self.per_page))?;
        Ok(())
    }
}
