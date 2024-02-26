use diesel::{pg::Pg, prelude::*, query_builder::*, sql_types::BigInt};
use rocket_db_pools::diesel::methods::LoadQuery;
use rocket_db_pools::diesel::{AsyncPgConnection, RunQueryDsl};

pub const DEFAULT_PER_PAGE: i64 = 10;

pub trait Paginate: Sized {
    fn paginate(self, page: Option<i64>) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: Option<i64>) -> Paginated<Self> {
        let page = page.unwrap_or(1);

        Paginated {
            query: self,
            per_page: DEFAULT_PER_PAGE,
            page,
            offset: page * DEFAULT_PER_PAGE,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    page: i64,
    offset: i64,
    per_page: i64,
}

impl<T> Paginated<T> {
    pub fn per_page(self, per_page: Option<i64>) -> Self {
        let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);

        Paginated {
            per_page,
            offset: self.page * per_page,
            ..self
        }
    }

    pub async fn load_and_count_total<'a, U>(
        self,
        conn: &'a mut AsyncPgConnection,
    ) -> QueryResult<(Vec<U>, i64)>
    where
        Self: LoadQuery<'a, AsyncPgConnection, (U, i64)>,
        U: Send,
        T: 'a,
    {
        // Ignore those linting errors. `get(0)` cannot be replaced with `first()`.
        #![allow(clippy::get_first)]

        let results = self.load::<(U, i64)>(conn).await?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        Ok((records, total))
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
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}

pub struct PaginationOptions {
    pub per_page: i64,
    pub page: i64,
}
