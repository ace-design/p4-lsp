use indextree::{Arena, NodeId};

use super::tree::{Ast, Direction, Node, NodeKind, TypeDecType};
use crate::metadata::types::{BaseType, Type, TypeList};
use crate::utils;

pub struct TreesitterTranslator {
    arena: Arena<Node>,
    source_code: String,
    tree: tree_sitter::Tree,
}

impl TreesitterTranslator {
    fn new(source_code: String, tree: tree_sitter::Tree) -> TreesitterTranslator {
        TreesitterTranslator {
            arena: Arena::new(),
            source_code,
            tree,
        }
    }

    pub fn translate(source_code: String, tree: tree_sitter::Tree) -> Ast {
        let mut translator = TreesitterTranslator::new(source_code, tree);
        let root_id = translator.parse_root();

        Ast::initialize(translator.arena, root_id)
    }

    fn new_error_node(&mut self, node: &tree_sitter::Node) -> NodeId {
        self.arena
            .new_node(Node::new(NodeKind::Error, node, &self.source_code))
    }

    fn parse_root(&mut self) -> NodeId {
        let root_syntax_node = self.tree.root_node();
        let ast_root = self.arena.new_node(Node::new(
            NodeKind::Root,
            &root_syntax_node,
            &self.source_code,
        ));

        // TODO: REMOVE CLONE
        let tree = self.tree.clone();
        let mut cursor = tree.walk();
        for child in tree.root_node().named_children(&mut cursor) {
            let new_child = if child.is_error() {
                Some(self.new_error_node(&child))
            } else {
                match child.kind() {
                    "constant_declaration" => self.parse_const_dec(&child),
                    "parser_declaration" => self.parse_parser(&child),
                    "type_declaration" => self.parse_type_dec(&child),
                    "control_declaration" => self.parse_control(&child),
                    _ => None,
                }
            };

            if let Some(new_child) = new_child {
                ast_root.append(new_child, &mut self.arena);
            }
        }

        ast_root
    }

