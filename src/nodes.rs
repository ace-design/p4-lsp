use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tree_sitter_p4::NODE_TYPES as NODE_TYPES_JSON;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Type {
    #[serde(rename = "type")]
    pub kind: String,
    pub named: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Children {
    pub multiple: bool,
    pub required: bool,
    pub types: Vec<Type>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NodeType {
    #[serde(rename = "type")]
    pub kind: String,
    pub named: bool,
    pub fields: Option<HashMap<String, Children>>,
    pub children: Option<Children>,
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
    use super::{Children, NodeType, Type, NODE_TYPES};
    use std::collections::HashMap;

    #[test]
    fn test_node_types_parsed() {
        assert_ne!(NODE_TYPES.len(), 0);
    }

    #[test]
    fn test_node_types_parsed_correctly() {
        assert_eq!(
            NODE_TYPES.get("return_statement").unwrap(),
            &NodeType {
                kind: "return_statement".into(),
                named: true,
                fields: Some(HashMap::new()),
                children: Some(Children {
                    multiple: false,
                    required: false,
                    types: vec![Type {
                        kind: "expression".into(),
                        named: true,
                    }]
                }),
            }
        );
    }
}
