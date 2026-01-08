use std::collections::{HashMap, HashSet};

use zngur_def::{
    Merge, RustPathAndGenerics, RustTrait, RustType, TypeVar, ZngurConstructor, ZngurField,
    ZngurMethod, ZngurMethodDetails, ZngurType,
};

fn matches_template<'a, 'b>(
    ty: &'a RustType,
    generic: &'b RustType,
    mapping: &mut HashMap<&'b TypeVar, &'a RustType>,
) -> bool {
    fn match_lists<'a, 'b>(
        v1: &'a [RustType],
        v2: &'b [RustType],
        mapping: &mut HashMap<&'b TypeVar, &'a RustType>,
    ) -> bool {
        v1.len() == v2.len() && match_iters(v1, v2, mapping)
    }

    fn match_iters<'a, 'b>(
        i1: impl IntoIterator<Item = &'a RustType>,
        i2: impl IntoIterator<Item = &'b RustType>,
        mapping: &mut HashMap<&'b TypeVar, &'a RustType>,
    ) -> bool {
        i1.into_iter()
            .zip(i2)
            .all(|(ty1, ty2)| matches_template(ty1, ty2, mapping))
    }

    fn match_generics<'a, 'b>(
        t1: &'a RustPathAndGenerics,
        t2: &'b RustPathAndGenerics,
        mapping: &mut HashMap<&'b TypeVar, &'a RustType>,
    ) -> bool {
        // For now named generics must be in the same order
        t1.path == t2.path
            && match_lists(&t1.generics, &t2.generics, mapping)
            && t1.named_generics.len() == t2.named_generics.len()
            && t1
                .named_generics
                .iter()
                .zip(t2.named_generics.iter())
                .all(|((n1, t1), (n2, t2))| n1 == n2 && matches_template(t1, t2, mapping))
    }

    fn match_trait<'a, 'b>(
        t1: &'a RustTrait,
        t2: &'b RustTrait,
        mapping: &mut HashMap<&'b TypeVar, &'a RustType>,
    ) -> bool {
        match (t1, t2) {
            (RustTrait::Normal(t1), RustTrait::Normal(t2)) => match_generics(t1, t2, mapping),
            (
                RustTrait::Fn {
                    name: n1,
                    inputs: i1,
                    output: o1,
                },
                RustTrait::Fn {
                    name: n2,
                    inputs: i2,
                    output: o2,
                },
            ) => n1 == n2 && match_lists(i1, i2, mapping) && matches_template(o1, o2, mapping),
            (_, _) => false,
        }
    }

    match (ty, generic) {
        (ty, RustType::TypeVar(v)) => mapping
            .insert(v, ty)
            .map(|prev_binding| prev_binding == ty)
            .unwrap_or(true),
        (RustType::Primitive(p1), RustType::Primitive(p2)) => p1 == p2,
        (RustType::Ref(m1, t1), RustType::Ref(m2, t2))
        | (RustType::Raw(m1, t1), RustType::Raw(m2, t2)) => {
            m1 == m2 && matches_template(t1, t2, mapping)
        }
        (RustType::Boxed(t1), RustType::Boxed(t2)) | (RustType::Slice(t1), RustType::Slice(t2)) => {
            matches_template(t1, t2, mapping)
        }
        (RustType::Dyn(t1, b1), RustType::Dyn(t2, b2))
        | (RustType::Impl(t1, b1), RustType::Impl(t2, b2)) => {
            (b1 == b2) && match_trait(t1, t2, mapping)
        }
        (RustType::Tuple(tys1), RustType::Tuple(tys2)) => match_lists(tys1, tys2, mapping),
        (RustType::Adt(adt1), RustType::Adt(adt2)) => match_generics(adt1, adt2, mapping),
        (_, _) => false,
    }
}

#[derive(Debug)]
enum SubstitutionError<'a> {
    UnboundVar(&'a TypeVar),
    UndefinedType,
}

