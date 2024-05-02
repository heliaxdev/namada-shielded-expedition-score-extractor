use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::sql_types::{BigInt, Bool};

pub trait CountInnerDsl: Sized {
    fn count_inner(self) -> CountInner<Self>;
}

impl<T> CountInnerDsl for T {
    fn count_inner(self) -> CountInner<Self> {
        CountInner { query: self }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct CountInner<T> {
    query: T,
}

impl<T: Query> Query for CountInner<T> {
    type SqlType = BigInt;
}

impl<T> RunQueryDsl<PgConnection> for CountInner<T> {}

impl<T> QueryFragment<Pg> for CountInner<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT COUNT(*) as n FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") as count_inner_dsl");
        Ok(())
    }
}

impl<T> CountInner<T> {
    pub fn get<'a>(self, conn: &mut PgConnection) -> QueryResult<i64>
    where
        Self: LoadQuery<'a, PgConnection, i64>,
    {
        self.load::<i64>(conn)
            .map(|result| result.first().copied().unwrap_or_default())
    }
}

pub trait ExistsInnerDsl: Sized {
    fn exists_inner(self) -> ExistsInner<Self>;
}

impl<T> ExistsInnerDsl for T {
    fn exists_inner(self) -> ExistsInner<Self> {
        ExistsInner { query: self }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct ExistsInner<T> {
    query: T,
}

impl<T: Query> Query for ExistsInner<T> {
    type SqlType = Bool;
}

impl<T> RunQueryDsl<PgConnection> for ExistsInner<T> {}

impl<T> QueryFragment<Pg> for ExistsInner<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT EXISTS(");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") AS exists_inner_dsl");
        Ok(())
    }
}

impl<T> ExistsInner<T> {
    pub fn get<'a>(self, conn: &mut PgConnection) -> QueryResult<bool>
    where
        Self: LoadQuery<'a, PgConnection, bool>,
    {
        self.load::<bool>(conn)
            .map(|result| result.first().copied().unwrap_or_default())
    }
}
