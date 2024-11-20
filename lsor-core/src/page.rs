use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use crate::{
    column::col,
    cond::{and, gt, lt},
    cursor::Cursor,
    derive_from,
    driver::PushPrql,
    either::if_then_else,
    expr::{case, count, sum, when},
    filter::Filtered,
    sort::SortedBy,
    var::{one, zero},
};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    SimpleObject,
)]
#[graphql(rename_fields = "snake_case")]
pub struct TotalCount {
    pub total_count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Pagination {
    pub cursor: Cursor,
    pub after: Option<String>,
    pub before: Option<String>,
    pub first: usize,
    pub last: usize,
}

pub fn select_page_info<Query>(
    query: Query,
    cursor: Cursor,
    start: String,
    end: String,
) -> SelectPageInfo<Query> {
    SelectPageInfo {
        query,
        cursor,
        start,
        end,
    }
}

pub fn select_page_items<Query>(query: Query, pagination: Pagination) -> SelectPageItems<Query> {
    SelectPageItems { query, pagination }
}

pub struct SelectPageInfo<Query> {
    pub query: Query,
    pub cursor: Cursor,
    pub start: String,
    pub end: String,
}

impl<Query> PushPrql for SelectPageInfo<Query>
where
    Query: SortedBy + PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        use super::sort::Sorting;

        let sorting = self.query.sorting();
        let order = sorting.order();
        let start = self.cursor.decode(&self.start);
        let end = self.cursor.decode(&self.end);

        derive_from(
            &self.query,
            vec![
                (col("total_count"), &count() as &dyn PushPrql),
                (
                    col("has_prev_page"),
                    &gt(
                        sum(case([when(if_then_else(
                            order.is_asc(),
                            || lt(&sorting, &start),
                            || gt(&sorting, &start),
                        ))
                        .then(one())])
                        .otherwise(zero())),
                        zero(),
                    ) as &dyn PushPrql,
                ),
                (
                    col("has_next_page"),
                    &gt(
                        sum(case([when(if_then_else(
                            order.is_asc(),
                            || gt(&sorting, &end),
                            || lt(&sorting, &end),
                        ))
                        .then(one())])
                        .otherwise(zero())),
                        zero(),
                    ) as &dyn PushPrql,
                ),
            ],
        )
        .push_to_driver(driver);
    }
}

pub struct SelectPageItems<Query> {
    pub query: Query,
    pub pagination: Pagination,
}

impl<Query> PushPrql for SelectPageItems<Query>
where
    Query: SortedBy + PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        use super::sort::Sorting;

        let sorting = self.query.sorting();
        let order = sorting.order();

        let after = self
            .pagination
            .after
            .as_ref()
            .map(|v| self.pagination.cursor.decode(v))
            .unwrap_or(self.pagination.cursor.min());

        let before = self
            .pagination
            .before
            .as_ref()
            .map(|v| self.pagination.cursor.decode(v))
            .unwrap_or(self.pagination.cursor.max());

        Filtered {
            query: &self.query,
            filter: and(
                if_then_else(
                    order.is_asc(),
                    || gt(&sorting, &after),
                    || lt(&sorting, &after),
                ),
                if_then_else(
                    order.is_asc(),
                    || lt(&sorting, &before),
                    || gt(&sorting, &before),
                ),
            ),
        }
        .take(self.pagination.first)
        .sort(sorting.flip())
        .take(self.pagination.last)
        .sort(&sorting)
        .push_to_driver(driver);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{driver::Driver, from::from, sort, table::table, Empty};

    #[test]
    fn test_select_page_info() {
        let mut driver = Driver::new();
        {
            let query = from(table("page")).sort(col("created_at").asc());
            let cursor = Cursor::String;
            let start = "start".to_string();
            let end = "end".to_string();
            let select_page_info = SelectPageInfo {
                query,
                cursor,
                start,
                end,
            };
            select_page_info.push_to_driver(&mut driver);
        }
        assert_eq!(
            driver.sql(),
            "SELECT *, COUNT(*) OVER (ORDER BY created_at ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS total_count, SUM(CASE WHEN created_at < $1 THEN 1 ELSE 0 END) OVER (ORDER BY created_at ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) > 0 AS has_prev_page, SUM(CASE WHEN created_at > $2 THEN 1 ELSE 0 END) OVER (ORDER BY created_at ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) > 0 AS has_next_page FROM page ORDER BY created_at"
        );
    }

    #[test]
    fn test_select_page_items() {
        let mut driver = Driver::new();
        {
            let query = from(table("page"))
                .derive("cursor", col("created_at"))
                .sort(col("cursor").desc());
            let cursor = Cursor::String;
            let after = Some("after".to_string());
            let before = Some("before".to_string());
            let select_page_info = SelectPageItems {
                query,
                pagination: Pagination {
                    cursor,
                    after,
                    before,
                    first: 10,
                    last: 5,
                },
            };
            select_page_info.push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "WITH table_2 AS (SELECT *, created_at AS cursor FROM page), table_1 AS (SELECT * FROM table_2 WHERE cursor < $1 AND cursor > $2 ORDER BY cursor DESC LIMIT 10), table_0 AS (SELECT * FROM table_1 ORDER BY cursor LIMIT 5) SELECT * FROM table_0 ORDER BY cursor DESC");
    }

    #[test]
    fn test_select_distinct_page_items() {
        let mut driver = Driver::new();
        {
            let sort_by_name = sort(col("name").desc());
            let sort_by_name_then_created_at = sort_by_name.sort(col("created_at").desc());
            let query = from(table("page"))
                .group(col("name"), sort_by_name_then_created_at.take(1))
                .sort(col("name").desc());
            let cursor = Cursor::String;
            let after = Some("after".to_string());
            let before = Some("before".to_string());
            let select_page_info = SelectPageItems {
                query,
                pagination: Pagination {
                    cursor,
                    after,
                    before,
                    first: 10,
                    last: 5,
                },
            };
            select_page_info.push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "WITH table_2 AS (SELECT DISTINCT ON (name) * FROM page ORDER BY name, created_at DESC), table_1 AS (SELECT * FROM table_2 WHERE name < $1 AND name > $2 ORDER BY name DESC LIMIT 10), table_0 AS (SELECT * FROM table_1 ORDER BY name LIMIT 5) SELECT * FROM table_0 ORDER BY name DESC");
    }

    #[test]
    fn test_select_page_items_with_dangling_cursor() {
        let mut driver = Driver::new();
        {
            let query = from(table("page")).sort(col("created_at").desc());
            let query = query.derive("cursor", col("created_at"));
            let cursor = Cursor::String;
            let after = Some("after".to_string());
            let before = Some("before".to_string());
            let select_page_info = SelectPageItems {
                query,
                pagination: Pagination {
                    cursor,
                    after,
                    before,
                    first: 10,
                    last: 5,
                },
            };
            select_page_info.push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "WITH table_2 AS (SELECT *, created_at AS cursor FROM page), table_1 AS (SELECT * FROM table_2 WHERE created_at < $1 AND created_at > $2 ORDER BY created_at DESC LIMIT 10), table_0 AS (SELECT * FROM table_1 ORDER BY created_at LIMIT 5) SELECT * FROM table_0 ORDER BY created_at DESC");
    }
}