fn substitute_vars<'a>(
    ty: &'a RustType,
    mapping: &HashMap<&TypeVar, &RustType>,
    defined_types: &HashSet<RustType>,
) -> Result<RustType, SubstitutionError<'a>> {
    fn substitute_vec<'a>(
        vec: &'a Vec<RustType>,
        mapping: &HashMap<&TypeVar, &RustType>,
    ) -> Result<Vec<RustType>, &'a TypeVar> {
        vec.iter().map(|ty| substitute_type(ty, mapping)).collect()
    }

    fn substitute_generics<'a>(
        path_and_generics: &'a RustPathAndGenerics,
        mapping: &HashMap<&TypeVar, &RustType>,
    ) -> Result<RustPathAndGenerics, &'a TypeVar> {
        let RustPathAndGenerics {
            path,
            generics,
            named_generics,
        } = path_and_generics;
        let result = RustPathAndGenerics {
            path: path.clone(),
            generics: substitute_vec(generics, mapping)?,
            named_generics: named_generics
                .iter()
                .map(|(name, ty)| substitute_type(ty, mapping).map(|ty| (name.clone(), ty)))
                .collect::<Result<_, _>>()?,
        };
        Ok(result)
    }

    fn substitute_trait<'a>(
        rust_trait: &'a RustTrait,
        mapping: &HashMap<&TypeVar, &RustType>,
    ) -> Result<RustTrait, &'a TypeVar> {
        let result = match rust_trait {
            RustTrait::Normal(path_and_generics) => {
                RustTrait::Normal(substitute_generics(path_and_generics, mapping)?)
            }
            RustTrait::Fn {
                name,
                inputs,
                output,
            } => RustTrait::Fn {
                name: name.clone(),
                inputs: substitute_vec(inputs, mapping)?,
                output: Box::new(substitute_type(output, mapping)?),
            },
        };
        Ok(result)
    }

    fn substitute_type<'a>(
        ty: &'a RustType,
        mapping: &HashMap<&TypeVar, &RustType>,
    ) -> Result<RustType, &'a TypeVar> {
        let ty = match ty {
            RustType::TypeVar(v) => match mapping.get(v) {
                Some(ty) => (*ty).clone(),
                None => return Err(v),
            },
            p @ RustType::Primitive(_) => p.clone(),
            RustType::Ref(m, t) => RustType::Ref(*m, Box::new(substitute_type(t, mapping)?)),
            RustType::Raw(m, t) => RustType::Raw(*m, Box::new(substitute_type(t, mapping)?)),
            RustType::Boxed(t) => RustType::Boxed(Box::new(substitute_type(t, mapping)?)),
            RustType::Slice(t) => RustType::Slice(Box::new(substitute_type(t, mapping)?)),
            // TODO: Recurse
            RustType::Dyn(rust_trait, bounds) => {
                RustType::Dyn(substitute_trait(rust_trait, mapping)?, bounds.clone())
            }
            RustType::Impl(rust_trait, bounds) => {
                RustType::Impl(substitute_trait(rust_trait, mapping)?, bounds.clone())
            }
            RustType::Tuple(tys) => RustType::Tuple(substitute_vec(tys, mapping)?),
            RustType::Adt(path_and_generics) => {
                RustType::Adt(substitute_generics(path_and_generics, mapping)?)
            }
        };
        Ok(ty)
    }

    match substitute_type(ty, mapping) {
        Ok(ty) => {
            let mut curr_ty = &ty;
            let type_defined = loop {
                match curr_ty {
                    // Primitives and Unit are automatically defined
                    RustType::Primitive(_) => break true,
                    RustType::Tuple(t) if t.is_empty() => break true,
                    // Ref and raw types are automatically defined
                    RustType::Ref(_, inner) | RustType::Raw(_, inner) => curr_ty = inner,
                    ty => break defined_types.contains(&ty),
                }
            };
            if type_defined {
                Ok(ty)
            } else {
                Err(SubstitutionError::UndefinedType)
            }
        }
        Err(var) => Err(SubstitutionError::UnboundVar(var)),
    }
}

