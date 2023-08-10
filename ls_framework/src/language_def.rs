use serde::Deserialize;
use tokio::sync::OnceCell;
use tower_lsp::lsp_types::CompletionItemKind;

use crate::metadata::NodeKind;

#[derive(Debug, Deserialize, Clone)]
pub enum SymbolCompletionType {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl SymbolCompletionType {
    pub fn get_completion_kind(&self) -> CompletionItemKind {
        match self {
            SymbolCompletionType::Text => CompletionItemKind::TEXT,
            SymbolCompletionType::Method => CompletionItemKind::METHOD,
            SymbolCompletionType::Function => CompletionItemKind::FUNCTION,
            SymbolCompletionType::Constructor => CompletionItemKind::CONSTRUCTOR,
            SymbolCompletionType::Field => CompletionItemKind::FIELD,
            SymbolCompletionType::Variable => CompletionItemKind::VARIABLE,
            SymbolCompletionType::Class => CompletionItemKind::CLASS,
            SymbolCompletionType::Interface => CompletionItemKind::INTERFACE,
            SymbolCompletionType::Module => CompletionItemKind::MODULE,
            SymbolCompletionType::Property => CompletionItemKind::PROPERTY,
            SymbolCompletionType::Unit => CompletionItemKind::UNIT,
            SymbolCompletionType::Value => CompletionItemKind::VALUE,
            SymbolCompletionType::Enum => CompletionItemKind::ENUM,
            SymbolCompletionType::Keyword => CompletionItemKind::KEYWORD,
            SymbolCompletionType::Snippet => CompletionItemKind::SNIPPET,
            SymbolCompletionType::Color => CompletionItemKind::COLOR,
            SymbolCompletionType::File => CompletionItemKind::FILE,
            SymbolCompletionType::Reference => CompletionItemKind::REFERENCE,
            SymbolCompletionType::Folder => CompletionItemKind::FOLDER,
            SymbolCompletionType::EnumMember => CompletionItemKind::ENUM_MEMBER,
            SymbolCompletionType::Constant => CompletionItemKind::CONSTANT,
            SymbolCompletionType::Struct => CompletionItemKind::STRUCT,
            SymbolCompletionType::Event => CompletionItemKind::EVENT,
            SymbolCompletionType::Operator => CompletionItemKind::OPERATOR,
            SymbolCompletionType::TypeParameter => CompletionItemKind::TYPE_PARAMETER,
        }
    }
}

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
}

#[derive(Debug, Deserialize)]
pub struct LanguageDefinition {
    pub keywords: Vec<String>,
    pub symbol_types: Vec<SymbolDef>,
    pub ast_rules: Vec<Rule>,
}

static INSTANCE: OnceCell<LanguageDefinition> = OnceCell::const_new();

impl LanguageDefinition {
    pub fn load(language_definition: &str) {
        let language_def_modified =
            format!("#![enable(unwrap_variant_newtypes)]\n{language_definition}");

        INSTANCE
            .set(
                ron::de::from_str(&language_def_modified).unwrap_or_else(|e| {
                    error!("Failed to parse rules: {}", e);
                    panic!("Failed to parse rules: {}", e);
                }),
            )
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

    pub fn get_scope_nodes(&self) -> Vec<NodeKind> {
        self.ast_rules
            .iter()
            .filter(|rule| rule.is_scope)
            .map(|rule| NodeKind::Node(rule.node_name.clone()))
            .collect()
    }
}
