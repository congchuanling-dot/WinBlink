#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    Application,
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct SearchItem {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub item_type: ItemType,
}
