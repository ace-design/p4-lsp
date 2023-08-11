use std::collections::HashSet;

use itertools::Itertools;
use serde::Deserialize;
use tokio::sync::OnceCell;
use tower_lsp::lsp_types;

use crate::lsp_mappings::{SemanticTokenType, SymbolCompletionType};
use crate::metadata::NodeKind;

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub node_name: String,
    #[serde(default)]
    pub symbol: Symbol,
    #[serde(default)]
    pub is_scope: bool,
    #[serde(default)]
    pub children: Vec<Multiplicity>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Multiplicity {
    One(Child),
    Maybe(Child),
    Many(Child),
}

impl Multiplicity {
    pub fn get_child(&self) -> &Child {
        match self {
            Multiplicity::One(c) | Multiplicity::Maybe(c) | Multiplicity::Many(c) => c,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Child {
    pub query: TreesitterNodeQuery,
    pub rule: DirectOrRule,
    pub semantic_token_type: Option<SemanticTokenType>,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
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

#[derive(Debug, PartialEq, Deserialize, Clone, Default)]
pub enum Symbol {
    Init {
        #[serde(rename(deserialize = "type"))]
        kind: String,
        name_node: String,
    },
    Usage,
    #[default]
    None,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SymbolDef {
    pub name: String,
    pub completion_type: SymbolCompletionType,
    pub semantic_token_type: SemanticTokenType,
}

#[derive(Debug, Deserialize)]
pub struct LanguageDefinition {
    pub keywords: Vec<String>,
    pub symbol_types: Vec<SymbolDef>,
    pub ast_rules: Vec<Rule>,
}

static INSTANCE: OnceCell<LanguageDefinition> = OnceCell::const_new();
static SCOPE_NODES: OnceCell<Vec<NodeKind>> = OnceCell::const_new();
static KEYWORDS: OnceCell<HashSet<String>> = OnceCell::const_new();
static SEMANTIC_TOKEN_TYPES: OnceCell<Vec<lsp_types::SemanticTokenType>> = OnceCell::const_new();

impl LanguageDefinition {
    pub fn load(language_definition: &str) {
        let language_def_modified = format!(
            "#![enable(unwrap_variant_newtypes)]\n#![enable(implicit_some)]\n{language_definition}"
        );

        INSTANCE
            .set(
                ron::de::from_str(&language_def_modified).unwrap_or_else(|e| {
                    error!("Failed to parse rules: {}", e);
                    panic!("Failed to parse rules: {}", e);
                }),
            )
            .unwrap();

        let instance = INSTANCE.get().unwrap();

        SCOPE_NODES
            .set(
                instance
                    .ast_rules
                    .iter()
                    .filter(|rule| rule.is_scope)
                    .map(|rule| NodeKind::Node(rule.node_name.clone()))
                    .collect(),
            )
            .unwrap();

        SEMANTIC_TOKEN_TYPES
            .set(
                instance
                    .symbol_types
                    .iter()
                    .map(|s| s.semantic_token_type.get())
                    .unique()
                    .collect::<Vec<lsp_types::SemanticTokenType>>(),
            )
            .unwrap();

        KEYWORDS
            .set(HashSet::from_iter(instance.keywords.clone()))
            .unwrap();
    }

    pub fn get() -> &'static LanguageDefinition {
        INSTANCE
            .get()
            .expect("LanguageDefinition has not been loaded.")
    }

    pub fn rule_with_name(&self, name: &str) -> Option<&Rule> {
        self.ast_rules.iter().find(|rule| rule.node_name == name)
    }

    pub fn get_semantic_token_types(&self) -> &Vec<lsp_types::SemanticTokenType> {
        SEMANTIC_TOKEN_TYPES
            .get()
            .expect("LanguageDefinition has not been loaded.")
    }

    pub fn get_scope_nodes() -> &'static Vec<NodeKind> {
        SCOPE_NODES
            .get()
            .expect("LanguageDefinition has not been loaded.")
    }

    pub fn get_keywords() -> &'static HashSet<String> {
        KEYWORDS
            .get()
            .expect("LanguageDefinition has not been loaded.")
    }
}
