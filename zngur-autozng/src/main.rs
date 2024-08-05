use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RustdocRustType {
    BorrowedRef {
        mutable: bool,
        #[serde(rename = "type")]
        inner: Box<RustdocRustType>,
    },
    RawPointer {
        mutable: bool,
        #[serde(rename = "type")]
        inner: Box<RustdocRustType>,
    },
    Primitive(String),
    Generic(String),
    Tuple(Vec<RustdocRustType>),
    Slice(Box<RustdocRustType>),
    ResolvedPath {
        name: String,
    },
    QualifiedPath {},
}

impl RustdocRustType {
    fn render(&self) -> String {
        match self {
            RustdocRustType::BorrowedRef {
                mutable: false,
                inner,
            } => format!("&{}", inner.render()),
            RustdocRustType::BorrowedRef {
                mutable: true,
                inner,
            } => format!("&mut {}", inner.render()),
            RustdocRustType::RawPointer { .. } => todo!(),
            RustdocRustType::Primitive(n) => n.clone(),
            RustdocRustType::Generic(n) => n.clone(),
            RustdocRustType::Tuple(_) => todo!(),
            RustdocRustType::Slice(_) => todo!(),
            RustdocRustType::ResolvedPath { name } => name.clone(),
            RustdocRustType::QualifiedPath {} => todo!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RustdocFunctionDecl {
    inputs: Vec<(String, RustdocRustType)>,
    output: Option<RustdocRustType>,
    #[serde(flatten)]
    other_fields: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RustdocItemInner {
    Function {
        decl: RustdocFunctionDecl,
        #[serde(flatten)]
        other_fields: Value,
    },
    Struct {
        impls: Vec<String>,
        #[serde(flatten)]
        other_fields: Value,
    },
    Impl {
        items: Vec<String>,
        #[serde(rename = "trait")]
        for_trait: Option<serde_json::Value>,
        #[serde(flatten)]
        other_fields: Value,
    },
    Module {
        #[serde(flatten)]
        other_fields: Value,
    },
    StructField {
        #[serde(flatten)]
        other_fields: Value,
    },
    Import {
        #[serde(flatten)]
        other_fields: Value,
    },
    AssocType {
        #[serde(flatten)]
        other_fields: Value,
    },
    Variant {
        #[serde(flatten)]
        other_fields: Value,
    },
    TypeAlias {
        #[serde(flatten)]
        other_fields: Value,
    },
    Enum {
        impls: Vec<String>,
        #[serde(flatten)]
        other_fields: Value,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct RustdocItem {
    name: Option<String>,
    inner: RustdocItemInner,
    #[serde(flatten)]
    other_fields: Value,
}

#[derive(Serialize, Deserialize)]
struct RustdocOutput {
    index: HashMap<String, RustdocItem>,
}

fn main() {
    let s = std::fs::read_to_string("./doc.json").unwrap();
    let d: RustdocOutput = serde_json::from_str(&s).unwrap();
    for x in &d.index {
        if let RustdocItemInner::Struct { impls, .. } | RustdocItemInner::Enum { impls, .. } =
            &x.1.inner
        {
            println!("type crate::{} {{", x.1.name.as_ref().unwrap());
            println!("    #heap_allocated;");
            for imp in impls {
                let imp = &d.index[imp];
                // dbg!(imp);

                if let RustdocItemInner::Impl {
                    items, for_trait, ..
                } = &imp.inner
                {
                    if for_trait.is_some() {
                        continue;
                    }
                    for item in items {
                        let item = &d.index[item];
                        if let RustdocItemInner::Function {
                            decl: RustdocFunctionDecl { inputs, output, .. },
                            ..
                        } = &item.inner
                        {
                            print!("    fn {}(", item.name.as_deref().unwrap());
                            let mut first = true;
                            for (name, ty) in inputs {
                                if !first {
                                    print!(", ");
                                }
                                first = false;
                                if name == "self" {
                                    print!("self");
                                    continue;
                                }
                                print!("{}", ty.render());
                            }
                            print!(")");
                            if let Some(output) = output {
                                print!(" -> {}", output.render());
                            }
                            println!(";");
                        }
                    }
                }
            }
            println!("}}");
        }
    }
}