    fn parse_const_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::ConstantDec, node, &self.source_code));

        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // Add value node
        let value_node = node.child_by_field_name("value").unwrap();
        node_id.append(
            self.parse_value(&value_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        Some(node_id)
    }

    fn parse_value(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {

        fn loop_value(node_value: &mut NodeId, last_node: NodeId, node: &tree_sitter::Node, selfV : &mut TreesitterTranslator) -> NodeId{
            let kind = node.kind();
            //debug!("{}", kind);
            let mut name_node : NodeId = last_node;
            let accept = ["non_type_name","type_name","apply","key","actions","state","entries","type"];
            if kind != "expression" && kind != "initializer" {
                //debug!("{},{}", kind,accept.contains(&kind));
                if accept.contains(&kind){
                    name_node = selfV.arena.new_node(Node::new(
                        NodeKind::ValueSymbol,
                        &node,
                        &selfV.source_code,
                    ));
                    node_value.append(name_node, &mut selfV.arena);
                } else if kind == "member" || kind == "name"{
                    //debug!("seconde node : {},{:?},{}", kind, node, utils::get_node_text(&node, &selfV.source_code));
                    name_node = selfV.arena.new_node(Node::new(
                        NodeKind::ValueSymbol,
                        &node,
                        &selfV.source_code,
                    ));
                    last_node.append(name_node, &mut selfV.arena);
                }
            }
            else{
                let mut cursor = node.walk();
                for field_child in node.children(&mut cursor) {
                    if field_child.is_error(){
                    }
                    else{
                        let mut loop_child = 1;
                        while loop_child != 0 {
                            match field_child.child(loop_child-1){
                                Some(x) => {
                                    name_node = loop_value(node_value, name_node.clone(), &x, selfV);
                                    loop_child += 1;
                                }
                                None => {
                                    loop_child = 0;
                                }
                            }                            
                        }
                    }
                }
            }
            return name_node;
        }

        let mut node_value = self.arena
            .new_node(Node::new(NodeKind::Value, node, &self.source_code));
        let mut last_node = node_value.clone();
        
        loop_value(&mut node_value, last_node.clone(), node, self);

        Some(node_value)
    }

    fn parse_type_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let type_kind_node = node.child_by_field_name("type_kind")?;

        let type_type = match type_kind_node.kind() {
            "typedef_declaration" => TypeDecType::TypeDef,
            "header_type_declaration" => TypeDecType::HeaderType,
            "header_union_declaration" => TypeDecType::HeaderUnion,
            "struct_type_declaration" => TypeDecType::Struct,
            "enum_declaration" => TypeDecType::Enum,
            "parser_type_declaration" => TypeDecType::Parser,
            "control_type_declaration" => TypeDecType::Control,
            "package_type_declaration" => TypeDecType::Package,
            _ => return None,
        };

        let node_id = self.arena.new_node(Node::new(
            NodeKind::TypeDec(type_type.clone()),
            &node,
            &self.source_code,
        ));

        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &type_kind_node.child_by_field_name("name")?,
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        match type_type {
            TypeDecType::TypeDef => {
                let type_node: NodeId =
                    self.parse_type_ref(&type_kind_node.child_by_field_name("type")?)?;
                node_id.append(type_node, &mut self.arena);
            },
            TypeDecType::HeaderType => {
                match type_kind_node.child_by_field_name("field_list"){
                    Some(x) => {
                        node_id.append(self.parse_type_fields_dec(&x).unwrap_or_else(|| self.new_error_node(&x)), &mut self.arena); 
                    },
                    None => {
                    }              
                }
            },
            TypeDecType::HeaderUnion => {
                match type_kind_node.child_by_field_name("field_list"){
                    Some(x) => {
                        node_id.append(self.parse_type_fields_dec(&x).unwrap_or_else(|| self.new_error_node(&x)), &mut self.arena); 
                    },
                    None => {
                    }              
                }
            },
            TypeDecType::Struct => {
                match type_kind_node.child_by_field_name("field_list"){
                    Some(x) => {
                        node_id.append(self.parse_type_fields_dec(&x).unwrap_or_else(|| self.new_error_node(&x)), &mut self.arena);
                    },
                    None => {
                    }
                }
            },
            TypeDecType::Enum => {
                match type_kind_node.child_by_field_name("type"){
                    Some(x) => {node_id.append(self.parse_type_ref(&x).unwrap_or_else(|| self.new_error_node(&x)), &mut self.arena);},
                    None => {}
                }
                match type_kind_node.child_by_field_name("option_list"){
                    Some(x) => {
                        node_id.append(self.parse_type_options_dec(&x).unwrap_or_else(|| self.new_error_node(&x)), &mut self.arena); 
                    },
                    None => {
                    }              
                }
            },
            TypeDecType::Parser => {
                if let Some(params_syntax_node) = type_kind_node.child_by_field_name("parameters"){
                    let params_node_id = self
                        .parse_params(&params_syntax_node)
                        .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
                    node_id.append(params_node_id, &mut self.arena);
                }
            },
            TypeDecType::Control => {
                if let Some(params_syntax_node) = type_kind_node.child_by_field_name("parameters"){
                    let params_node_id = self
                        .parse_params(&params_syntax_node)
                        .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
                    node_id.append(params_node_id, &mut self.arena);
                }
            },
            TypeDecType::Package => {
                if let Some(name_node) = type_kind_node.child_by_field_name("name"){
                    let name_node_id: NodeId = self.arena.new_node(Node::new(
                        NodeKind::Name,
                        &name_node,
                        &self.source_code,
                    ));
                    node_id.append(name_node_id, &mut self.arena);

                    /*if let Some(params_syntax_node) = type_kind_node.child_by_field_name("parameters"){
                        node_id.append(self
                            .parse_params(&params_syntax_node)
                            .unwrap_or_else(|| self.new_error_node(&params_syntax_node)),
                            &mut self.arena);
                    }*/
                }else{
                    if let Some(params_syntax_node) = type_kind_node.child_by_field_name("parameters"){
                        node_id.append(self
                            .parse_params(&params_syntax_node)
                            .unwrap_or_else(|| self.new_error_node(&params_syntax_node)),
                            &mut self.arena);
                    }
                }
            },
            _ => {}
        }

        Some(node_id)
    }
    
    fn parse_type_fields_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let fields_node_id =
        self.arena
            .new_node(Node::new(NodeKind::Fields, &node, &self.source_code));

        let mut cursor = node.walk();
        for field_child in node.named_children(&mut cursor) {
            let new_node_id = if field_child.is_error() {
                self.new_error_node(&field_child)
            } else {
                let field_node_id = self.arena.new_node(Node::new(
                    NodeKind::Field,
                    &field_child,
                    &self.source_code,
                ));

                // Add name node
                match field_child.child_by_field_name("name"){
                    Some(x) => {
                        field_node_id.append(self.arena.new_node(Node::new(
                                NodeKind::Name,
                                &x,
                                &self.source_code,
                            )),
                            &mut self.arena,
                        );
                    },
                    None => {},
                }

                // Add type node
                match field_child.child_by_field_name("type"){
                    Some(x) => {
                        field_node_id.append(
                            self.parse_type_ref(&x)
                                .unwrap_or_else(|| self.new_error_node(&x)),
                            &mut self.arena,
                        );
                    },
                    None => {},
                }

                field_node_id

            };

            fields_node_id.append(new_node_id, &mut self.arena);
        }
        return Some(fields_node_id);
    }
    
    fn parse_type_options_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let options_node_id =
        self.arena
            .new_node(Node::new(NodeKind::Options, &node, &self.source_code));

        let mut cursor = node.walk();
        for option_child in node.named_children(&mut cursor) {
            let new_node_id = if option_child.is_error() {
                self.new_error_node(&option_child)
            } else {
                //let node_text = utils::get_node_text(&option_child, &self.source_code);
                //let text = node_text.as_str().trim();
                
                let option_node_id = self.arena.new_node(Node::new(
                    NodeKind::Option,
                    &option_child,
                    &self.source_code,
                ));

                // Add name node
                option_node_id.append(self.arena.new_node(Node::new(
                        NodeKind::Name,
                        &option_child,
                        &self.source_code,
                    )),
                    &mut self.arena,
                );

                option_node_id

            };

            options_node_id.append(new_node_id, &mut self.arena);
        }
        return Some(options_node_id);
    }

    fn parse_type_ref(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let child = node.named_child(0)?;
        let type_type: Type = match child.kind() {
            "base_type" => Type::Base(self.parse_base_type(&child)?),
            "type_name" => Type::Name,
            "specialized_type" => Type::Specialized,
            "header_stack_type" => Type::Header,
            "tuple_type" => Type::Tuple,
            _ => return None,
        };
        let node = self.arena.new_node(Node::new(
            NodeKind::Type(type_type),
            node,
            &self.source_code,
        ));

        Some(node)
    }
    fn parse_type_list_ref(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let child = node.named_child(0)?;
        let type_type: TypeList = match child.kind() {
            "base_type" => TypeList::Base(self.parse_base_type(&child)?),
            "type_name" => TypeList::Name,
            "specialized_type" => TypeList::Specialized,
            "header_stack_type" => TypeList::Header,
            "tuple_type" => TypeList::Tuple,
            _ => return None,
        };
        let node = self.arena.new_node(Node::new(
            NodeKind::TypeList(type_type),
            node,
            &self.source_code,
        ));

        Some(node)
    }

    fn parse_base_type(&self, node: &tree_sitter::Node) -> Option<BaseType> {
        
        let node_text = utils::get_node_text(node, &self.source_code);
        let text = node_text.as_str().trim();

        match text {
            "bool" => Some(BaseType::Bool),
            "int" => Some(BaseType::Int),
            "bit" => Some(BaseType::Bit),
            "string" => Some(BaseType::String),
            "varbit" => Some(BaseType::Varbit),
            "error" => Some(BaseType::Error),
            "match_kind" => Some(BaseType::MatchKind),
            _ => {
                let child = node.named_child(0).unwrap();
                let size = if child.kind() == "integer" {
                    Some(
                        utils::get_node_text(&child, &self.source_code)
                            .parse::<u32>()
                            .unwrap(),
                    )
                } else {
                    None
                };

                if text.starts_with("int") {
                    Some(BaseType::SizedInt(size))
                } else if text.starts_with("bit") {
                    Some(BaseType::SizedBit(size))
                } else if text.starts_with("varbit") {
                    Some(BaseType::SizedVarbit(size))
                } else {
                    None
                }
            }
        }
    }

    fn parse_parser(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::ParserDec, node, &self.source_code));

        let (name_node_id, parameters_node_id) = self
            .parse_parser_type_dec(&node.child_by_field_name("declaration")?)
            .unwrap();
        node_id.append(name_node_id, &mut self.arena);
        node_id.append(parameters_node_id, &mut self.arena);

        let body_syntax_node = &node.child_by_field_name("body")?;
        let body_node_id = self.arena.new_node(Node::new(
            NodeKind::Body,
            body_syntax_node,
            &self.source_code,
        ));
        node_id.append(body_node_id, &mut self.arena);

        let mut cursor = body_syntax_node.walk();
        for syntax_child in body_syntax_node.named_children(&mut cursor) {
            let child_node_id = match syntax_child.kind() {
                // _parser_local_element
                "constant_declaration" => self.parse_const_dec(&syntax_child),
                "variable_declaration" => self.parse_var_dec(&syntax_child),
                //"instantiation" => self.instantiation(&syntax_child),
                "value_set_declaration" => self.parse_val_set(&syntax_child),

                // parser_state
                "parser_state" => self.parse_state(&syntax_child),
                _ => None,
            };

            if let Some(id) = child_node_id {
                body_node_id.append(id, &mut self.arena);
            }
        }

        Some(node_id)
    }

    fn parse_parser_type_dec(&mut self, node: &tree_sitter::Node) -> Option<(NodeId, NodeId)> {
        let name_node_id = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name")?,
            &self.source_code,
        ));

        let params_syntax_node = node.child_by_field_name("parameters").unwrap();
        let params_node_id = self
            .parse_params(&params_syntax_node)
            .unwrap_or_else(|| self.new_error_node(&params_syntax_node));

        Some((name_node_id, params_node_id))
    }

    fn parse_control(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::ControlDec, node, &self.source_code));

        let (name_node_id, parameters_node_id) = self
            .parse_parser_type_dec(&node.child_by_field_name("declaration")?)
            .unwrap();
        node_id.append(name_node_id, &mut self.arena);
        node_id.append(parameters_node_id, &mut self.arena);

        let body_syntax_node = &node.child_by_field_name("body")?;
        let body_node_id = self.arena.new_node(Node::new(
            NodeKind::Body,
            body_syntax_node,
            &self.source_code,
        ));
        node_id.append(body_node_id, &mut self.arena);

        let mut cursor = body_syntax_node.walk();
        for syntax_child in body_syntax_node.named_children(&mut cursor) {
            let child_node_id = match syntax_child.kind() {
                "constant_declaration" => self.parse_const_dec(&syntax_child),
                "variable_declaration" => self.parse_var_dec(&syntax_child),
                "instantiation" => self.instantiation(&syntax_child),
                "action_declaration" => self.parse_control_action(&syntax_child),
                "table_declaration" => self.parse_control_table(&syntax_child),

                "block_statement" => self.parse_block(&syntax_child),
                _ => None,
            };

            if let Some(id) = child_node_id {
                body_node_id.append(id, &mut self.arena);
            }
        }

        Some(node_id)
    }

    fn parse_control_type_dec(&mut self, node: &tree_sitter::Node) -> Option<(NodeId, NodeId)> {
        let name_node_id = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name")?,
            &self.source_code,
        ));

        let params_syntax_node = node.child_by_field_name("parameters").unwrap();
        let params_node_id = self
            .parse_params(&params_syntax_node)
            .unwrap_or_else(|| self.new_error_node(&params_syntax_node));

        Some((name_node_id, params_node_id))
    }
    
    fn parse_params(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let params_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Params, &node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            //debug!("a,{:?}",syntax_child);
            let new_node_id = if syntax_child.is_error() {
                self.new_error_node(&syntax_child)
            } else {
                let param_node_id = self.arena.new_node(Node::new(
                    NodeKind::Param,
                    &syntax_child,
                    &self.source_code,
                ));

                // Add name node
                let name_node_id = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &syntax_child.child_by_field_name("name")?,
                    &self.source_code,
                ));
                param_node_id.append(name_node_id, &mut self.arena);

                // Add type node
                let type_syntax_node = syntax_child.child_by_field_name("type")?;
                param_node_id.append(
                    self.parse_type_ref(&type_syntax_node)
                        .unwrap_or_else(|| self.new_error_node(&type_syntax_node)),
                    &mut self.arena,
                );

                // Add direction node
                if let Some(dir_syntax_node) = syntax_child.child_by_field_name("direction") {
                    param_node_id.append(
                        self.parse_direction(&dir_syntax_node)
                            .unwrap_or_else(|| self.new_error_node(&dir_syntax_node)),
                        &mut self.arena,
                    )
                };

                // Add value node
                if let Some(value_syntax_node) = syntax_child.child_by_field_name("value") {
                    param_node_id.append(
                        self.parse_value(&value_syntax_node)
                            .unwrap_or_else(|| self.new_error_node(&value_syntax_node)),
                        &mut self.arena,
                    );
                }

                param_node_id
            };
            //debug!("a,{:?}",new_node_id);

            params_node_id.append(new_node_id, &mut self.arena);
        }

        Some(params_node_id)
    }
    fn parse_args(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let params_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Params, &node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            //debug!("a,{:?}",syntax_child);
            let new_node_id = if syntax_child.is_error() {
                self.new_error_node(&syntax_child)
            } else {
                let param_node_id = self.arena.new_node(Node::new(
                    NodeKind::Param,
                    &syntax_child,
                    &self.source_code,
                ));

                // Add name node
                if let Some(name_node) = syntax_child.child_by_field_name("name"){
                    let name_node_id: NodeId = self.arena.new_node(Node::new(
                        NodeKind::Name,
                        &name_node,
                        &self.source_code,
                    ));
                    param_node_id.append(name_node_id, &mut self.arena);
                }
                // Add value node
                if let Some(value_syntax_node) = syntax_child.child_by_field_name("expression") {
                    param_node_id.append(
                        self.parse_value(&value_syntax_node)
                            .unwrap_or_else(|| self.new_error_node(&value_syntax_node)),
                        &mut self.arena,
                    );
                }

                param_node_id
            };
            //debug!("a,{:?}",new_node_id);

            params_node_id.append(new_node_id, &mut self.arena);
        }

        Some(params_node_id)
    }
    fn parse_obj_initializer(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let obj_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Obj, &node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            let child_node_id = match syntax_child.kind() {
                "function_declaration" => self.function_declaration(&syntax_child),
                "instantiation" => self.instantiation(&syntax_child),
                _ => None,
            };
            debug!("{:?},{:?}",syntax_child.kind(),child_node_id);
            if let Some(child_node) = child_node_id {
                obj_node_id.append(child_node, &mut self.arena);
            }
        }

        Some(obj_node_id)
    }
    fn function_declaration(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let fn_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Function, &node, &self.source_code));
        
        if let Some(x) = self.parse_block(&node.named_child(0)?){
            fn_node_id.append(x, &mut self.arena);
        }
        
        if let Some(x) = self.function_prototype(&node.named_child(1)?){
            fn_node_id.append(x, &mut self.arena);
        }

        Some(fn_node_id)
    }
    fn function_prototype(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let fn_node_id =
            self.arena
                .new_node(Node::new(NodeKind::FunctionName, &node, &self.source_code));
        
        let type_node = node.child_by_field_name("type").unwrap();
        if type_node.kind() == "type_ref"{
            // TODO
            fn_node_id.append(
                self.parse_type_ref(&type_node)
                    .unwrap_or_else(|| self.new_error_node(&type_node)),
                &mut self.arena,
            );
        }

        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name")?,
            &self.source_code,
        ));
        fn_node_id.append(name_node, &mut self.arena);

        if let Some(params_syntax_node) = node.child_by_field_name("parameters_list"){
            fn_node_id.append(self
                .parse_params(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node)),
                &mut self.arena);
        }

        Some(fn_node_id)
    }
    fn parse_block(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let block_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Block, &node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            let child_node_id = match syntax_child.kind() {
                "constant_declaration" => self.parse_const_dec(&syntax_child),
                "variable_declaration" => self.parse_var_dec(&syntax_child),
                "assignment_or_method_call_statement" => self.parse_state_assignment(&syntax_child),
                "direct_application" => self.parse_state_direct(&syntax_child),
                "conditional_statement" => self.parse_state_conditional(&syntax_child),
                "empty_statement" => None,
                "block_statement" => self.parse_block(&syntax_child),

                "exit_statement" => None,
                "return_statement" => self.return_statement(&syntax_child),
                "switch_statement" => self.switch_statement(&syntax_child),
                _ => None,
            };
            debug!("{:?},{:?}",syntax_child.kind(),child_node_id);
            if let Some(child_node) = child_node_id {
                block_node_id.append(child_node, &mut self.arena);
            }
        }

        Some(block_node_id)
    }

    fn parse_direction(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let dir = match utils::get_node_text(node, &self.source_code).as_str() {
            "in" => Direction::In,
            "out" => Direction::Out,
            "inout" => Direction::InOut,
            _ => {
                debug!("None");
                return None;
            }
        };

        Some(
            self.arena
                .new_node(Node::new(NodeKind::Direction(dir), node, &self.source_code)),
        )
    }
    
    fn parse_var_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::VariableDec, node, &self.source_code));

        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // Add value node
        let value_node = node.child_by_field_name("value").unwrap();
        node_id.append(
            self.parse_value(&value_node)
                .unwrap_or_else(|| self.new_error_node(&value_node)),
            &mut self.arena,
        );

        Some(node_id)
    }
    fn parse_state_assignment(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::Assignment, node, &self.source_code));

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // Add value node
        if let Some(value_node) = node.child_by_field_name("expression"){
            node_id.append(
                self.parse_value(&value_node)
                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                &mut self.arena,
            );
        }
        
        if let Some(params_syntax_node) = node.child_by_field_name("parameters"){
            let params_node_id = self
                .parse_args(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }
        if let Some(param_list) = node.child_by_field_name("type"){
            debug!("{:?}",param_list);
            debug!("{:?}",param_list.named_child(0));

            let params_node_id =
            self.arena
                .new_node(Node::new(NodeKind::ParamsList, &node, &self.source_code));

            let mut cursor = param_list.walk();
            for syntax_child in param_list.named_children(&mut cursor) {
                let child_node_id = match syntax_child.named_child(0)?.kind() {
                    "type_ref" => self.parse_type_list_ref(&syntax_child),
                    "non_type_name" => {return Some(self.arena.new_node(Node::new(
                        NodeKind::TypeList(TypeList::NoName),
                        node,
                        &self.source_code,
                    )));},
                    _ => Some(self.new_error_node(&syntax_child)),
                };
                debug!("{:?},{:?},{:?}",syntax_child,syntax_child.named_child(0)?.kind(),child_node_id);
                if let Some(child_node) = child_node_id {
                    params_node_id.append(child_node, &mut self.arena);
                }
            }

            node_id.append(params_node_id, &mut self.arena);

        }


        Some(node_id)
    }
    fn parse_state_direct(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id: NodeId =
            self.arena
                .new_node(Node::new(NodeKind::Assignment, node, &self.source_code));
        
        let type_name: Type;
        let name_type: tree_sitter::Node;
        if let Some(x) = node.child_by_field_name("name"){
            name_type = x;
            type_name = Type::Name;
        }else{
            name_type = node.child_by_field_name("specialized")?;
            type_name = Type::Specialized;
        }

        node_id.append(self.arena.new_node(Node::new(
            NodeKind::Type(type_name),
            &name_type,
            &self.source_code,
        )), &mut self.arena);
        
        if let Some(params_syntax_node) = node.child_by_field_name("args"){
            let params_node_id = self
                .parse_args(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }


        Some(node_id)
    }
    fn parse_state_block(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id: NodeId =
            self.arena
                .new_node(Node::new(NodeKind::ParserBlock, node, &self.source_code));
            
        let body = node.child_by_field_name("body")?;
        let mut cursor = body.walk();
        for body_child in body.named_children(&mut cursor) {
            let child_node_id = match body_child.kind() {
                "assignment_or_method_call_statement" => self.parse_state_assignment(&body_child),
                "direct_application" => self.parse_state_direct(&body_child),
                "parser_block_statement" => self.parse_state_block(&body_child),
                "constant_declaration" => self.parse_const_dec(&body_child),
                "variable_declaration" => self.parse_var_dec(&body_child),
                "empty_statement" => None,
                "conditional_statement" => self.parse_state_conditional(&body_child),
                _ => None,
            };

            if let Some(id) = child_node_id {
                node_id.append(id, &mut self.arena);
            }
        }

        Some(node_id)
    }
    
    fn parse_state_conditional(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::Assignment, node, &self.source_code));

        // Add value node
        if let Some(value_node) = node.child_by_field_name("expression"){
            node_id.append(
                self.parse_value(&value_node)
                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                &mut self.arena,
            );
        }
        
        // Add body if
        let node_if = node.child_by_field_name("bodyIf").unwrap();
        node_id.append(self.fn_statement(node_if, NodeKind::BodyIf)?, &mut self.arena);
        
        // Add body if
        if let Some(node_else) = node.child_by_field_name("bodyElse"){
            node_id.append(self.fn_statement(node_else, NodeKind::BodyElse)?, &mut self.arena);
        }

        Some(node_id)
    }
    fn fn_statement(&mut self, node: tree_sitter::Node<'_>, type_node : NodeKind) -> Option<NodeId>{
        let node_id = self.arena.new_node(Node::new(
            type_node,
            &node,
            &self.source_code,
        ));

        let mut cursor = node.walk();
        for body_child in node.named_children(&mut cursor) {
            let child_node_id = match body_child.kind() {
                "assignment_or_method_call_statement" => self.parse_state_assignment(&body_child),
                "direct_application" => self.parse_state_direct(&body_child),
                "conditional_statement" => self.parse_state_conditional(&body_child),
                "empty_statement" => None,
                "block_statement" => self.parse_block(&body_child),

                "exit_statement" => None,
                "return_statement" => self.return_statement(&body_child),
                "switch_statement" => self.switch_statement(&body_child),
                _ => None,
            };

            if let Some(id) = child_node_id {
                node_id.append(id, &mut self.arena);
            }
        }


        Some(node_id)
    }
    fn return_statement(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::Return, node, &self.source_code));

        // Add value node
        if let Some(value_node) = node.child_by_field_name("expression"){
            node_id.append(
                self.parse_value(&value_node)
                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                &mut self.arena,
            );
            return 
            Some(node_id);
        }
        return None;

    }
    fn switch_statement(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::Switch, node, &self.source_code));

        // Add value node
        let value_node = node.child_by_field_name("expression")?;
        node_id.append(
            self.parse_value(&value_node)
                .unwrap_or_else(|| self.new_error_node(&value_node)),
            &mut self.arena,
        );

        let body_node = node.child_by_field_name("body")?;
        let mut cursor = body_node.walk();
        for body_child in body_node.named_children(&mut cursor) {
            let label =
                self.arena
                    .new_node(Node::new(NodeKind::SwitchLabel, node, &self.source_code));
            
            label.append(
                self.parse_value(&body_child.child_by_field_name("name")?)
                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                &mut self.arena,
            );
            
            if let Some(value_node) = node.child_by_field_name("value"){
                label.append(
                    self.parse_block(&value_node)
                        .unwrap_or_else(|| self.new_error_node(&value_node)),
                    &mut self.arena,
                );
            }
            node_id.append(label, &mut self.arena);    
        }
        
        Some(node_id)

    }
    


    fn parse_val_set(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::ValueSet, node, &self.source_code));

        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // Add value node
        let value_node = node.child_by_field_name("expression").unwrap();
        node_id.append(
            self.parse_value(&value_node)
                .unwrap_or_else(|| self.new_error_node(&value_node)),
            &mut self.arena,
        );

        Some(node_id)
    }
    
    fn instantiation(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::Instantiation, node, &self.source_code));

        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        if let Some(params_syntax_node) = node.child_by_field_name("args"){
            let params_node_id = self
                .parse_args(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }

        if let Some(params_syntax_node) = node.child_by_field_name("obj"){
            let params_node_id = self
                .parse_obj_initializer(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_control_action(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::ControlAction, node, &self.source_code));


        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);


        if let Some(params_syntax_node) = node.child_by_field_name("parameters"){
            let params_node_id = self
                .parse_params(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }

        let block_syntax_node = node.child_by_field_name("block").unwrap();
        let block_node_id = self
            .parse_block(&block_syntax_node)
            .unwrap_or_else(|| self.new_error_node(&block_syntax_node));
        node_id.append(block_node_id, &mut self.arena);

        Some(node_id)
    }

    fn parse_control_table(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id: NodeId =
            self.arena
                .new_node(Node::new(NodeKind::ControlAction, node, &self.source_code));


        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // TODO
        //let table_syntax_node = node.child_by_field_name("table").unwrap();
        //let table_node_id = self
        //    .parse_table(&table_syntax_node)
        //    .unwrap_or_else(|| self.new_error_node(&table_syntax_node));
        //node_id.append(table_node_id, &mut self.arena);

        Some(node_id)
    }

    fn parse_state(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::StateParser, node, &self.source_code));

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // Add value node
        let body_node = node.child_by_field_name("body").unwrap();
        let value_node = self.arena.new_node(Node::new(
            NodeKind::Body,
            &body_node,
            &self.source_code,
        ));

        let statement = body_node.child_by_field_name("statement")?;
        let mut cursor = statement.walk();
        for body_child in statement.named_children(&mut cursor) {
            let child_node_id = match body_child.kind() {
                "assignment_or_method_call_statement" => self.parse_state_assignment(&body_child),
                "direct_application" => self.parse_state_direct(&body_child),
                "parser_block_statement" => self.parse_state_block(&body_child),
                "constant_declaration" => self.parse_const_dec(&body_child),
                "variable_declaration" => self.parse_var_dec(&body_child),
                "empty_statement" => None,
                "conditional_statement" => self.parse_state_conditional(&body_child),
                _ => None,
            };

            if let Some(id) = child_node_id {
                value_node.append(id, &mut self.arena);
            }
        }

        if let Some(transition_statement) = body_node.child_by_field_name("transition_statement"){
            let transition_node = self.arena.new_node(Node::new(
                NodeKind::TransitionStatement,
                &transition_statement,
                &self.source_code,
            ));

            if let Some(name) = transition_statement.child_by_field_name("name"){
                transition_node.append(self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &name,
                    &self.source_code,
                )), &mut self.arena);
            } else{
                let t = transition_statement.named_child(0)?;
                let select_expression_params = t.named_child(0)?.named_child(0);
                let select_expression_body = t.named_child(1)?.named_child(0);


                if let Some(select_expression_params_node) = select_expression_params{
                    let expression_params_node = self.arena.new_node(Node::new(
                        NodeKind::Params,
                        &select_expression_params_node,
                        &self.source_code,
                    ));
                    
                    let mut cursor = select_expression_params_node.walk();
                    for param_child in select_expression_params_node.named_children(&mut cursor) {
                        expression_params_node.append(
                            self.parse_value(&param_child)
                                .unwrap_or_else(|| self.new_error_node(&param_child)),
                            &mut self.arena,
                        );
                    }
                    transition_node.append(expression_params_node, &mut self.arena);
                }
                if let Some(select_expression_body_node) = select_expression_body{
                    let expression_body_node = self.arena.new_node(Node::new(
                        NodeKind::Body,
                        &select_expression_body_node,
                        &self.source_code,
                    ));
                    let mut cursor = select_expression_body_node.walk();
                    for body_child in select_expression_body_node.named_children(&mut cursor) {
                        let t = self.arena.new_node(Node::new(
                            NodeKind::Row,
                            &body_node,
                            &self.source_code,
                        ));

                        // Add name node
                        match body_child.child_by_field_name("name"){
                            Some(x) => {
                                t.append(self.arena.new_node(Node::new(
                                        NodeKind::Name,
                                        &x,
                                        &self.source_code,
                                    )),
                                    &mut self.arena,
                                );
                            },
                            None => {},
                        }

                        // Add type node, TODO
                        /*match body_child.child_by_field_name("type"){
                            Some(x) => {
                                t.append(
                                    self.parse_type_ref(&x)
                                        .unwrap_or_else(|| self.new_error_node(&x)),
                                    &mut self.arena,
                                );
                            },
                            None => {},
                        }*/
                        expression_body_node.append(t,&mut self.arena);
                    }
                    transition_node.append(expression_body_node, &mut self.arena);
                }
            }

            node_id.append(transition_node, &mut self.arena);
        }
        node_id.append(value_node, &mut self.arena);


        Some(node_id)
    }
}

