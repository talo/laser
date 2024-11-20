use async_graphql::Enum;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    cursor::{Cursor, Iterable},
    driver::{Driver, PushPrql},
    take::Taken,
    ColumnName, Derive, Empty,
};

pub fn sort<By>(by: By) -> Sorted<Empty, By> {
    Sorted {
        query: Empty,
        sort: by,
    }
}

/// The implementation of `PushPrql` must only push the expression that is being
/// ordered by. It must not push the order itself.
pub trait Sorting: PushPrql {
    fn order(&self) -> Order;
    fn flip(&self) -> impl Sorting;
    fn push_to_driver_with_order(&self, driver: &mut Driver);
}

impl<T> Sorting for &T
where
    T: Sorting,
{
    fn order(&self) -> Order {
        (*self).order()
    }

    fn flip(&self) -> impl Sorting {
        (*self).flip()
    }

    fn push_to_driver_with_order(&self, driver: &mut Driver) {
        (*self).push_to_driver_with_order(driver)
    }
}

pub trait SortedBy {
    fn sorting(&self) -> impl Sorting;
}

impl<T> SortedBy for &T
where
    T: SortedBy,
{
    fn sorting(&self) -> impl Sorting {
        (*self).sorting()
    }
}

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    pub fn flip(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }

    pub fn is_asc(&self) -> bool {
        matches!(self, Self::Asc)
    }

    pub fn is_desc(&self) -> bool {
        matches!(self, Self::Desc)
    }
}

impl PushPrql for Order {
    fn push_to_driver(&self, driver: &mut Driver) {
        if let Self::Desc = self {
            driver.push('-');
        }
    }
}

pub struct Sort<By> {
    pub order: Order,
    pub by: By,
}

impl<By> Sorting for Sort<By>
where
    By: PushPrql,
{
    fn order(&self) -> Order {
        self.order
    }

    fn flip(&self) -> impl Sorting {
        Sort {
            order: self.order.flip(),
            by: &self.by,
        }
    }

    fn push_to_driver_with_order(&self, driver: &mut Driver) {
        self.order.push_to_driver(driver);
        self.by.push_to_driver(driver);
    }
}

impl<By> PushPrql for Sort<By>
where
    By: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        self.by.push_to_driver(driver);
    }
}

pub struct Sorted<Query, Sort> {
    pub query: Query,
    pub sort: Sort,
}

impl<Query, Sort> Sorted<Query, Sort> {
    pub fn sort<Sort2>(&self, sort: Sort2) -> Sorted<&Self, Sort2> {
        Sorted { query: self, sort }
    }

    pub fn take(&self, n: usize) -> Taken<&Self> {
        Taken { query: self, n }
    }

    pub fn derive<Expr>(&self, name: &'static str, expr: Expr) -> Derive<&Self, Expr> {
        Derive {
            query: self,
            derivations: vec![(ColumnName { name }, expr)],
        }
    }
}

impl<Query, Sort> SortedBy for Sorted<Query, Sort>
where
    Sort: Sorting,
{
    fn sorting(&self) -> impl Sorting {
        &self.sort
    }
}

impl<Query, Sort> PushPrql for Sorted<Query, Sort>
where
    Query: PushPrql,
    Sort: Sorting,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        self.query.push_to_driver(driver);
        driver.push("\nsort { ");
        self.sort.push_to_driver_with_order(driver);
        driver.push(" }");
    }
}

pub trait SortBy<By> {
    fn sort_by(by: By) -> Sort<By>;
}

impl<By> SortBy<By> for i32
where
    By: PushPrql,
{
    fn sort_by(by: By) -> Sort<By> {
        Sort {
            order: Order::Asc,
            by,
        }
    }
}

impl<By> SortBy<By> for u64
where
    By: PushPrql,
{
    fn sort_by(by: By) -> Sort<By> {
        Sort {
            order: Order::Asc,
            by,
        }
    }
}

impl<By> SortBy<By> for i64
where
    By: PushPrql,
{
    fn sort_by(by: By) -> Sort<By> {
        Sort {
            order: Order::Asc,
            by,
        }
    }
}

