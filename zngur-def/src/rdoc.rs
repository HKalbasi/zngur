use std::{cell::OnceCell, iter::Peekable, mem::take, sync::OnceLock, vec::IntoIter};

use crate::*;
use rustdoc_types::{
    Crate, Enum, Function, GenericArg, GenericArgs, Id, Item, ItemSummary, Struct, Type,
};

type LOMap = HashMap<String, LayoutPolicy>;

pub struct RustDocParser {
    items: Peekable<IntoIter<Item>>,
    current: u32,
    crate_map: HashMap<u32, Crate>,
    layout_info: LOMap,
    pub externals: HashMap<u32, Vec<ItemSummary>>,
    spec: ZngurSpec,
}

impl RustDocParser {
    pub fn new(items: Vec<Item>, crate_map: HashMap<u32, Crate>, layout_info: LOMap) -> Self {
        let p = items.into_iter().peekable();
        Self {
            items: p,
            current: 0,
            crate_map,
            layout_info,
            externals: HashMap::new(),
            spec: ZngurSpec::default(),
        }
    }

    pub fn parse(mut self) -> ZngurSpec {
        while self.items.peek().is_some() {
            self.item_into_spec();
        }

        let externals = take(&mut self.externals);

        for each in externals.into_iter() {
            self.current = each.0;
            self.externals_to_items(each.1);
            while self.items.peek().is_some() {
                self.item_into_spec();
            }
        }

        // println!("{:?}", &self.spec);
        self.spec
    }

    fn modify_path(&mut self) {
        let Some(s) = self.summary() else {
            return;
        };
        if s.crate_id == 0 {
            *s.path.first_mut().unwrap() = "crate".into();
        }
    }

    fn externals_to_items(&mut self, summs: Vec<ItemSummary>) {
        let new_items = summs
            .iter()
            .map(|s| {
                self.crate_map
                    .get(&self.current)
                    .unwrap()
                    .index
                    .values()
                    .find(|x| x.name.as_ref() == Some(s.path.last().unwrap()))
                    .unwrap()
                    .clone()
            })
            .collect::<Vec<_>>();
        self.items = new_items.into_iter().peekable();
    }

    fn item_into_spec(&mut self) {
        //TODO:NRB fix this when working with std deps. For now just disable
        // self.modify_path();
        let i = self.items.peek().unwrap();
        match &i.inner {
            ItemEnum::Struct(_) | ItemEnum::Enum(_) => {
                self.convert_adt();
            }
            ItemEnum::Function(_) => {
                self.convert_function();
            }
            ItemEnum::Module(_) => {}
            ItemEnum::ExternCrate { .. } => {}
            ItemEnum::Union(_) => {}
            // Will be handled recursively
            ItemEnum::StructField(_) => {}
            ItemEnum::Variant(_) => {}
            ItemEnum::Trait(_) => {}
            ItemEnum::TraitAlias(_) => {}
            // Handled recursively
            ItemEnum::Impl(_) => {}
            ItemEnum::TypeAlias(_) => {}
            ItemEnum::Constant { .. } => {}
            ItemEnum::Static(_) => {}
            ItemEnum::ExternType => {}
            ItemEnum::Macro(_) => {}
            ItemEnum::ProcMacro(_) => {}
            ItemEnum::Primitive(_) => {}
            ItemEnum::AssocConst { .. } => {}
            ItemEnum::AssocType { .. } => {}
            ItemEnum::Use(_) => {}
        }
        // CONSUMES HERE
        self.items.next();
    }

    fn convert_adt(&mut self) {
        let ty = RustType::Adt(RustPathAndGenerics {
            path: self.path().unwrap(),
            generics: vec![],
            named_generics: vec![],
        });

        let mut ztype = ZngurType {
            ty,
            layout: *self.layout().unwrap_or(&LayoutPolicy::HeapAllocated),
            methods: vec![],
            //TODO:NRB handle dynamically
            wellknown_traits: vec![ZngurWellknownTrait::Drop],
            constructors: vec![],
            fields: vec![],
            cpp_value: None,
            cpp_ref: None,
        };

        self.convert_impls(&mut ztype);
        self.spec.types.push(ztype);
    }

    fn convert_function(&mut self) {
        // skipping builtins
        let Some(fpath) = self.path() else {
            return;
        };
        let item = self.items.peek().unwrap().clone();
        let ItemEnum::Function(f) = &item.inner else {
            return;
        };

        let inputs = f
            .sig
            .inputs
            .iter()
            .map(|(_name, ty)| self.convert_rdt(ty).unwrap())
            .collect::<Vec<_>>();

        let output = if f.sig.output.is_some() {
            self.convert_rdt(f.sig.output.as_ref().unwrap()).unwrap()
        } else {
            RustType::UNIT
        };

        self.spec.funcs.push(ZngurFn {
            path: RustPathAndGenerics {
                path: fpath,
                generics: vec![],
                named_generics: vec![],
            },
            inputs,
            output,
        });
    }