#[cfg(test)]
mod tests {
    use indextree::Arena;
    use tree_sitter::{Parser, Tree};
    use tree_sitter_p4::language;

    use super::super::tree::{Node, NodeKind, TypeDecType};
    use super::TreesitterTranslator;
    use super::{BaseType, Type};

    fn get_syntax_tree(source_code: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(language()).unwrap();
        parser.parse(source_code, None).unwrap()
    }

    fn print_arenas(expected: &Arena<Node>, actual: &Arena<Node>) {
        println!("Expected:");
        for node in expected.iter() {
            println!("{:?}", node.get());
        }
        println!();
        println!("Actual:");
        for node in actual.iter() {
            println!("{:?}", node.get());
        }
    }

    #[test]
    fn test_const_declaration() {
        let source_code = r#"
            const bit<16> TYPE_IPV4 = 10;
        "#;
        let syntax_tree = get_syntax_tree(source_code);
        let translated_ast =
            TreesitterTranslator::translate(source_code.to_string(), syntax_tree.clone());

        let mut arena: Arena<Node> = Arena::new();
        let mut syntax_node = syntax_tree.root_node();
        let root = arena.new_node(Node::new(NodeKind::Root, &syntax_node, source_code));

        syntax_node = syntax_node.named_child(0).unwrap();
        let constant_syntax_node = syntax_node;
        let constant_dec =
            arena.new_node(Node::new(NodeKind::ConstantDec, &syntax_node, source_code));
        root.append(constant_dec, &mut arena);

        syntax_node = constant_syntax_node.child_by_field_name("type").unwrap();
        let type_dec = arena.new_node(Node::new(
            NodeKind::Type(Type::Base(BaseType::SizedBit(Some(16)))),
            &syntax_node,
            source_code,
        ));
        constant_dec.append(type_dec, &mut arena);

        syntax_node = constant_syntax_node.child_by_field_name("name").unwrap();
        let name_dec = arena.new_node(Node::new(NodeKind::Name, &syntax_node, source_code));
        constant_dec.append(name_dec, &mut arena);

        syntax_node = constant_syntax_node.child_by_field_name("value").unwrap();
        let value = arena.new_node(Node::new(NodeKind::Value, &syntax_node, source_code));
        constant_dec.append(value, &mut arena);

        print_arenas(&arena, &translated_ast.get_arena());
        assert!(translated_ast.get_arena().eq(&arena))
    }

