use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    ParseContext, Span, Spanned, Token, ZngParser,
    conditional::{MatchPattern, MatchPatternParse, Matchable, MatchableParse},
    spanned,
};
use chumsky::prelude::*;

pub trait RustCfgProvider {
    /// Gets values assoceated with a config key if it's present.
    fn get_cfg(&self, key: &str) -> Option<Vec<String>>;
    /// Gets a list of feature names that are enabeled
    fn get_features(&self) -> Vec<String>;
}

pub struct InMemoryRustCfgProvider {
    cfg: Rc<HashMap<String, Vec<String>>>,
    features: Rc<HashSet<String>>,
}

impl InMemoryRustCfgProvider {
    pub fn new<CfgPairs, CfgKey, CfgValues, CfgValue, FeatureValues, FeatureValue>(
        cfg_vlaues: CfgPairs,
        features: FeatureValues,
    ) -> Self
    where
        CfgPairs: IntoIterator<Item = (CfgKey, CfgValues)>,
        CfgKey: AsRef<str>,
        CfgValues: IntoIterator<Item = CfgValue>,
        CfgValue: AsRef<str>,
        FeatureValues: IntoIterator<Item = FeatureValue>,
        FeatureValue: AsRef<str>,
    {
        InMemoryRustCfgProvider {
            cfg: Rc::new(
                cfg_vlaues
                    .into_iter()
                    .map(|(key, values)| {
                        (
                            key.as_ref().to_string(),
                            values
                                .into_iter()
                                .map(|value| value.as_ref().to_string())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect(),
            ),
            features: Rc::new(
                features
                    .into_iter()
                    .map(|feature| feature.as_ref().to_string())
                    .collect(),
            ),
        }
    }
}

impl RustCfgProvider for InMemoryRustCfgProvider {
    fn get_cfg(&self, key: &str) -> Option<Vec<String>> {
        self.cfg.get(key).map(|values| values.to_vec())
    }
    fn get_features(&self) -> Vec<String> {
        self.features.iter().cloned().collect()
    }
}

/// pull config values from the environment (for build scripts)
pub struct CargoEnvRustCfgProvider;

impl CargoEnvRustCfgProvider {
    fn split_values(values: &str) -> Vec<String> {
        values.split(",").map(str::to_owned).collect()
    }
}

const CARGO_FEATURE_PREFIX: &str = "CARGO_FEATURE_";
const CARGO_CFG_PREFIX: &str = "CARGO_CFG_";

impl RustCfgProvider for CargoEnvRustCfgProvider {
    fn get_cfg(&self, key: &str) -> Option<Vec<String>> {
        let key = key.to_uppercase();
        std::env::var(format!("{CARGO_CFG_PREFIX}{key}"))
            .map(|value| Self::split_values(&value))
            .ok()
    }

    fn get_features(&self) -> Vec<String> {
        let mut features = HashSet::new();
        // features exist in two places
        // CARGO_FEATURE_{name} and (msrv = 1.85) CARGO_CFG_FEATURE=<name,...>
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
        for (k, _) in std::env::vars_os() {
            // no panic if not unicode
            let Some(k) = k.to_str() else {
                continue;
            };
            if let Some(feature) = k.strip_prefix(CARGO_FEATURE_PREFIX) {
                features.insert(feature.to_lowercase());
            }
        }
        if let Ok(values) = std::env::var("CARGO_CFG_FEATURE") {
            features.extend(Self::split_values(&values));
        }
        features.into_iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgScrutinee<'src> {
    Key(&'src str),
    KeyWithItem(&'src str, &'src str),
    Feature(&'src str),
    AllFeatures,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessedCfgScrutinee {
    Empty,
    Some,
    Values(Vec<String>),
}

/// Match on config keys and features
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CfgConditional<'src> {
    /// a list of confg key paths
    pub keys: Vec<CfgScrutinee<'src>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgPatternItem<'src> {
    Empty, // a `_` pattern
    Some,  // the config has "some" value for the key
    None,  // the config has "no" value for the key
    Value(Spanned<&'src str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgPattern<'src> {
    Single(Vec<CfgPatternItem<'src>>, Span),
    Tuple(Vec<Vec<CfgPatternItem<'src>>>, Span),
}

impl<'src> MatchPattern for CfgPattern<'src> {}
impl<'src> MatchPatternParse<'src> for CfgPattern<'src> {
    fn parser() -> impl ZngParser<'src, Self> {
        let or_pat = choice((
            just(Token::Underscore).to(CfgPatternItem::Empty),
            spanned(select! {
                Token::Str(c) => c
            })
            .map(CfgPatternItem::Value),
            just(Token::Ident("Some")).to(CfgPatternItem::Some),
            just(Token::Ident("None")).to(CfgPatternItem::None),
        ))
        .separated_by(just(Token::Pipe))
        .at_least(1)
        .collect::<Vec<_>>();

        choice((
            spanned(
                or_pat
                    .clone()
                    .separated_by(just(Token::Comma))
                    .at_least(1)
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(|items| {
                let span = items.span;
                CfgPattern::Tuple(items.inner, span)
            }),
            spanned(or_pat).map(|pat| {
                let span = pat.span;
                CfgPattern::Single(pat.inner, span)
            }),
        ))
    }
}

impl<'src> Matchable for CfgConditional<'src> {
    type Pattern = CfgPattern<'src>;

    fn eval(&self, pattern: &Self::Pattern, ctx: &mut ParseContext) -> bool {
        let cfg = ctx.get_config_provider();
        let scrutinee = self
            .keys
            .iter()
            .map(|key| match key {
                CfgScrutinee::Key(key) => cfg
                    .get_cfg(key)
                    .map(|values| {
                        if values.is_empty() {
                            ProcessedCfgScrutinee::Some
                        } else {
                            ProcessedCfgScrutinee::Values(values)
                        }
                    })
                    .unwrap_or(ProcessedCfgScrutinee::Empty),
                CfgScrutinee::KeyWithItem(key, item) => cfg
                    .get_cfg(key)
                    .and_then(|values| {
                        values
                            .iter()
                            .any(|value| value == item)
                            .then_some(ProcessedCfgScrutinee::Some)
                    })
                    .unwrap_or(ProcessedCfgScrutinee::Empty),
                CfgScrutinee::AllFeatures => {
                    ProcessedCfgScrutinee::Values(cfg.get_features())
                }
                CfgScrutinee::Feature(feature) => {
                    if cfg.get_features().iter().any(|value| value == feature) {
                        ProcessedCfgScrutinee::Some
                    } else {
                        ProcessedCfgScrutinee::Empty
                    }
                }
            })
            .collect::<Vec<_>>();

        pattern.matches(&scrutinee, ctx)
    }
}

impl<'src> MatchableParse<'src> for CfgConditional<'src> {
    fn parser() -> impl ZngParser<'src, Self> {
        just([Token::Sharp, Token::Ident("cfg")])
            .ignore_then(
                (select! {Token::Ident(c) => c})
                    .separated_by(just(Token::Dot))
                    .at_least(1)
                    .at_most(2)
                    .collect::<Vec<_>>()
                    .map(|item| {
                        match &item[..] {
                            [key] if key == &"feature" => CfgScrutinee::AllFeatures,
                            [key, item] if key == &"feature" => CfgScrutinee::Feature(item),
                            [key] => CfgScrutinee::Key(key),
                            [key, item] => CfgScrutinee::KeyWithItem(key, item),
                            // the above at_least(1) and at_most(2) calls
                            // prevent this branch
                            _ => unreachable!(),
                        }
                    })
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(|item| CfgConditional { keys: item })
    }
}

impl CfgPattern<'_> {
    fn matches(&self, scrutinee: &[ProcessedCfgScrutinee], ctx: &mut ParseContext) -> bool {
        match self {
            Self::Single(pat, span) => {
                if scrutinee.len() == 1 {
                    let scrutinee = scrutinee.first().unwrap();
                    pat.iter().any(|pat| pat.matches(scrutinee))
                } else {
                    ctx.add_error_str("Can not match pattern against multiple cfg values.", *span);
                    false
                }
            }
            Self::Tuple(pats, span) => {
                if pats.len() != scrutinee.len() {
                    ctx.add_error_str(
                        "Number of pattern values and number of config values does not match.",
                        *span,
                    );
                    false
                } else {
                    pats.iter()
                        .zip(scrutinee.iter())
                        .all(|(pat, scrutinee)| pat.iter().any(|pat| pat.matches(scrutinee)))
                }
            }
        }
    }
}

impl CfgPatternItem<'_> {
    fn matches(&self, scrutinee: &ProcessedCfgScrutinee) -> bool {
        use ProcessedCfgScrutinee as PCS;
        match self {
            Self::Empty => true,
            Self::Some => !matches!(scrutinee, PCS::Empty),
            Self::None => matches!(scrutinee, PCS::Empty),
            Self::Value(v) => match &scrutinee {
                PCS::Empty | PCS::Some => false,
                PCS::Values(values) => values.iter().any(|value| value == v.inner),
            },
        }
    }
}
