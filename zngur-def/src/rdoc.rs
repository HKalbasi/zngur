use std::default;

use crate::*;
use rustdoc_types::{Crate, Type};

impl From<rustdoc_types::Crate> for ZngurSpec {
    fn from(value: rustdoc_types::Crate) -> Self {
        //TODO: dynamic ref: strum crate maybe?
        let hardcoded_traits = vec!["Debug", "Drop", "Unsized", "Copy"];
        let crate_name = value.index.get(&value.root).unwrap().clone().name.unwrap();
        let mut spec = Self::default();
        for (id, item) in value.index.iter() {
            //match the type
            match &item.inner {
                ItemEnum::Struct(s) => {
                    let mut path = value.paths.get(&id).unwrap().clone().path;
                    convert_path(&mut path, &crate_name);
                    //TODO: add generics
                    // Get from here
                    // let generics = s.generics;
                    let ty = RustType::Adt(RustPathAndGenerics {
                        path: path,
                        generics: vec![],
                        named_generics: vec![],
                    });
                    let mut ztype = ZngurType {
                        ty,
                        // TODO: figure this out.
                        layout: LayoutPolicy::HeapAllocated,
                        methods: vec![],
                        wellknown_traits: vec![],
                        constructors: vec![],
                        fields: vec![],
                        cpp_value: None,
                        cpp_ref: None,
                    };
                    s.impls
                        .iter()
                        .map(|x| value.index.get(x).unwrap())
                        .for_each(|x| 
                            //Pretty sure all impls, replace with if let/let else
                            match &x.inner {
                            ItemEnum::Impl(i) => {
                                match &i.trait_ {
                                    None => {
                                        for each in &i.items {
                                            let item = value.index.get(each).unwrap();
                                            let ItemEnum::Function(func) = item.inner.clone() else {return};
                                            let inp = func.sig.inputs.iter().map(|(name, ty)|{
                                                match ty {
                                                    //TODO: non-primitives
                                                    Type::Primitive(p) => {
                                                        RustType::Primitive(PrimitiveRustType::from(p.to_owned()))
                                                    }
                                                    _=>{
                                                        RustType::Primitive(PrimitiveRustType::from(String::from("u8")))
                                                    }

                                                }
                                            }).collect::<Vec<_>>();
                                            let output = if let Some(o) = func.sig.output {
                                                //TODO: add type decl
                                                RustType::UNIT
                                            }else {
                                                RustType::UNIT
                                            };
                                            let method = ZngurMethod {
                                                name: item.name.clone().unwrap(),
                                                inputs: inp,
                                                generics: vec![],
                                                //TODO: fix
                                                receiver: ZngurMethodReceiver::Static,
                                                output 
                                            };
                                            //TODO: use_path and deref
                                            ztype.methods.push(ZngurMethodDetails { data: method, use_path: None, deref: None })
                                        }
                                    } // This is native impl, not impl trait. add to
                                    // methods
                                    Some(t) => match t.path.as_str() {
                                        "Debug" => {
                                            ztype.wellknown_traits.push(ZngurWellknownTrait::Debug);
                                        }
                                        "Drop" => {
                                            ztype.wellknown_traits.push(ZngurWellknownTrait::Drop);
                                        }
                                        "Unsized" => {
                                            ztype
                                                .wellknown_traits
                                                .push(ZngurWellknownTrait::Unsized);
                                        }
                                        "Copy" => {
                                            ztype.wellknown_traits.push(ZngurWellknownTrait::Copy);
                                        }
                                        _ => {
                                            println!("Unsupported trait impl: {}", t.path)
                                        }
                                    },
                                }
                            }
                            _ => {}
                        });
                    spec.types.push(ztype);
                }
                ItemEnum::Module(_) => {
                    // Handle module items if needed
                }
                ItemEnum::ExternCrate { .. } => {
                    // Handle extern crate items if needed
                }
                ItemEnum::Union(_) => {}
                ItemEnum::StructField(_) => {
                    // Handle struct field items if needed
                }
                ItemEnum::Enum(e) => {
                    let mut path = value.paths.get(&id).unwrap().clone().path;
                    convert_path(&mut path, &crate_name);
                    //TODO: add generics
                    // Get from here
                    // let generics = e.generics;
                    let ty = RustType::Adt(RustPathAndGenerics {
                        path: path,
                        generics: vec![],
                        named_generics: vec![],
                    });
                    let mut ztype = ZngurType {
                        ty,
                        // TODO: figure this out.
                        layout: LayoutPolicy::HeapAllocated,
                        methods: vec![],
                        wellknown_traits: vec![],
                        constructors: vec![],
                        fields: vec![],
                        cpp_value: None,
                        cpp_ref: None,
                    };
                    e.impls
                        .iter()
                        .map(|x| value.index.get(x).unwrap())
                        .for_each(|x| 
                            //Pretty sure all impls, replace with if let/let else
                            match &x.inner {
                            ItemEnum::Impl(i) => {
                                match &i.trait_ {
                                    None => {
                                        for each in &i.items {
                                            let item = value.index.get(each).unwrap();
                                            let ItemEnum::Function(func) = item.inner.clone() else {return};
                                            let inp = func.sig.inputs.iter().map(|(name, ty)|{
                                                match ty {
                                                    //TODO: non-primitives
                                                    Type::Primitive(p) => {
                                                        RustType::Primitive(PrimitiveRustType::from(p.to_owned()))
                                                    }
                                                    _=>{
                                                        RustType::Primitive(PrimitiveRustType::from(String::from("u8")))
                                                    }

                                                }
                                            }).collect::<Vec<_>>();
                                            let output = if let Some(o) = func.sig.output {
                                                //TODO: add type decl
                                                RustType::UNIT
                                            }else {
                                                RustType::UNIT
                                            };
                                            let method = ZngurMethod {
                                                name: item.name.clone().unwrap(),
                                                inputs: inp,
                                                generics: vec![],
                                                //TODO: fix
                                                receiver: ZngurMethodReceiver::Static,
                                                output 
                                            };
                                            //TODO: use_path and deref
                                            ztype.methods.push(ZngurMethodDetails { data: method, use_path: None, deref: None })
                                        }
                                    } // This is native impl, not impl trait. add to
                                    // methods
                                    Some(t) => match t.path.as_str() {
                                        "Debug" => {
                                            ztype.wellknown_traits.push(ZngurWellknownTrait::Debug);
                                        }
                                        "Drop" => {
                                            ztype.wellknown_traits.push(ZngurWellknownTrait::Drop);
                                        }
                                        "Unsized" => {
                                            ztype
                                                .wellknown_traits
                                                .push(ZngurWellknownTrait::Unsized);
                                        }
                                        "Copy" => {
                                            ztype.wellknown_traits.push(ZngurWellknownTrait::Copy);
                                        }
                                        _ => {
                                            println!("Unsupported trait impl: {}", t.path)
                                        }
                                    },
                                }
                            }
                            _ => {}
                        });
                    spec.types.push(ztype);
                }
                ItemEnum::Variant(_) => {
                    // Handle variant items if needed
                }
                ItemEnum::Function(_) => {
                    // Handle function items if needed
                }
                ItemEnum::Trait(_) => {
                    // Handle trait items if needed
                }
                ItemEnum::TraitAlias(_) => {
                    // Handle trait alias items if needed
                }
                ItemEnum::Impl(_) => {
                    // Handle impl items if needed
                }
                ItemEnum::TypeAlias(_) => {
                    // Handle type alias items if needed
                }
                ItemEnum::Constant { .. } => {
                    // Handle constant items if needed
                }
                ItemEnum::Static(_) => {
                    // Handle static items if needed
                }
                ItemEnum::ExternType => {
                    // Handle extern type items if needed
                }
                ItemEnum::Macro(_) => {
                    // Handle macro items if needed
                }
                ItemEnum::ProcMacro(_) => {
                    // Handle proc macro items if needed
                }
                ItemEnum::Primitive(_) => {
                    // Handle primitive items if needed
                }
                ItemEnum::AssocConst { .. } => {
                    // Handle associated constant items if needed
                }
                ItemEnum::AssocType { .. } => {
                    // Handle associated type items if needed
                }
                ItemEnum::Use(_) => {
                    // Handle use items if needed
                }
            }
        }
        spec
    }
}

// HELPER FUNCTIONS

fn convert_path(raw: &mut Vec<String>, crate_name: &String) {
    let f = raw.first_mut().unwrap();
    if f == crate_name {
        *f = "crate".into();
    }
}