    #[test]
    fn test_typedec_typedef() {
        let source_code = r#"
            typedef bit<9> egressSpec_t;
        "#;
        let syntax_tree = get_syntax_tree(source_code);
        let translated_ast =
            TreesitterTranslator::translate(source_code.to_string(), syntax_tree.clone());

        let mut arena: Arena<Node> = Arena::new();
        let mut syntax_node = syntax_tree.root_node();
        let root = arena.new_node(Node::new(NodeKind::Root, &syntax_node, source_code));

        syntax_node = syntax_node.named_child(0).unwrap();
        let typedec_syntax_node = syntax_node;
        let type_dec = arena.new_node(Node::new(
            NodeKind::TypeDec(TypeDecType::TypeDef),
            &syntax_node,
            source_code,
        ));
        root.append(type_dec, &mut arena);

        syntax_node = typedec_syntax_node
            .child(0)
            .unwrap()
            .child_by_field_name("name")
            .unwrap();
        let name_dec = arena.new_node(Node::new(NodeKind::Name, &syntax_node, source_code));
        type_dec.append(name_dec, &mut arena);

        syntax_node = typedec_syntax_node
            .child(0)
            .unwrap()
            .child_by_field_name("type")
            .unwrap();
        let type_type_dec = arena.new_node(Node::new(
            NodeKind::Type(Type::Base(BaseType::SizedBit(Some(9)))),
            &syntax_node,
            source_code,
        ));
        type_dec.append(type_type_dec, &mut arena);

        print_arenas(&arena, &translated_ast.get_arena());
        assert!(translated_ast.get_arena().eq(&arena))
    }
    /*#[test]
    fn test_typedec_headers() {
        let source_code = r#"
            header ethernet_t {
                macAddr_t dstAddr;
                macAddr_t srcAddr;
                bit<16>   etherType;
            }        
        "#;
        let syntax_tree = get_syntax_tree(source_code);
        let translated_ast =
            TreesitterTranslator::translate(source_code.to_string(), syntax_tree.clone());

        let mut arena: Arena<Node> = Arena::new();
        let mut syntax_node = syntax_tree.root_node();
        let root = arena.new_node(Node::new(NodeKind::Root, &syntax_node, source_code));

        syntax_node = syntax_node.named_child(0).unwrap();
        let typedec_syntax_node = syntax_node;
        let type_dec = arena.new_node(Node::new(
            NodeKind::TypeDec(TypeDecType::HeaderType),
            &syntax_node,
            source_code,
        ));
        root.append(type_dec, &mut arena);

        syntax_node = typedec_syntax_node
            .child(0)
            .unwrap()
            .child_by_field_name("name")
            .unwrap();
        let name_dec = arena.new_node(Node::new(NodeKind::Name, &syntax_node, source_code));
        type_dec.append(name_dec, &mut arena);

        let node = typedec_syntax_node
        .child(0)
        .unwrap()
        .child_by_field_name("field_list")
        .unwrap();

        let fields = arena.new_node(Node::new(
            NodeKind::Fields,
            &node,
            source_code,
        ));
        root.append(fields, &mut arena);

        let node1 = node.named_child(0).unwrap();
        let field = arena.new_node(Node::new(
            NodeKind::Field,
            &node1,
            source_code,
        ));
        fields.append(field, &mut arena);

        let name_dec = arena.new_node(Node::new(NodeKind::Name, &node1
            .child_by_field_name("name")
            .unwrap(), source_code));
        field.append(name_dec, &mut arena);

        let type_dec: indextree::NodeId = arena.new_node(Node::new(
            NodeKind::Type(Type::Name),
            &node1.child_by_field_name("type").unwrap(),
            source_code,
        ));
        field.append(type_dec, &mut arena);
        
        let node2 = node.child(1).unwrap();
        let field = arena.new_node(Node::new(
            NodeKind::Field,
            &node2,
            source_code,
        ));
        fields.append(field, &mut arena);

        let name_dec = arena.new_node(Node::new(NodeKind::Name, &node2
            .child_by_field_name("name")
            .unwrap(), source_code));
        field.append(name_dec, &mut arena);
        let type_dec = arena.new_node(Node::new(
            NodeKind::Type(Type::Name),
            &node2.child_by_field_name("type").unwrap(),
            source_code,
        ));
        field.append(type_dec, &mut arena);

        let node3 = node.child(2).unwrap();
        let field = arena.new_node(Node::new(
            NodeKind::Field,
            &node3,
            source_code,
        ));
        fields.append(field, &mut arena);

        let name_dec = arena.new_node(Node::new(NodeKind::Name, &node3
            .child_by_field_name("name")
            .unwrap(), source_code));
        field.append(name_dec, &mut arena);
        let type_dec = arena.new_node(Node::new(
            NodeKind::Type(Type::Base(BaseType::SizedBit(Some(16)))),
            &node3.child_by_field_name("type").unwrap(),
            source_code,
        ));
        field.append(type_dec, &mut arena);
        
        print_arenas(&arena, &translated_ast.get_arena());
        assert!(translated_ast.get_arena().eq(&arena))
    }*/
}
