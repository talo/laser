use crate::{ColumnName, Derive, Filtered, PushPrql, Sorted, SortedBy, Sorting, Taken};

pub fn group<Query, Expr, Pipeline>(
    query: Query,
    expr: Expr,
    pipeline: Pipeline,
) -> Group<Query, Expr, Pipeline> {
    Group {
        query,
        expr,
        pipeline,
    }
}

pub struct Group<Query, Expr, Pipeline> {
    pub query: Query,
    pub expr: Expr,
    pub pipeline: Pipeline,
}

impl<Query, Expr, Pipeline> PushPrql for Group<Query, Expr, Pipeline>
where
    Query: PushPrql,
    Expr: PushPrql,
    Pipeline: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        self.query.push_to_driver(driver);
        driver.push("\ngroup ");
        self.expr.push_to_driver(driver);
        driver.push(" (");
        self.pipeline.push_to_driver(driver);
        driver.push(')');
    }
}

impl<Query, Expr, Pipeline> Group<Query, Expr, Pipeline> {
    pub fn filter<Filter>(self, filter: Filter) -> Filtered<Self, Filter> {
        Filtered {
            query: self,
            filter,
        }
    }

    pub fn sort<Sort>(self, sort: Sort) -> Sorted<Self, Sort> {
        Sorted { query: self, sort }
    }

    pub fn take(self, n: usize) -> Taken<Self> {
        Taken { query: self, n }
    }

    pub fn derive<Expr2>(self, name: &'static str, expr: Expr2) -> Derive<Self, Expr2> {
        Derive {
            query: self,
            derivations: vec![(ColumnName { name }, expr)],
        }
    }
}

impl<Query, Expr, Pipeline> SortedBy for Group<Query, Expr, Pipeline>
where
    Query: SortedBy,
{
    fn sorting(&self) -> impl Sorting {
        self.query.sorting()
    }
}
