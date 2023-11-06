use indextree::{Arena, NodeId};

use super::{tree::Translator, Ast, Node, NodeKind};
use crate::{
    language_def::{Child, DirectOrRule, LanguageDefinition, Rule, Symbol, TreesitterNodeQuery},
    lsp_mappings::HighlightType,
};

pub struct RulesTranslator {
    arena: Arena<Node>,
    source_code: String,
    tree: tree_sitter::Tree,
    language_def: &'static LanguageDefinition,
}

impl Translator for RulesTranslator {
    fn translate(source_code: String, syntax_tree: tree_sitter::Tree) -> Ast {
        let mut translator = RulesTranslator::new(source_code, syntax_tree);
        let root_id = translator.build();

        Ast::initialize(translator.arena, root_id)
    }
}

impl RulesTranslator {
    fn new(source_code: String, syntax_tree: tree_sitter::Tree) -> RulesTranslator {
        RulesTranslator {
            source_code,
            arena: Arena::new(),
            tree: syntax_tree,
            language_def: LanguageDefinition::get(),
        }
    }

    fn build(&mut self) -> NodeId {
        let root_rule = self.language_def.rule_with_name("Root").unwrap();

        self.parse(&root_rule.clone(), &self.tree.clone().root_node())
    }

    fn parse(&mut self, current_rule: &Rule, current_ts_node: &tree_sitter::Node) -> NodeId {
        let mut cursor = current_ts_node.walk();
        let children: Vec<tree_sitter::Node> = current_ts_node.children(&mut cursor).collect();

        let current_node_id = self.new_node(
            NodeKind::Node(current_rule.node_name.clone()),
            current_ts_node,
            current_rule.symbol.clone(),
            None,
        );
        // TODO: has_error vs is_error
        // for error_ts_node in children.iter().filter(|node| node.has_error()) {
        //     current_node_id.append(self.new_error_node(error_ts_node, None), &mut self.arena);
        // }

        for child in current_rule.children.iter() {
            self.query_parse_child(&children, child, current_node_id);
        }

        for child in &LanguageDefinition::get().global_ast_rules {
            self.query_parse_child(&children, child, current_node_id);
        }

        current_node_id
    }

    fn query_parse_child(
        &mut self,
        children: &[tree_sitter::Node],
        child: &Child,
        current_node_id: NodeId,
    ) {
        let (query, node_or_rule) = (&child.query, &child.rule);

        for (i, ts_node) in children.iter().enumerate() {
            let target_node = if let TreesitterNodeQuery::Path(path) = query {
                if path.is_empty() {
                    continue;
                }

                let mut current_ts_node = *ts_node;
                if !match &path[0] {
                    TreesitterNodeQuery::Path(_) => unimplemented!(), // TODO
                    TreesitterNodeQuery::Kind(kind) => current_ts_node.kind() == kind,
                    TreesitterNodeQuery::Field(name) => {
                        current_ts_node
                            .parent()
                            .unwrap()
                            .field_name_for_child(i as u32)
                            == Some(name)
                    }
                } {
                    continue;
                }

                let mut cursor = current_ts_node.walk();
                for element in path.iter().skip(1) {
                    let found = current_ts_node
                        .children(&mut cursor)
                        .enumerate()
                        .filter(|node| node.1.is_named())
                        .find(|(i, ts_node)| match element {
                            TreesitterNodeQuery::Path(_) => unimplemented!(),
                            TreesitterNodeQuery::Kind(kind) => ts_node.kind() == kind,
                            TreesitterNodeQuery::Field(name) => {
                                ts_node.parent().unwrap().field_name_for_child(*i as u32)
                                    == Some(name)
                            }
                        });

                    if let Some((_, node)) = found {
                        current_ts_node = node;
                    } else {
                        continue;
                    }
                }
                current_ts_node
            } else {
                *ts_node
            };

            if match query {
                TreesitterNodeQuery::Kind(kind) => ts_node.kind() == kind,
                TreesitterNodeQuery::Field(name) => {
                    ts_node.parent().unwrap().field_name_for_child(i as u32) == Some(name)
                }
                TreesitterNodeQuery::Path(_) => true,
            } {
                match node_or_rule {
                    DirectOrRule::Direct(node_kind) => {
                        if ts_node.has_error() {
                            // current_node_id
                            //     .append(self.new_error_node(ts_node, None), &mut self.arena);
                        }

                        current_node_id.append(
                            self.new_node(
                                NodeKind::Node(node_kind.clone()),
                                &target_node,
                                Symbol::None,
                                child.highlight_type.clone(),
                            ),
                            &mut self.arena,
                        );
                    }
                    DirectOrRule::Rule(name) => {
                        let rule = self.language_def.rule_with_name(name).unwrap().clone();
                        current_node_id.append(self.parse(&rule, &target_node), &mut self.arena);
                    }
                }
            }
        }
    }

    fn new_node(
        &mut self,
        kind: NodeKind,
        syntax_node: &tree_sitter::Node,
        symbol: Symbol,
        semantic_token_type: Option<HighlightType>,
    ) -> NodeId {
        self.arena.new_node(Node::new(
            kind,
            syntax_node,
            &self.source_code,
            symbol,
            semantic_token_type,
        ))
    }

    fn new_error_node(
        &mut self,
        syntax_node: &tree_sitter::Node,
        message: Option<String>,
    ) -> NodeId {
        self.arena.new_node(Node::new(
            NodeKind::Error(message),
            syntax_node,
            &self.source_code,
            Symbol::None,
            None,
        ))
    }
}
