use tree_sitter::Tree;

use crate::scope_tree::ScopeNode;

pub struct File {
    pub content: String,
    pub tree: Option<Tree>,
    pub scopes: Option<ScopeNode>,
}