    fn convert_impls(&mut self, ztype: &mut ZngurType) {
        let item = self.items.peek().unwrap();
        let impl_ids = match &item.inner {
            ItemEnum::Struct(s) => s.impls.clone(),
            ItemEnum::Enum(e) => e.impls.clone(),
            _ => return,
        };
        let items = impl_ids
            .iter()
            //Id to item
            .filter_map(|i| {
                self.crate_map
                    .get(&self.current)
                    .unwrap()
                    .index
                    .get(i)
                    .clone()
            })
            //item to impl type
            .filter_map(|x| {
                let ItemEnum::Impl(imp) = &x.inner else {
                    return None;
                };
                Some(imp.clone())
            })
            .collect::<Vec<_>>();
        items.iter().for_each(|i| {
            match &i.trait_ {
                None => {
                    // Native impl, add methods
                    for method_id in &i.items {
                        let mut method = self.convert_method(method_id);
                        // outputs self
                        if let RustType::Adt(s) = &method.output
                            && s.path == vec!["Self"]
                        {
                            method.output = ztype.ty.clone();
                        }
                        ztype.methods.push(ZngurMethodDetails {
                            data: method,
                            use_path: None,
                            deref: None,
                        });
                    }
                }
                Some(t) => {
                    // Trait impl, add to wellknown_traits
                    match t.path.as_str() {
                        "Debug" => {
                            ztype.wellknown_traits.push(ZngurWellknownTrait::Debug);
                        }
                        "Drop" => {
                            ztype.wellknown_traits.push(ZngurWellknownTrait::Drop);
                        }
                        "Unsized" => {
                            ztype.wellknown_traits.push(ZngurWellknownTrait::Unsized);
                        }
                        "Copy" => {
                            ztype.wellknown_traits.push(ZngurWellknownTrait::Copy);
                        }
                        _ => {
                            println!("Unsupported trait impl: {}", t.path)
                        }
                    }
                }
            }
        });
    }

    fn convert_method(&mut self, fid: &Id) -> ZngurMethod {
        let item = self
            .crate_map
            .get(&self.current)
            .unwrap()
            .index
            .get(fid)
            .unwrap()
            .clone();
        let ItemEnum::Function(func) = &item.inner else {
            //TODO:NRB keep this? better syntax?
            panic!()
        };

        let selff = func
            .sig
            .inputs
            .iter()
            .find(|(name, _ty)| name == "self")
            .map(|(_name, ty)| ty);
        let receiver = match selff {
            None => ZngurMethodReceiver::Static,
            Some(rec) => {
                match rec {
                    Type::BorrowedRef {
                        lifetime,
                        is_mutable,
                        type_,
                    } => {
                        if *is_mutable {
                            ZngurMethodReceiver::Ref(Mutability::Mut)
                        } else {
                            ZngurMethodReceiver::Ref(Mutability::Not)
                        }
                    }
                    Type::Generic(s) => {
                        // check s == "Self"?
                        ZngurMethodReceiver::Move
                    }
                    // Maybe check for borrow ref and all others default to move?
                    _ => panic!(),
                }
            }
        };

        //TODO:NRB fix crate local type names
        let inputs = func
            .sig
            .inputs
            .iter()
            .filter(|(name, _ty)| name != "self")
            .map(|(_name, ty)| self.convert_rdt(ty).unwrap())
            .collect::<Vec<_>>();

        let output = if func.sig.output.is_some() {
            self.convert_rdt(func.sig.output.as_ref().unwrap()).unwrap()
        } else {
            RustType::UNIT
        };
        ZngurMethod {
            name: item.name.clone().unwrap(),
            inputs,
            generics: vec![],
            receiver,
            output,
        }
    }

