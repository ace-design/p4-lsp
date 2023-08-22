#![allow(dead_code)]

use std::fmt;

use indextree::{Arena, NodeId};

use tower_lsp::lsp_types::Url;

use crate::file::File;

#[derive(Debug)]
pub enum ControlState {
    InControl,
    NotInControl,
}
impl fmt::Display for ControlState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("State")
    }
}

#[derive(Debug)]
pub struct Information {
    url: Url,
    control_state: ControlState,
}

impl Information {
    pub fn new(url: Url, control_state: ControlState) -> Information {
        Information {
            url: url,
            control_state: control_state,
        }
    }
    pub fn get_url(&self) -> Url {
        self.url.clone()
    }
}
impl fmt::Display for Information {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(format!("Url: {} Control State: {}",&self.url.to_string(),self.control_state).as_str())
    }
}
impl Clone for Information {
    fn clone(&self) -> Self {
        match self.control_state {
            ControlState::InControl => {
                return Information {
                    url: self.url.clone(),
                    control_state: ControlState::InControl,
                }
            }
            ControlState::NotInControl => {
                return Information {
                    url: self.url.clone(),
                    control_state: ControlState::NotInControl,
                }
            }
        }
    }
}
impl PartialEq for Information {
    fn eq(&self, other: &Information) -> bool {
        return self.url == other.url;
    }
    fn ne(&self, other: &Information) -> bool {
        !self.eq(other)
    }
}
impl Clone for File {
    fn clone(&self) -> Self {
        return File {
            uri: self.uri.clone(),
            source_code: self.source_code.clone(),
            tree: self.tree.clone(),
            symbol_table_manager: self.symbol_table_manager.clone(),
            ast_manager: self.ast_manager.clone(),
        };
    }
}
impl PartialEq for File {
    fn eq(&self, other: &File) -> bool {
        return self.uri == other.uri;
    }
    fn ne(&self, other: &File) -> bool {
        !self.eq(other)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub file: Option<File>,
    pub file_information: Option<Information>,
}

impl Node {
    pub fn new(file: Option<File>, information: Option<Information>) -> Node {
        Node {
            file: file,
            file_information: information,
        }
    }
}


impl fmt::Display for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("Node")
    }
}



#[derive(Debug, Clone)]
pub struct FileTree {
    arena: Arena<Node>,
    root_id: NodeId,
}

impl fmt::Display for FileTree {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.get_debug_tree())
    }
}

impl FileTree {
    pub fn initialize(arena: Arena<Node>, root_id: NodeId) -> FileTree {
        FileTree { arena, root_id }
    }

    pub fn get_root_id(&mut self) -> &mut NodeId {
         &mut self.root_id
    }

    pub fn get_prop(&mut self) -> Option<(&mut NodeId,&mut  Arena<Node>)>{
        Some((&mut self.root_id,&mut self.arena))
    }

    pub fn get_debug_tree(&self) -> String {
        let mut result = String::new();
        self._get_debug_tree(self.root_id, "", true, &mut result);
        result
    }

    pub fn get_arena(&mut self) -> &mut Arena<Node> {
        &mut self.arena
    }

    fn _get_debug_tree(&self, node_id: NodeId, indent: &str, last: bool, result: &mut String) {
        let node = self.arena.get(node_id).unwrap().get().clone();
        let a:Vec<String> = node_id.children(&self.arena).map(|descendant| {
            // Do something with the descendant node
            let node_temp = self.arena.get(descendant).unwrap().get().clone();
            return node_temp.file_information.unwrap().get_url().to_string()
        }).collect();
        let line = format!(
            "{}{} {:?} \n",
            indent,
            if last { "+- " } else { "|- " },
            node.file_information
        );

        result.push_str(&line);
        let indent = if last {
            indent.to_string() + "   "
        } else {
            indent.to_string() + "|  "
        };

        for (i, child) in node_id.children(&self.arena).enumerate() {
            self._get_debug_tree(
                child,
                &indent,
                i == node_id.children(&self.arena).collect::<Vec<_>>().len() - 1,
                result,
            );
        }
    }
}
