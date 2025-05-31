#[derive(Debug)]
pub struct PaginatedList<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> PaginatedList<T> {
    pub fn into_inner(self) -> Vec<T> {
        self.items
    }
}