impl<By> SortBy<By> for f32
where
    By: PushPrql,
{
    fn sort_by(by: By) -> Sort<By> {
        Sort {
            order: Order::Asc,
            by,
        }
    }
}

impl<By> SortBy<By> for f64
where
    By: PushPrql,
{
    fn sort_by(by: By) -> Sort<By> {
        Sort {
            order: Order::Asc,
            by,
        }
    }
}

impl<By> SortBy<By> for String
where
    By: PushPrql,
{
    fn sort_by(by: By) -> Sort<By> {
        Sort {
            order: Order::Asc,
            by,
        }
    }
}

macro_rules! impl_sortable {
    ($t:ty, $i:ident, $c:expr) => {
        impl Sortable for $t {
            type Sort = $i;
        }

        impl Sortable for Option<$t> {
            type Sort = $i;
        }

        #[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
        #[graphql(rename_items = "snake_case")]
        pub enum $i {
            Asc,
            Desc,
        }

        impl Iterable for $i {
            fn cursor(&self) -> Cursor {
                $c
            }
        }

        impl $i {
            pub fn order(&self) -> Order {
                match self {
                    Self::Asc => Order::Desc,
                    Self::Desc => Order::Asc,
                }
            }

            pub fn flip_as_self(&self) -> Self {
                match self {
                    Self::Asc => Self::Desc,
                    Self::Desc => Self::Asc,
                }
            }

            pub fn push_to_driver_with_lhs(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
                lhs.push_to_driver(driver);
            }

            pub fn push_to_driver_with_order_with_lhs(
                &self,
                lhs: &dyn PushPrql,
                driver: &mut Driver,
            ) {
                match self {
                    Self::Asc => {
                        lhs.push_to_driver(driver);
                    }
                    Self::Desc => {
                        driver.push('-');
                        lhs.push_to_driver(driver);
                    }
                }
            }
        }
    };
}

pub trait Sortable {
    type Sort;
}

impl_sortable!(i32, I32Sort, Cursor::I32);
impl_sortable!(i64, I64Sort, Cursor::I64);
impl_sortable!(u32, U32Sort, Cursor::I32);
impl_sortable!(u64, U64Sort, Cursor::I64);
impl_sortable!(f32, F32Sort, Cursor::F32);
impl_sortable!(f64, F64Sort, Cursor::F64);
impl_sortable!(String, StringSort, Cursor::String);
impl_sortable!(Uuid, UuidSort, Cursor::Uuid);
impl_sortable!(DateTime<Utc>, DateTimeSort, Cursor::DateTime);

#[cfg(test)]
mod test {
    use crate::{
        column::{col, json},
        cond::gt,
        from::from,
        table::table,
    };

    use super::*;

    #[test]
    fn test_sort() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .sort(col("age").asc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "SELECT * FROM users ORDER BY age");
    }

    #[test]
    fn test_sort_desc() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .sort(col("age").desc())
                .push_to_driver(&mut driver);
        }
        dbg!(driver.prql());
        assert_eq!(driver.sql(), "SELECT * FROM users ORDER BY age DESC");
    }

    #[test]
    fn test_filter_sort() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .filter(gt(col("age"), 18))
                .sort(col("age").asc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(
            driver.sql(),
            "SELECT * FROM users WHERE age > $1 ORDER BY age"
        );
    }

    #[test]
    fn test_take_sort() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .take(10)
                .sort(col("age").asc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(
            driver.sql(),
            "WITH table_0 AS (SELECT * FROM users LIMIT 10) SELECT * FROM table_0 ORDER BY age"
        );
    }

    #[test]
    fn test_sort_json() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .sort(json(col("info")).get("age").asc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "WITH table_0 AS (SELECT *, info->'age' AS _expr_0 FROM users) SELECT * FROM table_0 ORDER BY _expr_0");

        let mut driver = Driver::new();
        {
            from(table("users"))
                .sort(json(col("info")).get("age").desc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "WITH table_0 AS (SELECT *, info->'age' AS _expr_0 FROM users) SELECT * FROM table_0 ORDER BY _expr_0 DESC");
    }
}
