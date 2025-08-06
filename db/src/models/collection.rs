
pub struct Column {
    pub name: String,
    pub is_primary_key: bool,
}

pub struct Collection {
    pub name: String,
    pub columns: Vec<Column>
}
