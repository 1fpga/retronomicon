use serde::{Deserialize, Serialize};

/// A paginated response, containing serialized items that are in
/// a page format.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct Paginated<T> {
    /// The total number of items in the database at the time of the request.
    pub total: i64,

    /// The page requested. This may be different than the actual requested
    /// page if the requested page is out of bounds.
    pub page: i64,

    /// The number of item per page. This should correspond to the number of
    /// items in the `items` field. This may be different than the requested
    /// number of items in the `items` field if the number of items requested
    /// is out of bounds or if the page has less items than requested.
    pub per_page: i64,

    /// The number of pages total.
    pub page_count: i64,

    /// The items requested.
    pub items: Vec<T>,
}

impl<T> Paginated<T> {
    pub fn new(page: i64, per_page: i64, total: i64, items: Vec<T>) -> Self {
        let page_count = (total + per_page - 1) / per_page;
        Self {
            total,
            page,
            per_page,
            page_count,
            items,
        }
    }

    pub fn map_items<'a, T2: Serialize + Deserialize<'a>>(
        self,
        f: impl Fn(T) -> T2,
    ) -> Paginated<T2> {
        Paginated {
            total: self.total,
            page: self.page,
            per_page: self.per_page,
            page_count: self.page_count,
            items: self.items.into_iter().map(f).collect(),
        }
    }

    pub fn first(&self) -> Option<&T> {
        self.items.first()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.items.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn last(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
}
