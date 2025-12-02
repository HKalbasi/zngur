use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    ParseContext, Span, Spanned, Token, ZngParser,
    match_stmnt::{MatchPattern, Matchable},
    spanned,
};
use chumsky::prelude::*;

pub trait RustCfgProvider {
    /// Gets values assoceated with a config key if it's present.
    fn get_cfg(&self, key: &str) -> Option<Vec<&str>>;
    /// Gets a list of feature names that are enabeled
    fn get_features(&self) -> Vec<&str>;
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
    fn get_cfg(&self, key: &str) -> Option<Vec<&str>> {
        self.cfg
            .get(key)
            .map(|values| values.iter().map(std::convert::AsRef::as_ref).collect())
    }
    fn get_features(&self) -> Vec<&str> {
        self.features
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedCfgScrutinee<'src> {
    Key(&'src str),
    KeyWithItem(&'src str, &'src str),
    Feature(&'src str),
    AllFeatures,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessedCfgScrutinee<'src> {
    Empty,
    Some,
    Values(Vec<&'src str>),
}

/// Match on config keys and features
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMatchCfg<'src> {
    /// a list of confg key paths
    pub keys: Vec<ParsedCfgScrutinee<'src>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedMatchCfgPatternItem<'src> {
    Empty, // a `_` pattern
    Some,  // the config has "some" value for the key
    None,  // the config has "no" value for the key
    Value(Spanned<&'src str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedMatchCfgPattern<'src> {
    Single(Vec<ParsedMatchCfgPatternItem<'src>>, Span),
    Tuple(Vec<Vec<ParsedMatchCfgPatternItem<'src>>>, Span),
}

impl<'src> MatchPattern<'src> for ParsedMatchCfgPattern<'src> {
    fn parser() -> impl ZngParser<'src, Self> {
        let or_pat = choice((
            just(Token::Underscore).to(ParsedMatchCfgPatternItem::Empty),
            spanned(select! {
                Token::Str(c) => c
            })
            .map(ParsedMatchCfgPatternItem::Value),
            just(Token::Ident("Some")).to(ParsedMatchCfgPatternItem::Some),
            just(Token::Ident("None")).to(ParsedMatchCfgPatternItem::None),
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
                ParsedMatchCfgPattern::Tuple(items.inner, span)
            }),
            spanned(or_pat).map(|pat| {
                let span = pat.span;
                ParsedMatchCfgPattern::Single(pat.inner, span)
            }),
        ))
    }
}

impl<'src> Matchable<'src> for ParsedMatchCfg<'src> {
    type Pattern = ParsedMatchCfgPattern<'src>;
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
                            [key] if key == &"feature" => ParsedCfgScrutinee::AllFeatures,
                            [key, item] if key == &"feature" => ParsedCfgScrutinee::Feature(item),
                            [key] => ParsedCfgScrutinee::Key(key),
                            [key, item] => ParsedCfgScrutinee::KeyWithItem(key, item),
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
            .map(|item| ParsedMatchCfg { keys: item })
    }

    fn eval<
        'a,
        Item: crate::match_stmnt::MatchItem<'a> + 'a,
        Items: IntoIterator<Item = (Self::Pattern, Vec<Item>)>,
    >(
        &self,
        arms: Items,
        ctx: &mut ParseContext,
    ) -> Vec<Item::Processed> {
        let mut items = Vec::new();
        let cfg = ctx.get_config_provider();
        let scrutinee = self
            .keys
            .iter()
            .map(|key| match key {
                ParsedCfgScrutinee::Key(key) => cfg
                    .get_cfg(key)
                    .map(ProcessedCfgScrutinee::Values)
                    .unwrap_or(ProcessedCfgScrutinee::Empty),
                ParsedCfgScrutinee::KeyWithItem(key, item) => cfg
                    .get_cfg(key)
                    .and_then(|values| values.contains(item).then_some(ProcessedCfgScrutinee::Some))
                    .unwrap_or(ProcessedCfgScrutinee::Empty),
                ParsedCfgScrutinee::AllFeatures => {
                    ProcessedCfgScrutinee::Values(cfg.get_features())
                }
                ParsedCfgScrutinee::Feature(feature) => cfg
                    .get_features()
                    .contains(feature)
                    .then_some(ProcessedCfgScrutinee::Some)
                    .unwrap_or(ProcessedCfgScrutinee::Empty),
            })
            .collect::<Vec<_>>();
        for (pattern, body) in arms {
            if pattern.matches(&scrutinee, ctx) {
                items.extend(body.into_iter().map(|item| item.process()))
            }
        }
        items
    }
}

impl ParsedMatchCfgPattern<'_> {
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

impl ParsedMatchCfgPatternItem<'_> {
    fn matches(&self, scrutinee: &ProcessedCfgScrutinee) -> bool {
        use ProcessedCfgScrutinee as PCS;
        match self {
            Self::Empty => true,
            Self::Some => match &scrutinee {
                PCS::Empty => false,
                _ => true,
            },
            Self::None => match &scrutinee {
                PCS::Empty => true,
                _ => false,
            },
            Self::Value(v) => match &scrutinee {
                PCS::Empty | PCS::Some => false,
                PCS::Values(values) => values.contains(&v.inner),
            },
        }
    }
}
