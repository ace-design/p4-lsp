use std::collections::HashSet;

use itertools::Itertools;
use serde::Deserialize;
use tokio::sync::OnceCell;
use tower_lsp::lsp_types::{self, SemanticTokensLegend};

use crate::lsp_mappings::{HighlightType, SymbolCompletionType};
use crate::metadata::NodeKind;

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub node_name: String,
    #[serde(default)]
    pub symbol: Symbol,
    #[serde(default)]
    pub is_scope: bool,
    #[serde(default)]
    pub children: Vec<Child>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Child {
    pub query: TreesitterNodeQuery,
    pub rule: DirectOrRule,
    pub highlight_type: Option<HighlightType>,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub enum TreesitterNodeQuery {
    Path(Vec<TreesitterNodeQuery>),
    Kind(String),
    Field(String),
}

#[derive(Debug, Deserialize, Clone)]
pub enum DirectOrRule {
    Direct(String),
    Rule(String),
}

#[derive(Debug, PartialEq, Deserialize, Clone, Default)]
pub enum Symbol {
    Init {
        #[serde(rename(deserialize = "type"))]
        kind: String,
        name_node: String,
        type_node: Option<String>,
    },
    Usage,
    Field {
        name_node: String,
    },
    Expression,
    MemberUsage,
    #[default]
    None,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SymbolDef {
    pub name: String,
    pub completion_type: SymbolCompletionType,
    pub highlight_type: HighlightType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Language {
    pub name: String,
    pub file_extensions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LanguageDefinition {
    pub language: Language,
    pub keywords: Vec<String>,
    pub symbol_types: Vec<SymbolDef>,
    pub global_ast_rules: Vec<Child>,
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
            .set(instance.init_semanc_token_types())
            .unwrap();

        KEYWORDS
            .set(HashSet::from_iter(instance.keywords.clone()))
            .unwrap();
    }

    fn init_semanc_token_types(&self) -> Vec<lsp_types::SemanticTokenType> {
        let mut symbol_types = self
            .symbol_types
            .iter()
            .map(|s| s.highlight_type.get())
            .collect::<Vec<lsp_types::SemanticTokenType>>();
        symbol_types.insert(0, HighlightType::Keyword.get());

        for rule in &self.ast_rules {
            for child in &rule.children {
                if let Some(semantic_token_type) = &child.highlight_type {
                    symbol_types.push(semantic_token_type.get());
                }
            }
        }

        for child in &self.global_ast_rules {
            if let Some(semantic_token_type) = &child.highlight_type {
                symbol_types.push(semantic_token_type.get());
            }
        }

        symbol_types.into_iter().unique().collect_vec()
    }

    pub fn get() -> &'static LanguageDefinition {
        INSTANCE
            .get()
            .expect("LanguageDefinition has not been loaded.")
    }

    pub fn rule_with_name(&self, name: &str) -> Option<&Rule> {
        self.ast_rules.iter().find(|rule| rule.node_name == name)
    }

    pub fn get_semantic_token_types() -> &'static Vec<lsp_types::SemanticTokenType> {
        SEMANTIC_TOKEN_TYPES
            .get()
            .expect("LanguageDefinition has not been loaded.")
    }

    pub fn get_semantic_token_legend() -> SemanticTokensLegend {
        SemanticTokensLegend {
            token_types: LanguageDefinition::get_semantic_token_types().clone(),
            token_modifiers: vec![],
        }
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
