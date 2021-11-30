
#[derive(Debug)]
pub(crate) struct Directory {
    pub(crate) name: String,
    pub(crate) counter: i64,
    pub(crate) last_access: i64,
    pub(crate) score: f64,
    pub(crate) alias: String,
}
