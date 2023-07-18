use indextree::{Arena, NodeId};
use serde::Deserialize;

use super::{tree::Translator, Ast, Node, NodeKind};

pub struct RulesTranslator {
    arena: Arena<Node>,
    source_code: String,
    tree: tree_sitter::Tree,
    language_def: LanguageDefinition,
}

impl Translator for RulesTranslator {
    fn translate(source_code: String, syntax_tree: tree_sitter::Tree) -> Ast {
        let mut translator = RulesTranslator::new(source_code, syntax_tree);
        let root_id = translator.build();

        Ast::initialize(translator.arena, root_id)
    }
}

#[derive(Debug, Deserialize)]
struct LanguageDefinition {
    ast_rules: Vec<Rule>,
}

impl LanguageDefinition {
    pub fn rule_with_name(&self, name: &str) -> Option<&Rule> {
        self.ast_rules.iter().find(|rule| &rule.name == name)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Rule {
    name: String,
    node: NodeKind,
    #[serde(default)]
    is_scope: bool,
    #[serde(default)]
    children: Vec<Child>,
}

#[derive(Debug, Deserialize, Clone)]
enum Child {
    One(TreesitterNodeQuery, NodeOrRule),
    Maybe(TreesitterNodeQuery, NodeOrRule),
    Multiple(TreesitterNodeQuery, NodeOrRule),
}

#[derive(Debug, Deserialize, Clone)]
enum TreesitterNodeQuery {
    Kind(String),
    Field(String),
}

#[derive(Debug, Deserialize, Clone)]
enum NodeOrRule {
    Node(NodeKind),
    Rule(String),
}

static FILE_CONTENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/language_def/rules.ron"
));

impl LanguageDefinition {
    pub fn load() -> LanguageDefinition {
        ron::de::from_str(FILE_CONTENT).unwrap_or_else(|e| {
            error!("Failed to parse rules: {}", e);
            panic!("Failed to parse rules: {}", e);
        })
    }
}

impl RulesTranslator {
    fn new(source_code: String, syntax_tree: tree_sitter::Tree) -> RulesTranslator {
        RulesTranslator {
            source_code,
            arena: Arena::new(),
            tree: syntax_tree,
            language_def: LanguageDefinition::load(),
        }
    }

    fn build(&mut self) -> NodeId {
        let root_rule = self.language_def.rule_with_name("Root").unwrap();

        self.parse(&root_rule.clone(), &self.tree.clone().root_node())
    }

    fn parse(&mut self, current_rule: &Rule, current_ts_node: &tree_sitter::Node) -> NodeId {
        let mut cursor = current_ts_node.walk();
        let children: Vec<tree_sitter::Node> = current_ts_node.children(&mut cursor).collect();

        let current_node_id = self.new_node(current_rule.node.clone(), current_ts_node);
        for error_ts_node in children.iter().filter(|node| node.is_error()) {
            current_node_id.append(
                self.new_node(NodeKind::Error(None), error_ts_node),
                &mut self.arena,
            );
        }

        for child in current_rule.children.iter() {
            match child {
                Child::One(query, node_or_rule) => {
                    let mut counter = 0;
                    for (i, ts_node) in children.iter().enumerate().filter(|node| node.1.is_named())
                    {
                        if self.handle_query(
                            query,
                            ts_node,
                            current_ts_node,
                            i,
                            node_or_rule,
                            &current_node_id,
                        ) {
                            counter += 1;
                        }

                        if counter > 1 {
                            current_node_id.append(
                                self.new_node(
                                    NodeKind::Error(Some(format!("Too many '{:?}'.", query))),
                                    ts_node,
                                ),
                                &mut self.arena,
                            );
                        }
                    }

                    if counter == 0 {
                        current_node_id.append(
                            self.new_node(
                                NodeKind::Error(Some(format!("Missing '{:?}'.", query))),
                                current_ts_node,
                            ),
                            &mut self.arena,
                        );
                    }
                }
                Child::Maybe(query, node_or_rule) => {
                    let mut counter = 0;
                    for (i, ts_node) in children.iter().enumerate().filter(|node| node.1.is_named())
                    {
                        if self.handle_query(
                            query,
                            ts_node,
                            current_ts_node,
                            i,
                            node_or_rule,
                            &current_node_id,
                        ) {
                            counter += 1;
                        }

                        if counter > 1 {
                            current_node_id.append(
                                self.new_node(
                                    NodeKind::Error(Some(format!("Too many '{:?}'.", query))),
                                    ts_node,
                                ),
                                &mut self.arena,
                            );
                        }
                    }
                }
                Child::Multiple(query, node_or_rule) => {
                    for (i, ts_node) in children.iter().enumerate().filter(|node| node.1.is_named())
                    {
                        self.handle_query(
                            query,
                            ts_node,
                            current_ts_node,
                            i,
                            node_or_rule,
                            &current_node_id,
                        );
                    }
                }
            }
        }

        current_node_id
    }

    fn handle_query(
        &mut self,
        query: &TreesitterNodeQuery,
        ts_node: &tree_sitter::Node,
        current_ts_node: &tree_sitter::Node,
        index: usize,
        node_or_rule: &NodeOrRule,
        current_node_id: &NodeId,
    ) -> bool {
        let matched_query = match query {
            TreesitterNodeQuery::Kind(kind) => ts_node.kind() == kind,
            TreesitterNodeQuery::Field(name) => {
                current_ts_node.field_name_for_child(index as u32) == Some(name)
            }
        };

        if matched_query {
            match node_or_rule {
                NodeOrRule::Node(node_kind) => {
                    if ts_node.has_error() {
                        current_node_id.append(
                            self.new_node(NodeKind::Error(None), ts_node),
                            &mut self.arena,
                        );
                    }

                    current_node_id
                        .append(self.new_node(node_kind.clone(), ts_node), &mut self.arena);
                }
                NodeOrRule::Rule(name) => {
                    let rule = self.language_def.rule_with_name(&name).unwrap().clone();
                    current_node_id.append(self.parse(&rule, ts_node), &mut self.arena);
                }
            }
        }

        matched_query
    }

    fn new_node(&mut self, kind: NodeKind, syntax_node: &tree_sitter::Node) -> NodeId {
        self.arena
            .new_node(Node::new(kind, syntax_node, &self.source_code))
    }
}
