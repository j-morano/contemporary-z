#[allow(dead_code)]#[derive(Clone)]
pub(crate) struct Directory {
    pub name: String,
    pub counter: i64,
    pub last_access: i64,
    pub score: f64,
    pub alias: String,
}
