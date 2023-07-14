use indextree::{Arena, NodeId};

use super::tree::{Ast, Direction, Node, NodeKind, TypeDecType};
use crate::metadata::types::{BaseType, Type};
use crate::utils;

// todo : argument_list + annotation + expression value

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
            debug!("{:?}", child);
            let new_child = if child.is_error() {
                Some(self.new_error_node(&child))
            } else {
                match child.kind() {
                    "constant_declaration" => self.parse_const_dec(&child),
                    "parser_declaration" => self.parse_parser(&child),
                    "type_declaration" => self.parse_type_dec(&child),
                    "control_declaration" => self.parse_control(&child),
                    "action_declaration" => self.parse_control_action(&child),
                    "instantiation" => self.instantiation(&child),
                    "function_declaration" => self.function_declaration(&child),
                    "match_kind_declaration" => self.parse_match_kind(&child),
                    "error_declaration" => self.parse_error(&child),
                    "extern_declaration" => self.parse_extern(&child),

                    "preproc_include_declaration" => self.parse_preproc_include(&child),
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        self.parse_preproc_define(&child)
                    }
                    "preproc_undef_declaration" => self.parse_preproc_undef(&child),

                    _ => None,
                }
            };

            if let Some(new_child) = new_child {
                ast_root.append(new_child, &mut self.arena);
            }
        }

        ast_root
    }

    fn parse_method_prototype(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Methods, node, &self.source_code));

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.is_error() {
                node_id.append(self.new_error_node(&child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&child) {
                node_id.append(t, &mut self.arena);
            } else if !self.is_comment(&child) {
                let child_node_id =
                    self.arena
                        .new_node(Node::new(NodeKind::Method, node, &self.source_code));
                if let Some(annotation) = child.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }
                if let Some(type_node) = node.child_by_field_name("type") {
                    child_node_id.append(
                        self.parse_type_ref(&type_node, NodeKind::Type, true)
                            .unwrap_or_else(|| self.new_error_node(&type_node)),
                        &mut self.arena,
                    );

                    if let Some(paramters) = node.child_by_field_name("parameters") {
                        let params_node_id = self
                            .parse_params(&paramters)
                            .unwrap_or_else(|| self.new_error_node(&paramters));
                        child_node_id.append(params_node_id, &mut self.arena);
                    }
                }
                if let Some(function) = node.child_by_field_name("function") {
                    child_node_id.append(
                        self.function_prototype(&function)
                            .unwrap_or_else(|| self.new_error_node(&function)),
                        &mut self.arena,
                    );
                }

                for node_preproc in self.look_for_preproc(&child) {
                    child_node_id.append(node_preproc, &mut self.arena);
                }

                node_id.append(child_node_id, &mut self.arena);
            }
        }

        Some(node_id)
    }

    fn parse_extern(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Extern, node, &self.source_code));

        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }

        if let Some(node_name) = node.child_by_field_name("name") {
            let name_node =
                self.arena
                    .new_node(Node::new(NodeKind::Name, &node_name, &self.source_code));
            node_id.append(name_node, &mut self.arena);

            if let Some(paramters) = node.child_by_field_name("parameters_type") {
                node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
            }
            if let Some(method) = node.child_by_field_name("method") {
                node_id.append(
                    self.parse_method_prototype(&method)
                        .unwrap_or_else(|| self.new_error_node(&method)),
                    &mut self.arena,
                );
            }
        }
        if let Some(function) = node.child_by_field_name("function") {
            node_id.append(
                self.function_prototype(&function)
                    .unwrap_or_else(|| self.new_error_node(&function)),
                &mut self.arena,
            );
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }
        //node_id.append(self.parse_type_options_dec(&option_list_node).unwrap_or_else(|| self.new_error_node(&option_list_node)), &mut self.arena);

        Some(node_id)
    }
    fn parse_parameters_type(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_param_type_id =
            self.arena
                .new_node(Node::new(NodeKind::ParamType, node, &self.source_code));

        let node_param_type = node.named_child(self.get_named_child(&node, 0)?)?;

        let mut cursor = node_param_type.walk();
        for child in node_param_type.named_children(&mut cursor) {
            if child.kind() == "preproc_include_declaration" {
                if let Some(t) = self.parse_preproc_include(&child) {
                    node_param_type_id.append(t, &mut self.arena);
                }
            } else if let Some(t) = self.look_for_preproc_kind(&child) {
                node_param_type_id.append(t, &mut self.arena);
            } else {
                let child_id =
                    self.arena
                        .new_node(Node::new(NodeKind::Name, &child, &self.source_code));
                node_param_type_id.append(child_id, &mut self.arena);
            }
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_param_type_id.append(node_preproc, &mut self.arena);
        }

        Some(node_param_type_id)
    }
    fn parse_error(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::ErrorCst, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }

        // Add name node
        let option_list_node = node.child_by_field_name("option_list").unwrap();
        node_id.append(
            self.parse_type_options_dec(&option_list_node)
                .unwrap_or_else(|| self.new_error_node(&option_list_node)),
            &mut self.arena,
        );

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_match_kind(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::MatchKind, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }

        // Add name node
        let option_list_node = node.child_by_field_name("option_list").unwrap();
        node_id.append(
            self.parse_type_options_dec(&option_list_node)
                .unwrap_or_else(|| self.new_error_node(&option_list_node)),
            &mut self.arena,
        );

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn is_comment(&mut self, node: &tree_sitter::Node) -> bool {
        let kind = node.kind();
        if kind == "line_comment" && kind == "block_comment" {
            return true;
        }
        return false;
    }
    fn get_child(&mut self, node: &tree_sitter::Node, index_inital: u32) -> Option<usize> {
        let mut index = 0;
        let mut index_no_comment = 0;
        while true{
            let n = node.child(index);
            if let Some(x) = n{
                if !self.is_comment(&x){
                    if index_no_comment == index_inital{
                        return Some(index);
                    }
                    index_no_comment += 1;
                }
                index += 1;
            } else{
                return None;
            }
        }
        return None;
    }
    fn get_named_child(&mut self, node: &tree_sitter::Node, index_inital: u32) -> Option<usize> {
        let mut index = 0;
        let mut index_no_comment = 0;
        while true{
            let n = node.named_child(index);
            if let Some(x) = n{
                if !self.is_comment(&x){
                    if index_no_comment == index_inital{
                        return Some(index);
                    }
                    index_no_comment += 1;
                }
                index += 1;
            } else{
                return None;
            }
        }
        return None;
    }

    fn look_for_preproc_kind(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let kind = node.kind();
        if kind == "preproc_include_declaration" {
            return self.parse_preproc_include(&node);
        } else if kind == "preproc_define_declaration" || kind == "preproc_define_declaration_macro"
        {
            return self.parse_preproc_define(&node);
        } else if kind == "preproc_undef_declaration" {
            return self.parse_preproc_undef(&node);
        }
        None
    }

    fn look_for_preproc(&mut self, node: &tree_sitter::Node) -> Vec<NodeId> {
        let mut vec_node: Vec<NodeId> = vec![];

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            debug!("{:?}", child);
            let new_child = match child.kind() {
                "preproc_include_declaration" => self.parse_preproc_include(&child),
                "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                    self.parse_preproc_define(&child)
                }
                "preproc_undef_declaration" => self.parse_preproc_undef(&child),
                _ => None,
            };

            if let Some(new_child) = new_child {
                vec_node.push(new_child);
            }
        }

        vec_node
    }

    fn parse_preproc_include(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::PreprocInclude, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }

        // Add name node
        let node_name = node.child(self.get_child(&node, 3)?).unwrap();
        let name_node =
            self.arena
                .new_node(Node::new(NodeKind::Name, &node_name, &self.source_code));
        node_id.append(name_node, &mut self.arena);

        Some(node_id)
    }
    fn parse_preproc_define(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::PreprocDefine, node, &self.source_code));

        // Add keyword node
        let type_node = node.child_by_field_name("KeyWord").unwrap();
        node_id.append(
            self.arena
                .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
            &mut self.arena,
        );

        // Add name node
        let node_name = node.child_by_field_name("name").unwrap();
        let name_node =
            self.arena
                .new_node(Node::new(NodeKind::Name, &node_name, &self.source_code));
        node_id.append(name_node, &mut self.arena);

        // Add param node
        if let Some(param_node) = node.child_by_field_name("param") {
            node_id.append(
                self.parse_params_define(&param_node)
                    .unwrap_or_else(|| self.new_error_node(&param_node)),
                &mut self.arena,
            );
        }

        // Add body node
        if let Some(node_body) = node.child_by_field_name("body") {
            node_id.append(
                self.parse_value(&node_body)
                    .unwrap_or_else(|| self.new_error_node(&node_body)),
                &mut self.arena,
            );
        }

        // Add body node
        if let Some(node_body) = node.child_by_field_name("body_macro") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::Body, &node_body, &self.source_code)),
                &mut self.arena,
            );
        }

        Some(node_id)
    }

    fn parse_params_define(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let params_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Params, node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            if syntax_child.is_error() {
                params_node_id.append(self.new_error_node(&syntax_child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&syntax_child) {
                params_node_id.append(t, &mut self.arena);
            } else if syntax_child.kind() == "identifier" {
                let param_node_id = self.arena.new_node(Node::new(
                    NodeKind::Param,
                    &syntax_child,
                    &self.source_code,
                ));

                params_node_id.append(param_node_id, &mut self.arena);
            }
        }

        for node_preproc in self.look_for_preproc(&node) {
            params_node_id.append(node_preproc, &mut self.arena);
        }

        Some(params_node_id)
    }
    fn parse_preproc_undef(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::PreprocUndef, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }

        // Add name node
        let node_name = node.named_child(self.get_named_child(&node, 0)?).unwrap();
        let name_node =
            self.arena
                .new_node(Node::new(NodeKind::Name, &node_name, &self.source_code));
        node_id.append(name_node, &mut self.arena);

        Some(node_id)
    }

    fn parse_const_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::ConstantDec, node, &self.source_code));

        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }

        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node, NodeKind::Type, true)
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

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_value(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        // todo-preproc
        fn loop_value(
            node_value: &mut NodeId,
            last_node: NodeId,
            node: &tree_sitter::Node,
            self_v: &mut TreesitterTranslator,
        ) -> NodeId {
            let kind = node.kind();
            //debug!("{}", kind);
            let mut name_node: NodeId = last_node;
            let accept = [
                "non_type_name",
                "type_name",
                "prefixed_type",
                "apply",
                "key",
                "actions",
                "state",
                "entries",
                "type",
                "identifier",
            ];
            if kind != "expression" && kind != "initializer" {
                //debug!("{},{}", kind,accept.contains(&kind));
                match kind {
                    "bool" => {
                        name_node = self_v.arena.new_node(Node::new(
                            NodeKind::Type(Type::Base(BaseType::Bool)),
                            node,
                            &self_v.source_code,
                        ));
                        node_value.append(name_node, &mut self_v.arena);
                    }
                    "integer" => {
                        name_node = self_v.arena.new_node(Node::new(
                            NodeKind::Type(Type::Base(BaseType::Int)),
                            node,
                            &self_v.source_code,
                        ));
                        node_value.append(name_node, &mut self_v.arena);
                    }
                    "string" => {
                        name_node = self_v.arena.new_node(Node::new(
                            NodeKind::Type(Type::Base(BaseType::String)),
                            node,
                            &self_v.source_code,
                        ));
                        node_value.append(name_node, &mut self_v.arena);
                    }
                    "base_type" => {
                        let x = self_v.parse_type_ref(&node, NodeKind::Type, false);
                        if let Some(y) = x {
                            name_node = y.clone();
                            node_value.append(name_node, &mut self_v.arena);
                        }
                    }
                    "null_value" => {
                        name_node = self_v.arena.new_node(Node::new(
                            NodeKind::Type(Type::Base(BaseType::Null)),
                            node,
                            &self_v.source_code,
                        ));
                        node_value.append(name_node, &mut self_v.arena);
                    }
                    _ => {
                        if accept.contains(&kind) {
                            name_node = self_v.arena.new_node(Node::new(
                                NodeKind::Type(Type::Name),
                                node,
                                &self_v.source_code,
                            ));
                            node_value.append(name_node, &mut self_v.arena);
                        } else if kind == "member" || kind == "name" {
                            //debug!("seconde node : {},{:?},{}", kind, node, utils::get_node_text(&node, &selfV.source_code));
                            name_node = self_v.arena.new_node(Node::new(
                                NodeKind::ValueSymbol,
                                node,
                                &self_v.source_code,
                            ));
                            last_node.append(name_node, &mut self_v.arena);
                        } else {
                            debug!("{}:{:?}", kind, node);
                        }
                    }
                }
            } else {
                let mut cursor = node.walk();
                for field_child in node.children(&mut cursor) {
                    if field_child.is_error() {
                    } else if !self_v.is_comment(&field_child) {
                        let mut loop_child = 1;
                        while loop_child != 0 {
                            match field_child.child(loop_child - 1) {
                                Some(x) => {
                                    if self_v.is_comment(&x){} else {
                                        name_node = loop_value(node_value, name_node, &x, self_v);
                                    }
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
            name_node
        }

        let mut node_value =
            self.arena
                .new_node(Node::new(NodeKind::Value, node, &self.source_code));
        let last_node = node_value;

        loop_value(&mut node_value, last_node, node, self);

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
            node,
            &self.source_code,
        ));

        match type_type {
            TypeDecType::TypeDef => {
                // Add annotation node
                if let Some(annotation) = type_kind_node.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }
                // Add keyword node
                if let Some(type_node) = type_kind_node.child_by_field_name("KeyWord") {
                    node_id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }
                let type_node: NodeId = self.parse_type_ref(
                    &type_kind_node.child_by_field_name("type")?,
                    NodeKind::Type,
                    true,
                )?;
                node_id.append(type_node, &mut self.arena);

                let name_node = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &type_kind_node.child_by_field_name("name")?,
                    &self.source_code,
                ));
                node_id.append(name_node, &mut self.arena);
            }
            TypeDecType::HeaderType => {
                // Add annotation node
                if let Some(annotation) = type_kind_node.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }

                // Add keyword node
                if let Some(type_node) = type_kind_node.child_by_field_name("KeyWord") {
                    node_id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }

                let name_node = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &type_kind_node.child_by_field_name("name")?,
                    &self.source_code,
                ));
                node_id.append(name_node, &mut self.arena);

                if let Some(paramters) = type_kind_node.child_by_field_name("parameters_type") {
                    node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
                }
                if let Some(x) = type_kind_node.child_by_field_name("field_list") {
                    node_id.append(
                        self.parse_type_fields_dec(&x)
                            .unwrap_or_else(|| self.new_error_node(&x)),
                        &mut self.arena,
                    );
                }
            }
            TypeDecType::HeaderUnion => {
                // Add annotation node
                if let Some(annotation) = type_kind_node.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }

                // Add keyword node
                if let Some(type_node) = type_kind_node.child_by_field_name("KeyWord") {
                    node_id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }

                let name_node = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &type_kind_node.child_by_field_name("name")?,
                    &self.source_code,
                ));
                node_id.append(name_node, &mut self.arena);

                if let Some(paramters) = type_kind_node.child_by_field_name("parameters_type") {
                    node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
                }
                if let Some(x) = type_kind_node.child_by_field_name("field_list") {
                    node_id.append(
                        self.parse_type_fields_dec(&x)
                            .unwrap_or_else(|| self.new_error_node(&x)),
                        &mut self.arena,
                    );
                }
            }
            TypeDecType::Struct => {
                // Add annotation node
                if let Some(annotation) = type_kind_node.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }
                // Add keyword node
                if let Some(type_node) = type_kind_node.child_by_field_name("KeyWord") {
                    node_id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }

                let name_node = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &type_kind_node.child_by_field_name("name")?,
                    &self.source_code,
                ));
                node_id.append(name_node, &mut self.arena);

                if let Some(paramters) = type_kind_node.child_by_field_name("parameters_type") {
                    node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
                }
                if let Some(x) = type_kind_node.child_by_field_name("field_list") {
                    node_id.append(
                        self.parse_type_fields_dec(&x)
                            .unwrap_or_else(|| self.new_error_node(&x)),
                        &mut self.arena,
                    );
                }
            }
            TypeDecType::Enum => {
                // Add annotation node
                if let Some(annotation) = type_kind_node.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }
                // Add keyword node
                if let Some(type_node) = type_kind_node.child_by_field_name("KeyWord") {
                    node_id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }
                if let Some(x) = type_kind_node.child_by_field_name("type") {
                    node_id.append(
                        self.parse_type_ref(&x, NodeKind::Type, true)
                            .unwrap_or_else(|| self.new_error_node(&x)),
                        &mut self.arena,
                    );
                }
                let name_node = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &type_kind_node.child_by_field_name("name")?,
                    &self.source_code,
                ));
                node_id.append(name_node, &mut self.arena);
                if let Some(x) = type_kind_node.child_by_field_name("option_list") {
                    node_id.append(
                        self.parse_type_options_dec(&x)
                            .unwrap_or_else(|| self.new_error_node(&x)),
                        &mut self.arena,
                    );
                }
            }
            TypeDecType::Parser => {
                // Add annotation node
                if let Some(annotation) = type_kind_node.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }
                // Add keyword node
                if let Some(type_node) = type_kind_node.child_by_field_name("KeyWord") {
                    node_id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }
                let name_node = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &type_kind_node.child_by_field_name("name")?,
                    &self.source_code,
                ));
                node_id.append(name_node, &mut self.arena);
                if let Some(paramters) = type_kind_node.child_by_field_name("parameters_type") {
                    node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
                }
                if let Some(params_syntax_node) = type_kind_node.child_by_field_name("parameters") {
                    let params_node_id = self
                        .parse_params(&params_syntax_node)
                        .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
                    node_id.append(params_node_id, &mut self.arena);
                }
            }
            TypeDecType::Control => {
                // Add annotation node
                if let Some(annotation) = node.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }
                // Add keyword node
                if let Some(type_node) = type_kind_node.child_by_field_name("KeyWord") {
                    node_id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }
                let name_node = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &type_kind_node.child_by_field_name("name")?,
                    &self.source_code,
                ));
                node_id.append(name_node, &mut self.arena);
                if let Some(paramters) = type_kind_node.child_by_field_name("parameters_type") {
                    node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
                }
                if let Some(params_syntax_node) = type_kind_node.child_by_field_name("parameters") {
                    let params_node_id = self
                        .parse_params(&params_syntax_node)
                        .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
                    node_id.append(params_node_id, &mut self.arena);
                }
            }
            TypeDecType::Package => {
                // Add annotation node
                if let Some(annotation) = type_kind_node.child_by_field_name("annotation") {
                    node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }
                // Add keyword node
                if let Some(type_node) = type_kind_node.child_by_field_name("KeyWord") {
                    node_id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }
                if let Some(name_node) = type_kind_node.child_by_field_name("name") {
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
                }
                if let Some(paramters) = type_kind_node.child_by_field_name("parameters_type") {
                    node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
                } else if let Some(params_syntax_node) =
                    type_kind_node.child_by_field_name("parameters")
                {
                    node_id.append(
                        self.parse_params(&params_syntax_node)
                            .unwrap_or_else(|| self.new_error_node(&params_syntax_node)),
                        &mut self.arena,
                    );
                }
            }
        }
        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }
        for node_preproc in self.look_for_preproc(&type_kind_node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_type_fields_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let fields_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Fields, node, &self.source_code));

        let mut cursor = node.walk();
        for field_child in node.named_children(&mut cursor) {
            if field_child.is_error() {
                fields_node_id.append(self.new_error_node(&field_child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&field_child) {
                fields_node_id.append(t, &mut self.arena);
            } else if field_child.kind() == "struct_field" {
                let field_node_id = self.arena.new_node(Node::new(
                    NodeKind::Field,
                    &field_child,
                    &self.source_code,
                ));
                // Add annotation node
                if let Some(annotation) = field_child.child_by_field_name("annotation") {
                    field_node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }

                // Add name node
                if let Some(x) = field_child.child_by_field_name("name") {
                    field_node_id.append(
                        self.arena
                            .new_node(Node::new(NodeKind::Name, &x, &self.source_code)),
                        &mut self.arena,
                    );
                }

                // Add type node
                if let Some(x) = field_child.child_by_field_name("type") {
                    field_node_id.append(
                        self.parse_type_ref(&x, NodeKind::Type, true)
                            .unwrap_or_else(|| self.new_error_node(&x)),
                        &mut self.arena,
                    );
                }

                fields_node_id.append(field_node_id, &mut self.arena);
            }
        }
        Some(fields_node_id)
    }

    fn parse_type_options_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let options_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Options, node, &self.source_code));

        let mut cursor = node.walk();
        for option_child in node.named_children(&mut cursor) {
            let new_node_id = if option_child.is_error() {
                Some(self.new_error_node(&option_child))
            } else if let Some(t) = self.look_for_preproc_kind(&option_child) {
                Some(t)
            } else if !self.is_comment(&option_child) {
                //let node_text = utils::get_node_text(&option_child, &self.source_code);
                //let text = node_text.as_str().trim();
                debug!("{:?}", option_child); // todo-issue

                let option_node_id = self.arena.new_node(Node::new(
                    NodeKind::Option,
                    &option_child,
                    &self.source_code,
                ));

                // Add name node
                option_node_id.append(
                    self.arena.new_node(Node::new(
                        NodeKind::Name,
                        &option_child,
                        &self.source_code,
                    )),
                    &mut self.arena,
                );

                Some(option_node_id)
            } else{
                None
            };
            
            if let Some(x) = new_node_id{
                options_node_id.append(x, &mut self.arena);
            }
        }
        Some(options_node_id)
    }

    fn parse_type_ref(
        &mut self,
        node: &tree_sitter::Node,
        n_kind: fn(Type) -> NodeKind,
        child_bool: bool,
    ) -> Option<NodeId> {
        let child: tree_sitter::Node<'_>;
        if child_bool {
            child = node.named_child(self.get_named_child(&node, 0)?)?;
        } else {
            child = node.clone();
        }

        let node_id: Option<NodeId> = match child.kind() {
            "base_type" => {
                let node_text = utils::get_node_text(&child, &self.source_code);
                let text = node_text.as_str().trim();

                match text {
                    "bool" => Some(self.arena.new_node(Node::new(
                        n_kind(Type::Base(BaseType::Bool)),
                        node,
                        &self.source_code,
                    ))),
                    "int" => Some(self.arena.new_node(Node::new(
                        n_kind(Type::Base(BaseType::Int)),
                        node,
                        &self.source_code,
                    ))),
                    "bit" => Some(self.arena.new_node(Node::new(
                        n_kind(Type::Base(BaseType::Bit)),
                        node,
                        &self.source_code,
                    ))),
                    "string" => Some(self.arena.new_node(Node::new(
                        n_kind(Type::Base(BaseType::String)),
                        node,
                        &self.source_code,
                    ))),
                    "varbit" => Some(self.arena.new_node(Node::new(
                        n_kind(Type::Base(BaseType::Varbit)),
                        node,
                        &self.source_code,
                    ))),
                    "error" => Some(self.arena.new_node(Node::new(
                        n_kind(Type::Base(BaseType::Error)),
                        node,
                        &self.source_code,
                    ))),
                    "match_kind" => Some(self.arena.new_node(Node::new(
                        n_kind(Type::Base(BaseType::MatchKind)),
                        node,
                        &self.source_code,
                    ))),
                    _ => {
                        let child_child = child.named_child(self.get_named_child(&child, 0)?).unwrap();
                        let mut node_id: Option<NodeId> = None;
                        if child_child.kind() == "integer" {
                            let size = Some(
                                utils::get_node_text(&child_child, &self.source_code)
                                    .parse::<u32>()
                                    .unwrap(),
                            );

                            if text.starts_with("int") {
                                node_id = Some(self.arena.new_node(Node::new(
                                    n_kind(Type::Base(BaseType::SizedInt(size))),
                                    node,
                                    &self.source_code,
                                )));
                            } else if text.starts_with("bit") {
                                node_id = Some(self.arena.new_node(Node::new(
                                    n_kind(Type::Base(BaseType::SizedBit(size))),
                                    node,
                                    &self.source_code,
                                )));
                            } else if text.starts_with("varbit") {
                                node_id = Some(self.arena.new_node(Node::new(
                                    n_kind(Type::Base(BaseType::SizedVarbit(size))),
                                    node,
                                    &self.source_code,
                                )));
                            }
                        } else {
                            let node_return: NodeId;
                            if text.starts_with("int") {
                                node_return = self.arena.new_node(Node::new(
                                    n_kind(Type::Base(BaseType::SizedInt(None))),
                                    node,
                                    &self.source_code,
                                ));
                            } else if text.starts_with("bit") {
                                node_return = self.arena.new_node(Node::new(
                                    n_kind(Type::Base(BaseType::SizedBit(None))),
                                    node,
                                    &self.source_code,
                                ));
                            } else if text.starts_with("varbit") {
                                node_return = self.arena.new_node(Node::new(
                                    n_kind(Type::Base(BaseType::SizedVarbit(None))),
                                    node,
                                    &self.source_code,
                                ));
                            } else {
                                node_return = self.arena.new_node(Node::new(
                                    n_kind(Type::Base(BaseType::Error)),
                                    node,
                                    &self.source_code,
                                ));
                            }
                            if child_child.kind() == "define_symbol" {
                                node_return.append(
                                    self.arena.new_node(Node::new(
                                        NodeKind::DefineSymbol,
                                        &child_child,
                                        &self.source_code,
                                    )),
                                    &mut self.arena,
                                );
                            } else if child_child.kind() == "expression" {
                                node_return
                                    .append(self.parse_value(&child_child)?, &mut self.arena);
                            }

                            node_id = Some(node_return);
                        }
                        if let Some(t) = node_id {
                            for node_preproc in self.look_for_preproc(&child_child) {
                                t.append(node_preproc, &mut self.arena);
                            }
                            Some(t)
                        } else {
                            node_id
                        }
                    }
                }
            }
            "type_name" => Some(self.arena.new_node(Node::new(
                n_kind(Type::Name),
                node,
                &self.source_code,
            ))),
            "specialized_type" => Some(self.arena.new_node(Node::new(
                n_kind(Type::Specialized),
                node,
                &self.source_code,
            ))),
            "header_stack_type" => Some(self.arena.new_node(Node::new(
                n_kind(Type::Header),
                node,
                &self.source_code,
            ))),
            "tuple_type" => Some(self.arena.new_node(Node::new(
                n_kind(Type::Tuple),
                node,
                &self.source_code,
            ))),
            _ => None,
        };

        if let Some(node_id_t) = node_id {
            for node_preproc in self.look_for_preproc(&node) {
                node_id_t.append(node_preproc, &mut self.arena);
            }
            if child_bool {
                for node_preproc in self.look_for_preproc(&child) {
                    node_id_t.append(node_preproc, &mut self.arena);
                }
            }
            return Some(node_id_t);
        }

        node_id
    }

    fn parse_parser(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::ParserDec, node, &self.source_code));

        let declaration_body = &node.child_by_field_name("declaration")?;
        let key_word_node_id = self.arena.new_node(Node::new(
            NodeKind::KeyWord,
            &declaration_body.child_by_field_name("KeyWord")?,
            &self.source_code,
        ));
        node_id.append(key_word_node_id, &mut self.arena);

        let name_node_id = self.arena.new_node(Node::new(
            NodeKind::Name,
            &declaration_body.child_by_field_name("name")?,
            &self.source_code,
        ));
        node_id.append(name_node_id, &mut self.arena);

        if let Some(paramters) = declaration_body.child_by_field_name("parameters_type") {
            node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
        }
        if let Some(annotation) = declaration_body.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }

        let params_syntax_node = declaration_body.child_by_field_name("parameters").unwrap();
        let params_node_id = self
            .parse_params(&params_syntax_node)
            .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
        node_id.append(params_node_id, &mut self.arena);

        let body_syntax_node = &node.child_by_field_name("body")?;
        let body_node_id = self.arena.new_node(Node::new(
            NodeKind::Body,
            body_syntax_node,
            &self.source_code,
        ));

        let mut cursor = body_syntax_node.walk();
        for syntax_child in body_syntax_node.named_children(&mut cursor) {
            let child_node_id = if syntax_child.is_error() {
                Some(self.new_error_node(&syntax_child))
            } else {
                match syntax_child.kind() {
                    // _parser_local_element
                    "constant_declaration" => self.parse_const_dec(&syntax_child),
                    "variable_declaration" => self.parse_var_dec(&syntax_child),
                    "instantiation" => self.instantiation(&syntax_child),
                    "value_set_declaration" => self.parse_val_set(&syntax_child),

                    // parser_state
                    "parser_state" => self.parse_state(&syntax_child),

                    "preproc_include_declaration" => self.parse_preproc_include(&syntax_child),
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        self.parse_preproc_define(&syntax_child)
                    }
                    "preproc_undef_declaration" => self.parse_preproc_undef(&syntax_child),
                    _ => None,
                }
            };

            if let Some(id) = child_node_id {
                body_node_id.append(id, &mut self.arena);
            }
        }
        node_id.append(body_node_id, &mut self.arena);

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }
        for node_preproc in self.look_for_preproc(&declaration_body) {
            node_id.append(node_preproc, &mut self.arena);
        }
        for node_preproc in self.look_for_preproc(&body_syntax_node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_control(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::ControlDec, node, &self.source_code));

        let declaration_body = node.child_by_field_name("declaration")?;
        let key_word_node_id = self.arena.new_node(Node::new(
            NodeKind::KeyWord,
            &declaration_body.child_by_field_name("KeyWord")?,
            &self.source_code,
        ));
        node_id.append(key_word_node_id, &mut self.arena);

        let name_node_id = self.arena.new_node(Node::new(
            NodeKind::Name,
            &declaration_body.child_by_field_name("name")?,
            &self.source_code,
        ));
        node_id.append(name_node_id, &mut self.arena);

        if let Some(paramters) = declaration_body.child_by_field_name("parameters_type") {
            node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
        }
        if let Some(annotation) = declaration_body.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }

        let params_syntax_node = declaration_body.child_by_field_name("parameters").unwrap();
        let params_node_id = self
            .parse_params(&params_syntax_node)
            .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
        node_id.append(params_node_id, &mut self.arena);

        let body_syntax_node = &node.child_by_field_name("body")?;
        let body_node_id = self.arena.new_node(Node::new(
            NodeKind::Body,
            body_syntax_node,
            &self.source_code,
        ));

        let mut cursor = body_syntax_node.walk();
        for syntax_child in body_syntax_node.named_children(&mut cursor) {
            let child_node_id = if syntax_child.is_error() {
                Some(self.new_error_node(&syntax_child))
            } else {
                match syntax_child.kind() {
                    "constant_declaration" => self.parse_const_dec(&syntax_child),
                    "variable_declaration" => self.parse_var_dec(&syntax_child),
                    "instantiation" => self.instantiation(&syntax_child),
                    "action_declaration" => self.parse_control_action(&syntax_child),
                    "table_declaration" => self.parse_control_table(&syntax_child),

                    "block_statement" => self.parse_block(&syntax_child),

                    "preproc_include_declaration" => self.parse_preproc_include(&syntax_child),
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        self.parse_preproc_define(&syntax_child)
                    }
                    "preproc_undef_declaration" => self.parse_preproc_undef(&syntax_child),
                    _ => None,
                }
            };

            if let Some(id) = child_node_id {
                body_node_id.append(id, &mut self.arena);
            }
        }
        node_id.append(body_node_id, &mut self.arena);

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }
        for node_preproc in self.look_for_preproc(&declaration_body) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_params(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let params_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Params, node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            //debug!("a,{:?}",syntax_child);
            if syntax_child.is_error() {
                params_node_id.append(self.new_error_node(&syntax_child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&syntax_child) {
                params_node_id.append(t, &mut self.arena);
            } else if syntax_child.kind() == "parameter" {
                let param_node_id = self.arena.new_node(Node::new(
                    NodeKind::Param,
                    &syntax_child,
                    &self.source_code,
                ));
                // Add annotation node
                if let Some(annotation) = syntax_child.child_by_field_name("annotation") {
                    param_node_id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }

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
                    self.parse_type_ref(&type_syntax_node, NodeKind::Type, true)
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

                params_node_id.append(param_node_id, &mut self.arena);
            }
            //debug!("a,{:?}",new_node_id);
        }

        Some(params_node_id)
    }
    fn parse_args(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let params_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Args, node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            //debug!("a,{:?}",syntax_child);
            if syntax_child.is_error() {
                params_node_id.append(self.new_error_node(&syntax_child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&syntax_child) {
                params_node_id.append(t, &mut self.arena);
            } else if syntax_child.kind() == "argument" {
                let param_node_id =
                    self.arena
                        .new_node(Node::new(NodeKind::Arg, &syntax_child, &self.source_code));

                // Add name node
                if let Some(name_node) = syntax_child.child_by_field_name("name") {
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

                params_node_id.append(param_node_id, &mut self.arena);
            }
            //debug!("a,{:?}",new_node_id);
        }

        Some(params_node_id)
    }
    fn parse_obj_initializer(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let obj_node_id = self
            .arena
            .new_node(Node::new(NodeKind::Obj, node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            let child_node_id = if syntax_child.is_error() {
                Some(self.new_error_node(&syntax_child))
            } else {
                match syntax_child.kind() {
                    "function_declaration" => self.function_declaration(&syntax_child),
                    "instantiation" => self.instantiation(&syntax_child),

                    "preproc_include_declaration" => self.parse_preproc_include(&syntax_child),
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        self.parse_preproc_define(&syntax_child)
                    }
                    "preproc_undef_declaration" => self.parse_preproc_undef(&syntax_child),
                    _ => None,
                }
            };
            if let Some(child_node) = child_node_id {
                obj_node_id.append(child_node, &mut self.arena);
            }
        }

        Some(obj_node_id)
    }
    fn function_declaration(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let fn_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Function, node, &self.source_code));

        let mut index = self.get_named_child(&node, 0)?;

        if let Some(x) = self.function_prototype(&node.named_child(index)?) {
            fn_node_id.append(x, &mut self.arena);
        }

        index = self.get_named_child(&node, 1)?;

        if let Some(x) = self.parse_block(&node.named_child(index)?) {
            fn_node_id.append(x, &mut self.arena);
        }

        for node_preproc in self.look_for_preproc(&node) {
            fn_node_id.append(node_preproc, &mut self.arena);
        }

        Some(fn_node_id)
    }
    fn function_prototype(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let fn_node_id =
            self.arena
                .new_node(Node::new(NodeKind::FunctionName, node, &self.source_code));

        if let Some(paramters) = node.child_by_field_name("parameters_type") {
            fn_node_id.append(self.parse_parameters_type(&paramters)?, &mut self.arena);
        }
        let type_node = node.child_by_field_name("type").unwrap();
        if type_node.kind() == "type_ref" {
            // TODO
            fn_node_id.append(
                self.parse_type_ref(&type_node, NodeKind::Type, true)
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

        if let Some(params_syntax_node) = node.child_by_field_name("parameters_list") {
            fn_node_id.append(
                self.parse_params(&params_syntax_node)
                    .unwrap_or_else(|| self.new_error_node(&params_syntax_node)),
                &mut self.arena,
            );
        }

        for node_preproc in self.look_for_preproc(&node) {
            fn_node_id.append(node_preproc, &mut self.arena);
        }
        for node_preproc in self.look_for_preproc(&type_node) {
            fn_node_id.append(node_preproc, &mut self.arena);
        }

        Some(fn_node_id)
    }
    fn parse_block(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let block_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Block, node, &self.source_code));

        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            block_node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }
        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            let child_node_id = if syntax_child.is_error() {
                Some(self.new_error_node(&syntax_child))
            } else {
                match syntax_child.kind() {
                    "constant_declaration" => self.parse_const_dec(&syntax_child),
                    "variable_declaration" => self.parse_var_dec(&syntax_child),
                    "assignment_or_method_call_statement" => {
                        self.parse_state_assignment(&syntax_child)
                    }
                    "direct_application" => self.parse_state_direct(&syntax_child),
                    "conditional_statement" => self.parse_state_conditional(&syntax_child),
                    "empty_statement" => None,
                    "block_statement" => self.parse_block(&syntax_child),

                    "exit_statement" => None,
                    "return_statement" => self.return_statement(&syntax_child),
                    "switch_statement" => self.switch_statement(&syntax_child),

                    "preproc_include_declaration" => self.parse_preproc_include(&syntax_child),
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        self.parse_preproc_define(&syntax_child)
                    }
                    "preproc_undef_declaration" => self.parse_preproc_undef(&syntax_child),
                    _ => None,
                }
            };
            if let Some(child_node) = child_node_id {
                block_node_id.append(child_node, &mut self.arena);
            }
        }

        for node_preproc in self.look_for_preproc(&node) {
            block_node_id.append(node_preproc, &mut self.arena);
        }

        Some(block_node_id)
    }

    fn parse_direction(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        // not preproc
        let dir = match utils::get_node_text(node, &self.source_code).as_str() {
            "in" => Direction::In,
            "out" => Direction::Out,
            "inout" => Direction::InOut,
            _ => {
                //debug!("None");
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

        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }
        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node, NodeKind::Type, true)
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
        if let Some(value_node) = node.child_by_field_name("value") {
            node_id.append(
                self.parse_value(&value_node)
                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                &mut self.arena,
            );
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_name_assignment(&mut self, node_t: &tree_sitter::Node) -> Option<NodeId> {
        let node_first = *node_t;
        let mut last_node: Option<NodeId> = None;
        if let Some(mut node) = node_t.child(self.get_child(&node_t, 0)?) {
            let mut bool = true;
            while bool {
                match node.kind() {
                    "prefixed_non_type_name" => {
                        let index = self.get_named_child(&node, 0)?;
                        let node_id = self.arena.new_node(Node::new(
                            NodeKind::Type(Type::Name),
                            &node.named_child(index).unwrap(),
                            &self.source_code,
                        ));
                        if let Some(new_child) = last_node {
                            node_id.append(new_child, &mut self.arena);
                        }
                        last_node = Some(node_id);
                        bool = false;
                    }
                    "lvalue_dot" => {
                        let node_id = self.arena.new_node(Node::new(
                            NodeKind::StatementDot,
                            &node,
                            &self.source_code,
                        ));
                        let mut t = node.named_child(self.get_named_child(&node, 1)?).unwrap();
                        t = t.named_child(self.get_named_child(&t, 0)?).unwrap();
                        
                        let node_id_dot = self.arena.new_node(Node::new(
                            NodeKind::ValueSymbol,
                            &t,
                            &self.source_code,
                        ));

                        if let Some(new_child) = last_node {
                            node_id.append(new_child, &mut self.arena);
                        }
                        last_node = Some(node_id);

                        node_id.append(node_id_dot, &mut self.arena);

                        t = node.named_child(self.get_named_child(&t, 0)?).unwrap();
                        if let Some(x) = t.named_child(self.get_named_child(&t, 0)?) {
                            node = x
                        } else {
                            bool = false;
                        }
                    }
                    "lvalue_bra" => {
                        let node_id = self.arena.new_node(Node::new(
                            NodeKind::StatementExpr,
                            &node,
                            &self.source_code,
                        ));
                        let mut t = node.named_child(self.get_named_child(&node, 1)?).unwrap();
                        let node_id_expr = self
                            .parse_value(&t)
                            .unwrap_or_else(|| self.new_error_node(&t));

                        if let Some(new_child) = last_node {
                            node_id.append(new_child, &mut self.arena);
                        }
                        last_node = Some(node_id);

                        node_id.append(node_id_expr, &mut self.arena);

                        t = node.named_child(self.get_named_child(&node, 0)?).unwrap();
                        if let Some(x) = t.named_child(self.get_named_child(&t, 0)?) {
                            node = x
                        } else {
                            bool = false;
                        }
                    }
                    "lvalue_double_dot" => {
                        let node_id = self.arena.new_node(Node::new(
                            NodeKind::StatementDouble,
                            &node,
                            &self.source_code,
                        ));
                        let node_child_1 = node.named_child(self.get_named_child(&node, 1)?).unwrap();
                        let node_child_2 = node.named_child(self.get_named_child(&node, 2)?).unwrap();
                        let node_id_expr1 = self
                            .parse_value(&node_child_1)
                            .unwrap_or_else(|| self.new_error_node(&node_child_1));
                        let node_id_expr2 = self
                            .parse_value(&node_child_2)
                            .unwrap_or_else(|| self.new_error_node(&node_child_2));

                        if let Some(new_child) = last_node {
                            node_id.append(new_child, &mut self.arena);
                        }
                        last_node = Some(node_id);

                        node_id.append(node_id_expr1, &mut self.arena);
                        node_id.append(node_id_expr2, &mut self.arena);

                        let mut t = node.named_child(self.get_named_child(&node, 0)?).unwrap();
                        if let Some(x) = t.named_child(self.get_named_child(&t, 0)?) {
                            node = x
                        } else {
                            bool = false;
                        }
                    }
                    "preproc_include_declaration" => {
                        if let Some(t) = self.parse_preproc_include(&node) {
                            if let Some(x) = last_node {
                                x.append(t, &mut self.arena);
                            } else {
                                last_node = Some(t);
                            }
                        }
                    }
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        if let Some(t) = self.parse_preproc_define(&node) {
                            if let Some(x) = last_node {
                                x.append(t, &mut self.arena);
                            } else {
                                last_node = Some(t);
                            }
                        }
                    }
                    "preproc_undef_declaration" => {
                        if let Some(t) = self.parse_preproc_undef(&node) {
                            if let Some(x) = last_node {
                                x.append(t, &mut self.arena);
                            } else {
                                last_node = Some(t);
                            }
                        }
                    }
                    _ => {
                        bool = false;
                    }
                }
            }
        }

        let name_node = self.arena.new_node(Node::new(
            NodeKind::NameStatement,
            &node_first,
            &self.source_code,
        ));
        if let Some(new_child) = last_node {
            name_node.append(new_child, &mut self.arena);
        }

        for node_preproc in self.look_for_preproc(&node_first) {
            name_node.append(node_preproc, &mut self.arena);
        }

        Some(name_node)
    }
    fn parse_state_assignment(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Assignment, node, &self.source_code));

        // Add name node
        let name_node = &node.child_by_field_name("name").unwrap();
        node_id.append(
            self.parse_name_assignment(name_node)
                .unwrap_or_else(|| self.new_error_node(name_node)),
            &mut self.arena,
        );

        // Add value node
        if let Some(value_node) = node.child_by_field_name("expression") {
            node_id.append(
                self.parse_value(&value_node)
                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                &mut self.arena,
            );
        }

        if let Some(params_syntax_node) = node.child_by_field_name("parameters") {
            let params_node_id = self
                .parse_args(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }
        if let Some(param_list) = node.child_by_field_name("type") {
            let params_node_id =
                self.arena
                    .new_node(Node::new(NodeKind::ParamsList, node, &self.source_code));

            let mut cursor = param_list.walk();
            for syntax_child in param_list.named_children(&mut cursor) {
                let child_node_id = if syntax_child.is_error() {
                    Some(self.new_error_node(&syntax_child))
                } else if let Some(t) = self.look_for_preproc_kind(&syntax_child) {
                    Some(t)
                } else {
                    match syntax_child.named_child(self.get_named_child(&syntax_child, 0)?)?.kind() {
                        "type_ref" => self.parse_type_ref(&syntax_child, NodeKind::TypeList, true),
                        "non_type_name" => Some(self.arena.new_node(Node::new(
                            NodeKind::TypeList(Type::Name),
                            node,
                            &self.source_code,
                        ))),

                        "preproc_include_declaration" => self.parse_preproc_include(&syntax_child),
                        "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                            self.parse_preproc_define(&syntax_child)
                        }
                        "preproc_undef_declaration" => self.parse_preproc_undef(&syntax_child),
                        _ => Some(self.new_error_node(&syntax_child)),
                    }
                };
                if let Some(child_node) = child_node_id {
                    params_node_id.append(child_node, &mut self.arena);
                }
            }

            node_id.append(params_node_id, &mut self.arena);
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_state_direct(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id: NodeId = self.arena.new_node(Node::new(
            NodeKind::DirectApplication,
            node,
            &self.source_code,
        ));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }

        let type_name: Type;
        let name_type: tree_sitter::Node;
        if let Some(x) = node.child_by_field_name("name") {
            name_type = x;
            type_name = Type::Name;
        } else {
            name_type = node.child_by_field_name("specialized")?;
            type_name = Type::Specialized;
        }

        node_id.append(
            self.arena.new_node(Node::new(
                NodeKind::Type(type_name),
                &name_type,
                &self.source_code,
            )),
            &mut self.arena,
        );

        if let Some(params_syntax_node) = node.child_by_field_name("args") {
            let params_node_id = self
                .parse_args(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_state_block(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id: NodeId =
            self.arena
                .new_node(Node::new(NodeKind::ParserBlock, node, &self.source_code));

        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }
        let body = node.child_by_field_name("body")?;
        let mut cursor = body.walk();
        for body_child in body.named_children(&mut cursor) {
            let child_node_id = if body_child.is_error() {
                Some(self.new_error_node(&body_child))
            } else {
                match body_child.kind() {
                    "assignment_or_method_call_statement" => {
                        self.parse_state_assignment(&body_child)
                    }
                    "direct_application" => self.parse_state_direct(&body_child),
                    "parser_block_statement" => self.parse_state_block(&body_child),
                    "constant_declaration" => self.parse_const_dec(&body_child),
                    "variable_declaration" => self.parse_var_dec(&body_child),
                    "empty_statement" => None,
                    "conditional_statement" => self.parse_state_conditional(&body_child),

                    "preproc_include_declaration" => self.parse_preproc_include(&body_child),
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        self.parse_preproc_define(&body_child)
                    }
                    "preproc_undef_declaration" => self.parse_preproc_undef(&body_child),
                    _ => None,
                }
            };

            if let Some(id) = child_node_id {
                node_id.append(id, &mut self.arena);
            }
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_state_conditional(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::Conditional, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }
        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWordEnd") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }
        // Add value node
        if let Some(value_node) = node.child_by_field_name("expression") {
            node_id.append(
                self.parse_value(&value_node)
                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                &mut self.arena,
            );
        }

        // Add body if
        let node_if = node.child_by_field_name("bodyIf").unwrap();
        node_id.append(
            self.fn_statement(node_if, NodeKind::BodyIf)?,
            &mut self.arena,
        );

        // Add body if
        if let Some(node_else) = node.child_by_field_name("bodyElse") {
            node_id.append(
                self.fn_statement(node_else, NodeKind::BodyElse)?,
                &mut self.arena,
            );
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn fn_statement(&mut self, node: tree_sitter::Node<'_>, type_node: NodeKind) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(type_node, &node, &self.source_code));

        let mut cursor = node.walk();
        for body_child in node.named_children(&mut cursor) {
            let child_node_id = if body_child.is_error() {
                Some(self.new_error_node(&body_child))
            } else {
                match body_child.kind() {
                    "assignment_or_method_call_statement" => {
                        self.parse_state_assignment(&body_child)
                    }
                    "direct_application" => self.parse_state_direct(&body_child),
                    "conditional_statement" => self.parse_state_conditional(&body_child),
                    "empty_statement" => None,
                    "block_statement" => self.parse_block(&body_child),

                    "exit_statement" => None,
                    "return_statement" => self.return_statement(&body_child),
                    "switch_statement" => self.switch_statement(&body_child),

                    "preproc_include_declaration" => self.parse_preproc_include(&body_child),
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        self.parse_preproc_define(&body_child)
                    }
                    "preproc_undef_declaration" => self.parse_preproc_undef(&body_child),
                    _ => None,
                }
            };

            if let Some(id) = child_node_id {
                node_id.append(id, &mut self.arena);
            }
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn return_statement(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Return, node, &self.source_code));
        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }
        // Add value node
        if let Some(value_node) = node.child_by_field_name("expression") {
            node_id.append(
                self.parse_value(&value_node)
                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                &mut self.arena,
            );
            return Some(node_id);
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        None
    }
    fn switch_statement(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Switch, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }

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
            if body_child.is_error() {
                node_id.append(self.new_error_node(&body_child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&body_child) {
                node_id.append(t, &mut self.arena);
            } else if body_child.kind() == "switch_case" {
                let label: NodeId =
                    self.arena
                        .new_node(Node::new(NodeKind::SwitchLabel, node, &self.source_code));
                let n = body_child.child_by_field_name("name")?;
                label.append(
                    self.parse_value(&n)
                        .unwrap_or_else(|| self.new_error_node(&n)),
                    &mut self.arena,
                );

                if let Some(value_node) = node.child_by_field_name("value") {
                    label.append(
                        self.parse_block(&value_node)
                            .unwrap_or_else(|| self.new_error_node(&value_node)),
                        &mut self.arena,
                    );
                }
                node_id.append(label, &mut self.arena);
            }
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_val_set(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::ValueSet, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }
        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }

        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node, NodeKind::Type, true)
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

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn instantiation(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::Instantiation, node, &self.source_code));

        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }
        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node, NodeKind::Type, true)
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

        if let Some(params_syntax_node) = node.child_by_field_name("args") {
            let params_node_id = self
                .parse_args(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }

        if let Some(params_syntax_node) = node.child_by_field_name("obj") {
            let params_node_id = self
                .parse_obj_initializer(&params_syntax_node)
                .unwrap_or_else(|| self.new_error_node(&params_syntax_node));
            node_id.append(params_node_id, &mut self.arena);
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_control_action(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::ControlAction, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }
        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        if let Some(params_syntax_node) = node.child_by_field_name("parameters") {
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

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_control_table(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id: NodeId =
            self.arena
                .new_node(Node::new(NodeKind::ControlTable, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }
        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        let table_syntax_node = node.child_by_field_name("table").unwrap();
        let table_node_id = self.arena.new_node(Node::new(
            NodeKind::Table,
            &table_syntax_node,
            &self.source_code,
        ));
        let mut cursor = table_syntax_node.walk();
        for table_child in table_syntax_node.named_children(&mut cursor) {
            let mut child_node_id: Option<NodeId> = None;
            if table_child.is_error() {
                child_node_id = Some(self.new_error_node(&table_child));
            } else {
                match table_child.kind() {
                    "keys_table" => {
                        let keys = table_child.child_by_field_name("keys").unwrap();
                        let keys_node_id = self.arena.new_node(Node::new(
                            NodeKind::Keys,
                            &table_child,
                            &self.source_code,
                        ));
                        let mut cursor = keys.walk();
                        for keys_child in keys.named_children(&mut cursor) {
                            if keys_child.is_error() {
                                keys_node_id
                                    .append(self.new_error_node(&keys_child), &mut self.arena);
                            } else if let Some(t) = self.look_for_preproc_kind(&keys_child) {
                                keys_node_id.append(t, &mut self.arena);
                            } else if !self.is_comment(&keys_child) {
                                // Add name node
                                let key_node_id = self.arena.new_node(Node::new(
                                    NodeKind::Key,
                                    &keys_child,
                                    &self.source_code,
                                ));
                                // Add annotation node
                                if let Some(annotation) =
                                    keys_child.child_by_field_name("annotation")
                                {
                                    key_node_id.append(
                                        self.parse_annotation(&annotation)
                                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                                        &mut self.arena,
                                    );
                                }

                                // Add name node
                                let name_node = self.arena.new_node(Node::new(
                                    NodeKind::Name,
                                    &keys_child.child_by_field_name("name").unwrap(),
                                    &self.source_code,
                                ));
                                key_node_id.append(name_node, &mut self.arena);

                                // Add value node
                                let value_node =
                                    keys_child.child_by_field_name("expression").unwrap();
                                key_node_id.append(
                                    self.parse_value(&value_node)
                                        .unwrap_or_else(|| self.new_error_node(&value_node)),
                                    &mut self.arena,
                                );

                                keys_node_id.append(key_node_id, &mut self.arena);
                            }
                        }
                        child_node_id = Some(keys_node_id);
                    }
                    "action_table" => {
                        let actions = table_child.child_by_field_name("actions").unwrap();
                        let actions_node_id = self.arena.new_node(Node::new(
                            NodeKind::Actions,
                            &table_child,
                            &self.source_code,
                        ));
                        let mut cursor = actions.walk();
                        for actions_child in actions.named_children(&mut cursor) {
                            if actions_child.is_error() {
                                actions_node_id
                                    .append(self.new_error_node(&actions_child), &mut self.arena);
                            } else if let Some(t) = self.look_for_preproc_kind(&actions_child) {
                                actions_node_id.append(t, &mut self.arena);
                            } else if !self.is_comment(&actions_child) {
                                // Add name node
                                let action_node_id = self.arena.new_node(Node::new(
                                    NodeKind::Action,
                                    &actions_child,
                                    &self.source_code,
                                ));
                                // Add annotation node
                                if let Some(annotation) =
                                    actions_child.child_by_field_name("annotation")
                                {
                                    action_node_id.append(
                                        self.parse_annotation(&annotation)
                                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                                        &mut self.arena,
                                    );
                                }

                                // Add name node
                                let name_node = self.arena.new_node(Node::new(
                                    NodeKind::Type(Type::Name),
                                    &actions_child.child_by_field_name("name").unwrap(),
                                    &self.source_code,
                                ));
                                action_node_id.append(name_node, &mut self.arena);

                                if let Some(params_syntax_node) = node.child_by_field_name("args") {
                                    let params_node_id =
                                        self.parse_args(&params_syntax_node).unwrap_or_else(|| {
                                            self.new_error_node(&params_syntax_node)
                                        });
                                    node_id.append(params_node_id, &mut self.arena);
                                }

                                actions_node_id.append(action_node_id, &mut self.arena);
                            }
                        }
                        child_node_id = Some(actions_node_id);
                    }
                    "entries_table" => {
                        let entries = table_child.child_by_field_name("entries").unwrap();
                        let entries_node_id = self.arena.new_node(Node::new(
                            NodeKind::Entries,
                            &table_child,
                            &self.source_code,
                        ));
                        let mut cursor = entries.walk();
                        for entries_child in entries.named_children(&mut cursor) {
                            if entries_child.is_error() {
                                entries_node_id
                                    .append(self.new_error_node(&entries_child), &mut self.arena);
                            } else if let Some(t) = self.look_for_preproc_kind(&entries_child) {
                                entries_node_id.append(t, &mut self.arena);
                            } else if !self.is_comment(&entries_child) {
                                // Add name node
                                let entrie_node_id = self.arena.new_node(Node::new(
                                    NodeKind::Entrie,
                                    &entries_child,
                                    &self.source_code,
                                ));
                                // Add annotation node
                                if let Some(annotation) =
                                    entries_child.child_by_field_name("annotation")
                                {
                                    entrie_node_id.append(
                                        self.parse_annotation(&annotation)
                                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                                        &mut self.arena,
                                    );
                                }

                                // Add name node
                                let name_node = self.arena.new_node(Node::new(
                                    NodeKind::Name,
                                    &entries_child.child_by_field_name("name").unwrap(),
                                    &self.source_code,
                                ));
                                entrie_node_id.append(name_node, &mut self.arena);

                                if let Some(params_syntax_node) = node.child_by_field_name("args") {
                                    let params_node_id =
                                        self.parse_args(&params_syntax_node).unwrap_or_else(|| {
                                            self.new_error_node(&params_syntax_node)
                                        });
                                    node_id.append(params_node_id, &mut self.arena);
                                }

                                // _keyset_expression
                                if let Some(x) = entries_child.named_child(self.get_named_child(&entries_child, 0)?) {
                                    if x.kind() == "tuple_keyset_expression" {
                                        if let Some(y) = x.child_by_field_name("reduce") {
                                            entrie_node_id.append(
                                                self.parse_reduced_simple_keyset_expression(&y)
                                                    .unwrap_or_else(|| self.new_error_node(&y)),
                                                &mut self.arena,
                                            );
                                        } else {
                                            let node_child_0 = x.named_child(self.get_named_child(&x, 0)?)?;
                                            let node_child_1 = x.named_child(self.get_named_child(&x, 1)?)?;
                                            entrie_node_id.append(
                                                self.parse_simple_keyset_expression(&node_child_0)
                                                    .unwrap_or_else(|| self.new_error_node(&node_child_0)),
                                                &mut self.arena,
                                            );
                                            entrie_node_id.append(
                                                self.parse_simple_expression_list(&node_child_1)
                                                    .unwrap_or_else(|| self.new_error_node(&node_child_1)),
                                                &mut self.arena,
                                            );
                                        }
                                    } else if x.kind() == "simple_keyset_expression" {
                                        entrie_node_id.append(
                                            self.parse_simple_keyset_expression(&x)
                                                .unwrap_or_else(|| self.new_error_node(&x)),
                                            &mut self.arena,
                                        );
                                    } else if let Some(t) = self.look_for_preproc_kind(&x) {
                                        entrie_node_id.append(t, &mut self.arena);
                                    }
                                }

                                entries_node_id.append(entrie_node_id, &mut self.arena);
                            }
                        }
                        child_node_id = Some(entries_node_id);
                    }
                    "name_table" => {
                        let name = table_child.child_by_field_name("name").unwrap();
                        let table_kw_node_id = self.arena.new_node(Node::new(
                            NodeKind::TableKw,
                            &table_child,
                            &self.source_code,
                        ));

                        // Add name node
                        let name_node = self.arena.new_node(Node::new(
                            NodeKind::Name,
                            &name,
                            &self.source_code,
                        ));
                        table_kw_node_id.append(name_node, &mut self.arena);

                        // Add value node
                        if let Some(expr) = table_child.child_by_field_name("expression") {
                            let value_node = expr.named_child(self.get_named_child(&expr, 0)?).unwrap();
                            table_kw_node_id.append(
                                self.parse_value(&value_node)
                                    .unwrap_or_else(|| self.new_error_node(&value_node)),
                                &mut self.arena,
                            );
                        }
                        child_node_id = Some(table_kw_node_id);
                    }

                    "preproc_include_declaration" => {
                        child_node_id = self.parse_preproc_include(&table_child)
                    }
                    "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                        child_node_id = self.parse_preproc_define(&table_child)
                    }
                    "preproc_undef_declaration" => {
                        child_node_id = self.parse_preproc_undef(&table_child)
                    }
                    _ => {}
                }
            }

            if let Some(id) = child_node_id {
                // Add keyword node
                if let Some(type_node) = table_child.child_by_field_name("KeyWord") {
                    id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }
                // Add keyword node
                if let Some(type_node) = table_child.child_by_field_name("KeyWordEnd") {
                    id.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }
                // Add annotation node
                if let Some(annotation) = table_child.child_by_field_name("annotation") {
                    id.append(
                        self.parse_annotation(&annotation)
                            .unwrap_or_else(|| self.new_error_node(&annotation)),
                        &mut self.arena,
                    );
                }
                table_node_id.append(id, &mut self.arena);
            }
        }

        node_id.append(table_node_id, &mut self.arena);

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_state(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::StateParser, node, &self.source_code));

        // Add keyword node
        if let Some(type_node) = node.child_by_field_name("KeyWord") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::KeyWord, &type_node, &self.source_code)),
                &mut self.arena,
            );
        }
        // Add annotation node
        if let Some(annotation) = node.child_by_field_name("annotation") {
            node_id.append(
                self.parse_annotation(&annotation)
                    .unwrap_or_else(|| self.new_error_node(&annotation)),
                &mut self.arena,
            );
        }

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // Add value node
        let body_node = node.child_by_field_name("body").unwrap();
        let value_node =
            self.arena
                .new_node(Node::new(NodeKind::Body, &body_node, &self.source_code));

        if let Some(statement) = body_node.child_by_field_name("statement") {
            let mut cursor = statement.walk();
            for body_child in statement.named_children(&mut cursor) {
                let child_node_id = if body_child.is_error() {
                    Some(self.new_error_node(&body_child))
                } else {
                    match body_child.kind() {
                        "assignment_or_method_call_statement" => {
                            self.parse_state_assignment(&body_child)
                        }
                        "direct_application" => self.parse_state_direct(&body_child),
                        "parser_block_statement" => self.parse_state_block(&body_child),
                        "constant_declaration" => self.parse_const_dec(&body_child),
                        "variable_declaration" => self.parse_var_dec(&body_child),
                        "empty_statement" => None,
                        "conditional_statement" => self.parse_state_conditional(&body_child),

                        "preproc_include_declaration" => self.parse_preproc_include(&body_child),
                        "preproc_define_declaration" | "preproc_define_declaration_macro" => {
                            self.parse_preproc_define(&body_child)
                        }
                        "preproc_undef_declaration" => self.parse_preproc_undef(&body_child),
                        _ => None,
                    }
                };

                if let Some(id) = child_node_id {
                    value_node.append(id, &mut self.arena);
                }
            }
        }
        for node_preproc in self.look_for_preproc(&body_node) {
            value_node.append(node_preproc, &mut self.arena);
        }

        node_id.append(value_node, &mut self.arena);

        if let Some(transition_statement) = body_node.child_by_field_name("transition_statement") {
            let transition_node = self.arena.new_node(Node::new(
                NodeKind::TransitionStatement,
                &transition_statement,
                &self.source_code,
            ));

            // Add keyword node
            if let Some(type_node) = transition_statement.child_by_field_name("KeyWord") {
                transition_node.append(
                    self.arena.new_node(Node::new(
                        NodeKind::KeyWord,
                        &type_node,
                        &self.source_code,
                    )),
                    &mut self.arena,
                );
            }

            if let Some(name) = transition_statement.child_by_field_name("name") {
                transition_node.append(
                    self.arena.new_node(Node::new(
                        NodeKind::Type(Type::Name),
                        &name,
                        &self.source_code,
                    )),
                    &mut self.arena,
                );
            } else {
                let t = transition_statement.named_child(self.get_named_child(&transition_statement, 0)?)?;

                // Add keyword node
                if let Some(type_node) = t.child_by_field_name("KeyWord") {
                    transition_node.append(
                        self.arena.new_node(Node::new(
                            NodeKind::KeyWord,
                            &type_node,
                            &self.source_code,
                        )),
                        &mut self.arena,
                    );
                }

                let select_expression_params_temp = t.named_child(self.get_named_child(&t, 0)?)?;
                let select_expression_params = select_expression_params_temp.named_child(self.get_named_child(&select_expression_params_temp, 0)?);
                let select_expression_body_temp = t.named_child(self.get_named_child(&t, 1)?)?;
                let select_expression_body = select_expression_body_temp.named_child(self.get_named_child(&select_expression_body_temp, 0)?);

                if let Some(select_expression_params_node) = select_expression_params {
                    transition_node.append(
                        self.parse_expression_list(&select_expression_params_node)
                            .unwrap_or_else(|| self.new_error_node(&select_expression_params_node)),
                        &mut self.arena,
                    );
                }
                if let Some(select_expression_body_node) = select_expression_body {
                    let expression_body_node = self.arena.new_node(Node::new(
                        NodeKind::Body,
                        &select_expression_body_node,
                        &self.source_code,
                    ));
                    let mut cursor = select_expression_body_node.walk();
                    for body_child in select_expression_body_node.named_children(&mut cursor) {
                        if body_child.is_error() {
                            expression_body_node
                                .append(self.new_error_node(&body_child), &mut self.arena);
                        } else if let Some(t) = self.look_for_preproc_kind(&body_child) {
                            expression_body_node.append(t, &mut self.arena);
                        } else if body_child.kind() == "select_case" {
                            let t = self.arena.new_node(Node::new(
                                NodeKind::Row,
                                &body_node,
                                &self.source_code,
                            ));

                            // Add name node
                            if let Some(x) = body_child.child_by_field_name("name") {
                                t.append(
                                    self.arena.new_node(Node::new(
                                        NodeKind::Type(Type::Name),
                                        &x,
                                        &self.source_code,
                                    )),
                                    &mut self.arena,
                                );
                            }

                            if let Some(x) = body_child.child_by_field_name("type") {
                                // todo - preproc
                                if x.kind() == "tuple_keyset_expression" {
                                    if let Some(y) = x.child_by_field_name("reduce") {
                                        transition_node.append(
                                            self.parse_reduced_simple_keyset_expression(&y)
                                                .unwrap_or_else(|| self.new_error_node(&y)),
                                            &mut self.arena,
                                        );
                                    } else {
                                        let node_child_0 = x.named_child(self.get_named_child(&x, 0)?)?;
                                        let node_child_1 = x.named_child(self.get_named_child(&x, 1)?)?;
                                        transition_node.append(
                                            self.parse_simple_keyset_expression(&node_child_0)
                                                .unwrap_or_else(|| self.new_error_node(&node_child_0)),
                                            &mut self.arena,
                                        );
                                        transition_node.append(
                                            self.parse_simple_expression_list(&node_child_1)
                                                .unwrap_or_else(|| self.new_error_node(&node_child_1)),
                                            &mut self.arena,
                                        );
                                    }
                                } else if x.kind() == "simple_keyset_expression" {
                                    transition_node.append(
                                        self.parse_simple_keyset_expression(&x)
                                            .unwrap_or_else(|| self.new_error_node(&x)),
                                        &mut self.arena,
                                    );
                                }
                            }
                            expression_body_node.append(t, &mut self.arena);
                        }
                    }
                    transition_node.append(expression_body_node, &mut self.arena);
                }
                for node_preproc in self.look_for_preproc(&transition_statement) {
                    transition_node.append(node_preproc, &mut self.arena);
                }
            }

            node_id.append(transition_node, &mut self.arena);
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }

    fn parse_annotation(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::Annotations, node, &self.source_code));

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.is_error() {
                node_id.append(self.new_error_node(&child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&child) {
                node_id.append(t, &mut self.arena);
            } else if !self.is_comment(&child) {
                let child_node_id =
                    self.arena
                        .new_node(Node::new(NodeKind::Annotation, node, &self.source_code));

                child_node_id.append(
                    self.arena.new_node(Node::new(
                        NodeKind::Name,
                        &child.child_by_field_name("name").unwrap(),
                        &self.source_code,
                    )),
                    &mut self.arena,
                );

                if let Some(body) = child.child_by_field_name("body") {
                    child_node_id.append(
                        self.parse_annotation_body(&body)
                            .unwrap_or_else(|| self.new_error_node(&body)),
                        &mut self.arena,
                    );
                }
                if let Some(struct_body) = node.child_by_field_name("struct") {
                    if struct_body.kind() == "expression_list" {
                        child_node_id.append(
                            self.parse_expression_list(&struct_body)
                                .unwrap_or_else(|| self.new_error_node(&struct_body)),
                            &mut self.arena,
                        );
                    } else if struct_body.kind() == "kv_list" {
                        child_node_id.append(
                            self.parse_kv_list(&struct_body)
                                .unwrap_or_else(|| self.new_error_node(&struct_body)),
                            &mut self.arena,
                        );
                    } else if let Some(t) = self.look_for_preproc_kind(&struct_body) {
                        child_node_id.append(t, &mut self.arena);
                    }
                }

                node_id.append(child_node_id, &mut self.arena);
            }
        }

        Some(node_id)
    }

    fn parse_kv_list(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::KvList, node, &self.source_code));

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.is_error() {
                node_id.append(self.new_error_node(&child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&child) {
                node_id.append(t, &mut self.arena);
            } else if !self.is_comment(&child) {
                let child_node_id =
                    self.arena
                        .new_node(Node::new(NodeKind::Kv, node, &self.source_code));

                // Add name node
                let node_name = child.named_child(self.get_named_child(&child, 0)?).unwrap();
                let name_node =
                    self.arena
                        .new_node(Node::new(NodeKind::Name, &node_name, &self.source_code));
                child_node_id.append(name_node, &mut self.arena);

                // Add value node
                let node_value = child.named_child(self.get_named_child(&child, 1)?).unwrap();
                child_node_id.append(
                    self.parse_value(&node_value)
                        .unwrap_or_else(|| self.new_error_node(&node_value)),
                    &mut self.arena,
                );

                node_id.append(child_node_id, &mut self.arena);
            }
        }

        Some(node_id)
    }

    fn parse_annotation_body(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Body, node, &self.source_code));

        if let Some(body) = node.child_by_field_name("body") {
            node_id.append(
                self.parse_annotation_body(&body)
                    .unwrap_or_else(|| self.new_error_node(&body)),
                &mut self.arena,
            );
        }

        if let Some(body) = node.child_by_field_name("body2") {
            node_id.append(
                self.parse_annotation_body(&body)
                    .unwrap_or_else(|| self.new_error_node(&body)),
                &mut self.arena,
            );
        }

        if let Some(token) = node.child_by_field_name("token") {
            node_id.append(
                self.arena
                    .new_node(Node::new(NodeKind::Token, &token, &self.source_code)),
                &mut self.arena,
            );
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_expression_list(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Expression, node, &self.source_code));

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.is_error() {
                node_id.append(self.new_error_node(&child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&child) {
                node_id.append(t, &mut self.arena);
            } else if !self.is_comment(&child) {
                node_id.append(
                    self.parse_value(&child)
                        .unwrap_or_else(|| self.new_error_node(&child)),
                    &mut self.arena,
                );
            }
        }

        Some(node_id)
    }

    fn parse_simple_keyset_expression(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Expression, node, &self.source_code));

        if let Some(value) = node.child_by_field_name("value") {
            node_id.append(
                self.parse_value(&value)
                    .unwrap_or_else(|| self.new_error_node(&value)),
                &mut self.arena,
            );
        }
        if let Some(value) = node.child_by_field_name("value2") {
            node_id.append(
                self.parse_value(&value)
                    .unwrap_or_else(|| self.new_error_node(&value)),
                &mut self.arena,
            );
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_reduced_simple_keyset_expression(
        &mut self,
        node: &tree_sitter::Node,
    ) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Expression, node, &self.source_code));

        if let Some(value) = node.child_by_field_name("value") {
            node_id.append(
                self.parse_value(&value)
                    .unwrap_or_else(|| self.new_error_node(&value)),
                &mut self.arena,
            );
        }
        if let Some(value) = node.child_by_field_name("value2") {
            node_id.append(
                self.parse_value(&value)
                    .unwrap_or_else(|| self.new_error_node(&value)),
                &mut self.arena,
            );
        }

        for node_preproc in self.look_for_preproc(&node) {
            node_id.append(node_preproc, &mut self.arena);
        }

        Some(node_id)
    }
    fn parse_simple_expression_list(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::Expression, node, &self.source_code));

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.is_error() {
                node_id.append(self.new_error_node(&child), &mut self.arena);
            } else if let Some(t) = self.look_for_preproc_kind(&child) {
                node_id.append(t, &mut self.arena);
            } else if !self.is_comment(&child) {
                node_id.append(
                    self.parse_simple_keyset_expression(&child)
                        .unwrap_or_else(|| self.new_error_node(&child)),
                    &mut self.arena,
                );
            }
        }

        Some(node_id)
    }
}
