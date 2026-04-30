use serde::{Deserialize, Serialize};

/// Kinds of nodes in the code graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    File,
    Module,
    Struct,
    Enum,
    EnumVariant,
    Trait,
    Function,
    Method,
    Impl,
    Const,
    Static,
    TypeAlias,
    Field,
    Macro,
    Use,
    // Java-specific
    Class,
    Interface,
    Constructor,
    Annotation,
    AnnotationUsage,
    Package,
    InnerClass,
    InitBlock,
    AbstractMethod,
    // Go-specific
    InterfaceType,
    StructMethod,
    GoPackage,
    StructTag,
    // Scala-specific
    ScalaObject,
    CaseClass,
    ScalaPackage,
    ValField,
    VarField,
    // Shared
    GenericParam,
    // TypeScript/JavaScript-specific
    ArrowFunction,
    Decorator,
    Export,
    Namespace,
    // C/C++-specific
    Union,
    Typedef,
    Include,
    PreprocessorDef,
    Template,
    // Kotlin-specific
    DataClass,
    SealedClass,
    CompanionObject,
    KotlinObject,
    KotlinPackage,
    Property,
    // Dart-specific
    Mixin,
    Extension,
    Library,
    // C#-specific
    Delegate,
    Event,
    Record,
    CSharpProperty,
    // Pascal-specific
    Procedure,
    PascalUnit,
    PascalProgram,
    PascalRecord,
    // Protobuf-specific
    #[cfg(feature = "lang-protobuf")]
    ProtoMessage,
    #[cfg(feature = "lang-protobuf")]
    ProtoService,
    #[cfg(feature = "lang-protobuf")]
    ProtoRpc,
}

#[allow(clippy::should_implement_trait)]
impl NodeKind {
    /// Returns the string representation of this node kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::File => "file",
            NodeKind::Module => "module",
            NodeKind::Struct => "struct",
            NodeKind::Enum => "enum",
            NodeKind::EnumVariant => "enum_variant",
            NodeKind::Trait => "trait",
            NodeKind::Function => "function",
            NodeKind::Method => "method",
            NodeKind::Impl => "impl",
            NodeKind::Const => "const",
            NodeKind::Static => "static",
            NodeKind::TypeAlias => "type_alias",
            NodeKind::Field => "field",
            NodeKind::Macro => "macro",
            NodeKind::Use => "use",
            NodeKind::Class => "class",
            NodeKind::Interface => "interface",
            NodeKind::Constructor => "constructor",
            NodeKind::Annotation => "annotation",
            NodeKind::AnnotationUsage => "annotation_usage",
            NodeKind::Package => "package",
            NodeKind::InnerClass => "inner_class",
            NodeKind::InitBlock => "init_block",
            NodeKind::AbstractMethod => "abstract_method",
            NodeKind::InterfaceType => "interface_type",
            NodeKind::StructMethod => "struct_method",
            NodeKind::GoPackage => "go_package",
            NodeKind::StructTag => "struct_tag",
            NodeKind::ScalaObject => "object",
            NodeKind::CaseClass => "case_class",
            NodeKind::ScalaPackage => "scala_package",
            NodeKind::ValField => "val",
            NodeKind::VarField => "var",
            NodeKind::GenericParam => "generic_param",
            NodeKind::ArrowFunction => "arrow_function",
            NodeKind::Decorator => "decorator",
            NodeKind::Export => "export",
            NodeKind::Namespace => "namespace",
            NodeKind::Union => "union",
            NodeKind::Typedef => "typedef",
            NodeKind::Include => "include",
            NodeKind::PreprocessorDef => "preprocessor_def",
            NodeKind::Template => "template",
            NodeKind::DataClass => "data_class",
            NodeKind::SealedClass => "sealed_class",
            NodeKind::CompanionObject => "companion_object",
            NodeKind::KotlinObject => "kotlin_object",
            NodeKind::KotlinPackage => "kotlin_package",
            NodeKind::Property => "property",
            NodeKind::Mixin => "mixin",
            NodeKind::Extension => "extension",
            NodeKind::Library => "library",
            NodeKind::Delegate => "delegate",
            NodeKind::Event => "event",
            NodeKind::Record => "record",
            NodeKind::CSharpProperty => "csharp_property",
            NodeKind::Procedure => "procedure",
            NodeKind::PascalUnit => "pascal_unit",
            NodeKind::PascalProgram => "pascal_program",
            NodeKind::PascalRecord => "pascal_record",
            #[cfg(feature = "lang-protobuf")]
            NodeKind::ProtoMessage => "proto_message",
            #[cfg(feature = "lang-protobuf")]
            NodeKind::ProtoService => "proto_service",
            #[cfg(feature = "lang-protobuf")]
            NodeKind::ProtoRpc => "proto_rpc",
        }
    }

    /// Parses a string into a `NodeKind`, returning `None` for unrecognized values.
    pub fn from_str(s: &str) -> Option<NodeKind> {
        match s {
            "file" => Some(NodeKind::File),
            "module" => Some(NodeKind::Module),
            "struct" => Some(NodeKind::Struct),
            "enum" => Some(NodeKind::Enum),
            "enum_variant" => Some(NodeKind::EnumVariant),
            "trait" => Some(NodeKind::Trait),
            "function" => Some(NodeKind::Function),
            "method" => Some(NodeKind::Method),
            "impl" => Some(NodeKind::Impl),
            "const" => Some(NodeKind::Const),
            "static" => Some(NodeKind::Static),
            "type_alias" => Some(NodeKind::TypeAlias),
            "field" => Some(NodeKind::Field),
            "macro" => Some(NodeKind::Macro),
            "use" => Some(NodeKind::Use),
            "class" => Some(NodeKind::Class),
            "interface" => Some(NodeKind::Interface),
            "constructor" => Some(NodeKind::Constructor),
            "annotation" => Some(NodeKind::Annotation),
            "annotation_usage" => Some(NodeKind::AnnotationUsage),
            "package" => Some(NodeKind::Package),
            "inner_class" => Some(NodeKind::InnerClass),
            "init_block" => Some(NodeKind::InitBlock),
            "abstract_method" => Some(NodeKind::AbstractMethod),
            "interface_type" => Some(NodeKind::InterfaceType),
            "struct_method" => Some(NodeKind::StructMethod),
            "go_package" => Some(NodeKind::GoPackage),
            "struct_tag" => Some(NodeKind::StructTag),
            "object" => Some(NodeKind::ScalaObject),
            "case_class" => Some(NodeKind::CaseClass),
            "scala_package" => Some(NodeKind::ScalaPackage),
            "val" => Some(NodeKind::ValField),
            "var" => Some(NodeKind::VarField),
            "generic_param" => Some(NodeKind::GenericParam),
            "arrow_function" => Some(NodeKind::ArrowFunction),
            "decorator" => Some(NodeKind::Decorator),
            "export" => Some(NodeKind::Export),
            "namespace" => Some(NodeKind::Namespace),
            "union" => Some(NodeKind::Union),
            "typedef" => Some(NodeKind::Typedef),
            "include" => Some(NodeKind::Include),
            "preprocessor_def" => Some(NodeKind::PreprocessorDef),
            "template" => Some(NodeKind::Template),
            "data_class" => Some(NodeKind::DataClass),
            "sealed_class" => Some(NodeKind::SealedClass),
            "companion_object" => Some(NodeKind::CompanionObject),
            "kotlin_object" => Some(NodeKind::KotlinObject),
            "kotlin_package" => Some(NodeKind::KotlinPackage),
            "property" => Some(NodeKind::Property),
            "mixin" => Some(NodeKind::Mixin),
            "extension" => Some(NodeKind::Extension),
            "library" => Some(NodeKind::Library),
            "delegate" => Some(NodeKind::Delegate),
            "event" => Some(NodeKind::Event),
            "record" => Some(NodeKind::Record),
            "csharp_property" => Some(NodeKind::CSharpProperty),
            "procedure" => Some(NodeKind::Procedure),
            "pascal_unit" => Some(NodeKind::PascalUnit),
            "pascal_program" => Some(NodeKind::PascalProgram),
            "pascal_record" => Some(NodeKind::PascalRecord),
            #[cfg(feature = "lang-protobuf")]
            "proto_message" => Some(NodeKind::ProtoMessage),
            #[cfg(feature = "lang-protobuf")]
            "proto_service" => Some(NodeKind::ProtoService),
            #[cfg(feature = "lang-protobuf")]
            "proto_rpc" => Some(NodeKind::ProtoRpc),
            _ => None,
        }
    }
}