    fn convert_rdt(&mut self, value: &rustdoc_types::Type) -> Result<RustType, ()> {
        match value {
            Type::ResolvedPath(path) => {
                let pinfo = self
                    .crate_map
                    .get(&self.current)
                    .unwrap()
                    .paths
                    .get(&path.id)
                    .unwrap();
                if pinfo.crate_id != 0 {
                    self.externals
                        .entry(pinfo.crate_id)
                        .and_modify(|x| x.push(pinfo.clone()))
                        .or_insert(vec![pinfo.clone()]);
                }
                // Convert rustdoc_types::Path to RustPathAndGenerics
                let path_vec = path.path.split("::").map(|s| s.to_string()).collect();
                let mut generics = vec![];
                if let Some(args) = &path.args {
                    match &**args {
                        GenericArgs::AngleBracketed { args, constraints } => {
                            args.iter().for_each(|x| {
                                if let GenericArg::Type(t) = x {
                                    generics.push(self.convert_rdt(t).unwrap());
                                }
                            })
                        }
                        //TODO:NRB finish impl
                        GenericArgs::Parenthesized { inputs, output } => {}
                        GenericArgs::ReturnTypeNotation => {}
                    }
                }
                let named_generics = vec![];
                Ok(RustType::Adt(RustPathAndGenerics {
                    path: path_vec,
                    generics,
                    named_generics,
                }))
            }
            Type::DynTrait(dyn_trait) => {
                // Convert the first trait to RustTrait, ignore others for now
                if let Some(first_trait) = dyn_trait.traits.first() {
                    let trait_path = first_trait
                        .trait_
                        .path
                        .split("::")
                        .map(|s| s.to_string())
                        .collect();
                    let rust_trait = RustTrait::Normal(RustPathAndGenerics {
                        path: trait_path,
                        generics: vec![],
                        named_generics: vec![],
                    });
                    Ok(RustType::Dyn(rust_trait, vec![]))
                } else {
                    Ok(RustType::UNIT)
                }
            }
            Type::Generic(name) => {
                // For generic types, we'll need to handle them in context
                // For now, return a placeholder
                Ok(RustType::Adt(RustPathAndGenerics {
                    path: vec![name.clone()],
                    generics: vec![],
                    named_generics: vec![],
                }))
            }
            Type::Primitive(p) => Ok(RustType::Primitive(PrimitiveRustType::from(p.clone()))),
            Type::FunctionPointer(function_pointer) => {
                // Convert function pointer signature to RustType
                let inputs = function_pointer
                    .sig
                    .inputs
                    .iter()
                    .map(|(_, ty)| self.convert_rdt(ty).unwrap())
                    .collect();
                let output = function_pointer
                    .sig
                    .output
                    .as_ref()
                    .map(|ty| Box::new(self.convert_rdt(ty).unwrap()))
                    .unwrap_or(Box::new(RustType::UNIT));

                Ok(RustType::Dyn(
                    RustTrait::Fn {
                        name: "fn".to_string(),
                        inputs,
                        output,
                    },
                    vec![],
                ))
            }
            Type::Tuple(items) => {
                let converted_items = items
                    .iter()
                    .map(|ty| self.convert_rdt(ty).unwrap())
                    .collect();
                Ok(RustType::Tuple(converted_items))
            }
            Type::Slice(item_type) => {
                let converted_type = Box::new(self.convert_rdt(*&item_type).unwrap());
                Ok(RustType::Slice(converted_type))
            }
            Type::Array { type_, len: _ } => {
                // TODO:NRB Maybe add array variant later
                let converted_type = Box::new(self.convert_rdt(&*type_).unwrap());
                Ok(RustType::Slice(converted_type))
            }
            Type::Pat {
                type_,
                __pat_unstable_do_not_use: _,
            } => {
                // Pattern types are experimental, treat as the underlying type
                Ok(self.convert_rdt(&*type_).unwrap())
            }
            Type::ImplTrait(_generic_bounds) => {
                // Impl trait types are opaque, treat as unit for now
                // TODO: Implement proper impl trait handling
                Ok(RustType::UNIT)
            }
            Type::Infer => {
                // Inferred types, treat as unit for now
                Ok(RustType::UNIT)
            }
            Type::RawPointer { is_mutable, type_ } => {
                let mutability = if *is_mutable {
                    Mutability::Mut
                } else {
                    Mutability::Not
                };
                let converted_type = Box::new(self.convert_rdt(&*type_).unwrap());
                Ok(RustType::Raw(mutability, converted_type))
            }
            Type::BorrowedRef {
                lifetime: _,
                is_mutable,
                type_,
            } => {
                let mutability = if *is_mutable {
                    Mutability::Mut
                } else {
                    Mutability::Not
                };
                let converted_type = Box::new(self.convert_rdt(&*type_).unwrap());
                Ok(RustType::Ref(mutability, converted_type))
            }
            Type::QualifiedPath {
                name: _,
                args: _,
                self_type,
                trait_: _,
            } => {
                // Qualified paths like <T as Trait>::Item
                // For now, treat as the self type
                Ok(self.convert_rdt(&*self_type).unwrap())
            }
        }
    }

    fn summary(&mut self) -> Option<&mut ItemSummary> {
        self.crate_map
            .get_mut(&self.current)?
            .paths
            .get_mut(&self.items.peek()?.id)
    }

    fn path(&mut self) -> Option<Vec<String>> {
        let item = self.items.peek()?.clone();
        //TODO: NRB check what we want to do when there is no summary (builtin functions like "fmt").
        // Current impl skips them
        let p = self.summary()?;
        // Weird function naming syntax, no "crate::" when other types have it.  TODO:NRB check if on purpose.
        if let ItemEnum::Function(_) = item.inner {
            return Some(vec![item.name.unwrap()]);
        }
        Some(p.path.clone())
    }

    fn layout(&mut self) -> Option<&LayoutPolicy> {
        let s = self.summary()?.clone();
        return match s.crate_id {
            0 => self.layout_info.get(s.path.last()?),
            _ => self.layout_info.get(s.path.join("::").as_str()),
        };
    }
}

impl ZngurSpec {
    pub fn from_crate(crate_map: HashMap<u32, Crate>, layout_info: LOMap) -> Self {
        let items = crate_map
            .get(&0)
            .unwrap()
            .index
            .clone()
            .into_values()
            .collect();
        let parser = RustDocParser::new(items, crate_map, layout_info);
        parser.parse()
    }
}
