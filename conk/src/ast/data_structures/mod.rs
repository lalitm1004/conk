mod attribute;
pub use attribute::{BlockAttribute, FieldAttribute};

mod config;
pub use config::Config;

mod value;
pub use value::{Argument, Value};
use value::{parse_argument, parse_value};

mod declaration;
pub use declaration::{EntityDeclaration, EnumDeclaration, TemplateDeclaration};

mod field;
pub use field::{Field, FieldType};

#[derive(Debug, Clone)]
pub struct ConkAST {
    pub config: Option<Config>,
    pub enum_declarations: Vec<EnumDeclaration>,
    pub template_declarations: Vec<TemplateDeclaration>,
    pub entity_declarations: Vec<EntityDeclaration>,
}
