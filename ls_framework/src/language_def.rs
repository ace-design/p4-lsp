use serde::Deserialize;
use tokio::sync::OnceCell;

use crate::metadata::NodeKind;

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub node: NodeKind,
    #[serde(default)]
    pub is_scope: bool,
    #[serde(default)]
    pub children: Vec<Child>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Child {
    One(TreesitterNodeQuery, NodeOrRule),
    Maybe(TreesitterNodeQuery, NodeOrRule),
    Multiple(TreesitterNodeQuery, NodeOrRule),
}

#[derive(Debug, Deserialize, Clone)]
pub enum TreesitterNodeQuery {
    Path(Vec<TreesitterNodeQuery>),
    Kind(String),
    Field(String),
}

#[derive(Debug, Deserialize, Clone)]
pub enum NodeOrRule {
    Node(NodeKind),
    Rule(String),
}

#[derive(Debug, Deserialize)]
pub struct LanguageDefinition {
    pub ast_rules: Vec<Rule>,
}

static INSTANCE: OnceCell<LanguageDefinition> = OnceCell::const_new();

impl LanguageDefinition {
    pub fn load(language_definition: &str) {
        INSTANCE
            .set(ron::de::from_str(language_definition).unwrap_or_else(|e| {
                error!("Failed to parse rules: {}", e);
                panic!("Failed to parse rules: {}", e);
            }))
            .unwrap();
    }

    pub fn get() -> &'static LanguageDefinition {
        INSTANCE
            .get()
            .expect("LanguageDefinition has not been loaded.")
    }

    pub fn rule_with_name(&self, name: &str) -> Option<&Rule> {
        self.ast_rules.iter().find(|rule| rule.name == name)
    }

    pub fn get_scope_nodes(&self) -> Vec<NodeKind> {
        self.ast_rules
            .iter()
            .filter(|rule| rule.is_scope)
            .map(|rule| rule.node.clone())
            .collect()
    }
}
