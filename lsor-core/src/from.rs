use crate::{
    column::ColumnName,
    derive::Derive,
    driver::{Driver, PushPrql},
    filter::Filtered,
    sort::Sorted,
    table::TableName,
    take::Taken,
    Group,
};

pub fn from(table: TableName) -> From {
    From { table }
}

pub struct From {
    pub table: TableName,
}

impl From {
    pub fn filter<Filter>(self, filter: Filter) -> Filtered<Self, Filter> {
        Filtered {
            query: self,
            filter,
        }
    }

    pub fn group<Expr, Pipeline>(
        self,
        expr: Expr,
        pipeline: Pipeline,
    ) -> Group<Self, Expr, Pipeline> {
        Group {
            query: self,
            expr,
            pipeline,
        }
    }

    pub fn sort<Sort>(self, sort: Sort) -> Sorted<Self, Sort> {
        Sorted { query: self, sort }
    }

    pub fn take(self, n: usize) -> Taken<Self> {
        Taken { query: self, n }
    }

    pub fn derive<Expr>(self, name: &'static str, expr: Expr) -> Derive<Self, Expr> {
        Derive {
            query: self,
            derivations: vec![(ColumnName { name }, expr)],
        }
    }
}

impl PushPrql for From {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push("from ");
        self.table.push_to_driver(driver);
    }
}

#[cfg(test)]
mod test {
    use crate::table::table;

    use super::*;

    #[test]
    fn test_from() {
        let mut driver = Driver::new();
        {
            from(table("users")).push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "SELECT * FROM users");
    }
}
