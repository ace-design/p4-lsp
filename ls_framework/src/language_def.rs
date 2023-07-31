use serde::Deserialize;
use tokio::sync::OnceCell;
use tower_lsp::lsp_types::CompletionItemKind;

use crate::metadata::NodeKind;

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub node: NodeKind,
    #[serde(default)]
    pub symbol: Symbol,
    #[serde(default)]
    pub is_scope: bool,
    #[serde(default)]
    pub children: Vec<Child>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Child {
    One(TreesitterNodeQuery, DirectOrRule),
    Maybe(TreesitterNodeQuery, DirectOrRule),
    Multiple(TreesitterNodeQuery, DirectOrRule),
}

#[derive(Debug, Deserialize, Clone)]
pub enum TreesitterNodeQuery {
    Path(Vec<TreesitterNodeQuery>),
    Kind(String),
    Field(String),
}

#[derive(Debug, Deserialize, Clone)]
pub enum DirectOrRule {
    Direct(NodeKind),
    Rule(String),
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum Symbol {
    Init(String),
    Usage,
    #[default]
    None,
}

#[derive(Debug, Deserialize)]
pub struct LanguageDefinition {
    pub symbol_types: Vec<(String, CompletionItemKind)>,
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

    pub fn get_symbol_init_nodes(&self) -> Vec<(NodeKind, String)> {
        self.ast_rules
            .iter()
            .filter(|rule| matches!(rule.symbol, Symbol::Init(_)))
            .map(|rule| {
                (
                    rule.node.clone(),
                    match rule.symbol.clone() {
                        Symbol::Init(type_name) => type_name,
                        _ => unreachable!(),
                    },
                )
            })
            .collect()
    }
}
