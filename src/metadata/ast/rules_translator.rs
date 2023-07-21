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
    Path(Vec<TreesitterNodeQuery>),
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

    pub fn rule_with_name(&self, name: &str) -> Option<&Rule> {
        self.ast_rules.iter().find(|rule| &rule.name == name)
    }

    pub fn get_scope_nodes(&self) -> Vec<NodeKind> {
        self.ast_rules
            .iter()
            .filter(|rule| rule.is_scope)
            .map(|rule| rule.node.clone())
            .collect()
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
        // TODO: has_error vs is_error
        for error_ts_node in children.iter().filter(|node| node.has_error()) {
            current_node_id.append(
                self.new_node(NodeKind::Error(None), error_ts_node),
                &mut self.arena,
            );
        }

        for child in current_rule.children.iter() {
            self.query_parse_child(&children, child, current_node_id, &current_ts_node);
        }

        current_node_id
    }

    fn query_parse_child(
        &mut self,
        children: &Vec<tree_sitter::Node>,
        child: &Child,
        current_node_id: NodeId,
        current_ts_node: &tree_sitter::Node,
    ) {
        let (query, node_or_rule) = match child {
            Child::One(query, node_or_rule) => (query, node_or_rule),
            Child::Maybe(query, node_or_rule) => (query, node_or_rule),
            Child::Multiple(query, node_or_rule) => (query, node_or_rule),
        };

        let mut counter = 0;
        for (i, ts_node) in children.iter().enumerate().filter(|node| node.1.is_named()) {
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
                            == Some(&name)
                    }
                } {
                    continue;
                }

                let mut cursor = current_ts_node.walk();
                for element in path.iter().skip(1) {
                    let found = current_ts_node
                        .children(&mut cursor)
                        .into_iter()
                        .enumerate()
                        .filter(|node| node.1.is_named())
                        .find(|(i, ts_node)| match element {
                            TreesitterNodeQuery::Path(_) => unimplemented!(),
                            TreesitterNodeQuery::Kind(kind) => ts_node.kind() == kind,
                            TreesitterNodeQuery::Field(name) => {
                                ts_node.parent().unwrap().field_name_for_child(*i as u32)
                                    == Some(&name)
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
                    NodeOrRule::Node(node_kind) => {
                        if ts_node.has_error() {
                            current_node_id.append(
                                self.new_node(NodeKind::Error(None), ts_node),
                                &mut self.arena,
                            );
                        }

                        current_node_id.append(
                            self.new_node(node_kind.clone(), &target_node),
                            &mut self.arena,
                        );
                    }
                    NodeOrRule::Rule(name) => {
                        let rule = self.language_def.rule_with_name(&name).unwrap().clone();
                        current_node_id.append(self.parse(&rule, &target_node), &mut self.arena);
                    }
                }

                counter += 1;
            }

            if matches!(child, Child::One(_, _) | Child::Maybe(_, _)) && counter > 1 {
                current_node_id.append(
                    self.new_node(
                        NodeKind::Error(Some(format!("Too many '{:?}'.", query))),
                        ts_node,
                    ),
                    &mut self.arena,
                );
            }
        }

        if matches!(child, Child::One(_, _)) && counter == 0 {
            current_node_id.append(
                self.new_node(
                    NodeKind::Error(Some(format!("Missing '{:?}'.", query))),
                    current_ts_node,
                ),
                &mut self.arena,
            );
        }
    }

    fn new_node(&mut self, kind: NodeKind, syntax_node: &tree_sitter::Node) -> NodeId {
        self.arena
            .new_node(Node::new(kind, syntax_node, &self.source_code))
    }
}
