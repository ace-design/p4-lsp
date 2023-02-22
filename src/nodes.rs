use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tree_sitter_p4::NODE_TYPES as NODE_TYPES_JSON;

#[derive(Serialize, Deserialize)]
pub struct Type {
    #[serde(rename = "type")]
    kind: String,
    named: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Children {
    multiple: bool,
    required: bool,
    types: Vec<Type>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeType {
    #[serde(rename = "type")]
    kind: String,
    named: bool,
    fields: Option<HashMap<String, Children>>,
    children: Option<Children>,
}

lazy_static! {
    pub static ref NODE_TYPES: HashMap<String, NodeType> = {
        let node_type_list: Vec<NodeType> =
            serde_json::from_str(NODE_TYPES_JSON).expect("parsing node types");
        let mut node_type_dict: HashMap<String, NodeType> = HashMap::new();

        for node_type in node_type_list {
            node_type_dict.insert(node_type.kind.clone(), node_type);
        }

        node_type_dict
    };
}

#[cfg(test)]
mod tests {
    use super::NODE_TYPES;

    #[test]
    fn test_node_types() {
        assert_ne!(NODE_TYPES.len(), 0);
    }
}