fn substitute_method_vars<'a>(
    m: &'a ZngurMethodDetails,
    mapping: &HashMap<&TypeVar, &RustType>,
    defined_types: &HashSet<RustType>,
) -> Result<ZngurMethodDetails, SubstitutionError<'a>> {
    let ZngurMethodDetails {
        data:
            ZngurMethod {
                name,
                generics,
                receiver,
                inputs,
                output,
            },
        use_path,
        deref,
    } = m;
    Ok(ZngurMethodDetails {
        data: ZngurMethod {
            name: name.clone(),
            generics: generics
                .iter()
                .map(|ty| substitute_vars(ty, mapping, defined_types))
                .collect::<Result<_, _>>()?,
            receiver: *receiver,
            inputs: inputs
                .iter()
                .map(|ty| substitute_vars(ty, mapping, defined_types))
                .collect::<Result<_, _>>()?,
            output: substitute_vars(output, mapping, defined_types)?,
        },
        use_path: use_path.clone(),
        deref: match deref {
            Some((ty, mutability)) => {
                Some((substitute_vars(&ty, mapping, defined_types)?, *mutability))
            }
            None => None,
        },
    })
}

pub fn try_match_template(
    ty: &RustType,
    template: &ZngurType,
    defined_types: &HashSet<RustType>,
) -> Option<TemplateMatch> {
    let mut mapping = HashMap::new();
    if !matches_template(ty, &template.ty, &mut mapping) {
        return None;
    }
    let ZngurType {
        ty: template_ty,
        layout,
        wellknown_traits,
        methods,
        constructors,
        fields,
        cpp_ref,
        cpp_value,
    } = template;
    debug_assert_eq!(
        substitute_vars(template_ty, &mapping, defined_types).unwrap(),
        *ty
    );
    let new_ty = ZngurType {
        ty: ty.clone(),
        layout: *layout,
        wellknown_traits: wellknown_traits.clone(),
        methods: methods
            .iter()
            .filter_map(
                |method| match substitute_method_vars(method, &mapping, defined_types) {
                    Ok(m) => Some(m),
                    Err(SubstitutionError::UndefinedType) => None,
                    Err(SubstitutionError::UnboundVar(var)) => unreachable!(
                        "Unbound type variable {} in method {} in template {} for type {}",
                        var.0, method.data.name, template.ty, ty
                    ),
                },
            )
            .collect(),
        constructors: constructors
            .iter()
            .filter_map(|constructor| {
                match constructor
                    .inputs
                    .iter()
                    .map(|(name, ty)| {
                        substitute_vars(ty, &mapping, defined_types).map(|ty| (name.clone(), ty))
                    })
                    .collect()
                {
                    Ok(inputs) => Some(ZngurConstructor {
                        name: constructor.name.clone(),
                        inputs,
                    }),
                    Err(SubstitutionError::UndefinedType) => None,
                    Err(SubstitutionError::UnboundVar(var)) => unreachable!(
                        "Unbound type variable {} in constructor {:?} in template {} for type {}",
                        var.0, constructor.name, template.ty, ty
                    ),
                }
            })
            .collect(),
        fields: fields
            .iter()
            .filter_map(
                |field| match substitute_vars(&field.ty, &mapping, defined_types) {
                    Ok(ty) => Some(ZngurField {
                        name: field.name.clone(),
                        ty,
                        offset: field.offset,
                    }),
                    Err(SubstitutionError::UndefinedType) => None,
                    Err(SubstitutionError::UnboundVar(var)) => unreachable!(
                        "Unbound type variable {} in field {} in template {} for type {}",
                        var.0, field.name, template.ty, ty
                    ),
                },
            )
            .collect(),
        cpp_value: cpp_value.clone(),
        cpp_ref: cpp_ref.clone(),
    };
    Some(TemplateMatch(new_ty))
}

// Represents a ZngurType created from a template type
pub struct TemplateMatch(ZngurType);

impl Merge<ZngurType> for TemplateMatch {
    fn merge(self, into: &mut ZngurType) -> zngur_def::MergeResult {
        let TemplateMatch(mut ty) = self;
        // The concrete type's layout should override the template's layout without causing a conflict
        if into.layout.is_some() {
            ty.layout = None;
        }
        ty.merge(into)
    }
}