/// Kinds of edges in the code graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    Contains,
    Calls,
    Uses,
    Implements,
    TypeOf,
    Returns,
    DerivesMacro,
    Extends,
    Annotates,
    Receives,
}

#[allow(clippy::should_implement_trait)]
impl EdgeKind {
    /// Returns the string representation of this edge kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeKind::Contains => "contains",
            EdgeKind::Calls => "calls",
            EdgeKind::Uses => "uses",
            EdgeKind::Implements => "implements",
            EdgeKind::TypeOf => "type_of",
            EdgeKind::Returns => "returns",
            EdgeKind::DerivesMacro => "derives_macro",
            EdgeKind::Extends => "extends",
            EdgeKind::Annotates => "annotates",
            EdgeKind::Receives => "receives",
        }
    }

    /// Parses a string into an `EdgeKind`, returning `None` for unrecognized values.
    pub fn from_str(s: &str) -> Option<EdgeKind> {
        match s {
            "contains" => Some(EdgeKind::Contains),
            "calls" => Some(EdgeKind::Calls),
            "uses" => Some(EdgeKind::Uses),
            "implements" => Some(EdgeKind::Implements),
            "type_of" => Some(EdgeKind::TypeOf),
            "returns" => Some(EdgeKind::Returns),
            "derives_macro" => Some(EdgeKind::DerivesMacro),
            "extends" => Some(EdgeKind::Extends),
            "annotates" => Some(EdgeKind::Annotates),
            "receives" => Some(EdgeKind::Receives),
            _ => None,
        }
    }
}

/// Visibility of a code item.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Pub,
    PubCrate,
    PubSuper,
    #[default]
    Private,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pub => "public",
            Self::PubCrate => "pub_crate",
            Self::PubSuper => "pub_super",
            Self::Private => "private",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "public" | "pub" => Some(Self::Pub),
            "pub_crate" => Some(Self::PubCrate),
            "pub_super" => Some(Self::PubSuper),
            "private" => Some(Self::Private),
            _ => None,
        }
    }
}
