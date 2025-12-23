use std::collections::{HashMap, HashSet};

use crate::{
    ParseContext, Span, Token, ZngParser,
    conditional::{MatchPattern, MatchPatternParse, Matchable, MatchableParse},
    spanned,
};
use chumsky::prelude::*;


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessedCfgConditional {
    Single(ProcessedCfgScrutinee),
    Tuple(Vec<ProcessedCfgScrutinee>),
}

/// Match on config keys and features
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgConditional<'src> {
    Single(CfgScrutinee<'src>),
    Tuple(Vec<CfgScrutinee<'src>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgPatternItem<'src> {
    Empty, // a `_` pattern
    Some,  // the config has "some" value for the key
    None,  // the config has "no" value for the key
    Str(&'src str),
    Number(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgPattern<'src> {
    Single(CfgPatternItem<'src>, Span),
    And(Vec<CfgPattern<'src>>, Span),
    Or(Vec<CfgPattern<'src>>, Span),
    Not(Box<CfgPattern<'src>>, Span),
    Grouped(Box<CfgPattern<'src>>, Span),
    Tuple(Vec<CfgPattern<'src>>, Span),
}

impl<'src> MatchPattern for CfgPattern<'src> {
    fn default_some(span: Span) -> Self {
        CfgPattern::Single(CfgPatternItem::Some, span)
    }
}
impl<'src> MatchPatternParse<'src> for CfgPattern<'src> {
    fn parser() -> impl ZngParser<'src, Self> {
        let single = recursive(|pat| {
            let literals = select! {
                Token::Str(c) => CfgPatternItem::Str(c),
                Token::Number(n) => CfgPatternItem::Number(n),
            };
            let atom = choice((
                spanned(literals),
                spanned(just(Token::Underscore).to(CfgPatternItem::Empty)),
                spanned(just(Token::Ident("Some")).to(CfgPatternItem::Some)),
                spanned(just(Token::Ident("None")).to(CfgPatternItem::None)),
            ))
            .map(|item| CfgPattern::Single(item.inner, item.span))
            .or(spanned(
                pat.delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map(|item| CfgPattern::Grouped(Box::new(item.inner), item.span)));

            let not_pat = just(Token::Bang)
                .repeated()
                .foldr_with(atom, |_op, rhs, e| CfgPattern::Not(Box::new(rhs), e.span()));

            let and_pat = not_pat.clone().foldl_with(
                just(Token::And).ignore_then(not_pat).repeated(),
                |lhs, rhs, e| match lhs {
                    CfgPattern::And(mut items, _span) => {
                        items.push(rhs);
                        CfgPattern::And(items, e.span())
                    }
                    _ => CfgPattern::And(vec![lhs, rhs], e.span()),
                },
            );

            // or pat
            and_pat.clone().foldl_with(
                just(Token::Pipe).ignore_then(and_pat).repeated(),
                |lhs, rhs, e| match lhs {
                    CfgPattern::Or(mut items, _span) => {
                        items.push(rhs);
                        CfgPattern::Or(items, e.span())
                    }
                    _ => CfgPattern::Or(vec![lhs, rhs], e.span()),
                },
            )
        });

        spanned(
            single
                .clone()
                .separated_by(just(Token::Comma))
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
        )
        .map(|item| CfgPattern::Tuple(item.inner, item.span))
        .or(single)
    }
}

impl<'src> Matchable for CfgConditional<'src> {
    type Pattern = CfgPattern<'src>;

    fn eval(&self, pattern: &Self::Pattern, ctx: &mut ParseContext) -> bool {
        let cfg = ctx.get_config_provider();

        let process = |key: &CfgScrutinee<'src>| -> ProcessedCfgScrutinee {
            match key {
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
                CfgScrutinee::AllFeatures => ProcessedCfgScrutinee::Values(cfg.get_features()),
                CfgScrutinee::Feature(feature) => {
                    if cfg.get_features().iter().any(|value| value == feature) {
                        ProcessedCfgScrutinee::Some
                    } else {
                        ProcessedCfgScrutinee::Empty
                    }
                }
            }
        };

        let scrutinee = match self {
            Self::Single(key) => ProcessedCfgConditional::Single(process(key)),
            Self::Tuple(keys) => {
                ProcessedCfgConditional::Tuple(keys.iter().map(process).collect::<Vec<_>>())
            }
        };

        pattern.matches(&scrutinee, ctx)
    }
}

impl<'src> CfgScrutinee<'src> {
    fn parser() -> impl ZngParser<'src, Self> {
        select! {Token::Ident(c) => c, Token::Str(s) => s}
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
    }
}

impl<'src> MatchableParse<'src> for CfgConditional<'src> {
    fn parser() -> impl ZngParser<'src, Self> {
        let directive = just([Token::Ident("cfg"), Token::Bang]).ignore_then(
            CfgScrutinee::parser().delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
        );

        choice((
            directive.clone().map(CfgConditional::Single),
            directive
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .at_least(1)
                .collect::<Vec<_>>()
                .map(CfgConditional::Tuple)
                .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
        ))
    }

    fn combined() -> Option<
        crate::BoxedZngParser<
            'src,
            (
                crate::Spanned<Self>,
                crate::Spanned<<Self as Matchable>::Pattern>,
            ),
        >,
    > {
        let directive = just([Token::Ident("cfg"), Token::Bang])
            .ignore_then(
                spanned(CfgScrutinee::parser())
                    .then(
                        just(Token::Eq)
                            .ignore_then(spanned(CfgPattern::parser()))
                            .or_not(),
                    )
                    .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
            )
            .map_with(|(scrutinee, pat), e| {
                (
                    scrutinee,
                    pat.unwrap_or_else(|| Spanned {
                        inner: CfgPattern::default_some(e.span()),
                        span: e.span(),
                    }),
                )
            });
        Some(
            directive
                .clone()
                .map(|(scrutinee, pat)| {
                    (
                        Spanned {
                            inner: CfgConditional::Single(scrutinee.inner),
                            span: scrutinee.span,
                        },
                        pat,
                    )
                })
                .boxed(),
        )
    }
}

impl CfgPattern<'_> {
    fn matches(&self, scrutinee: &ProcessedCfgConditional, ctx: &mut ParseContext) -> bool {
        use ProcessedCfgConditional as PCC;
        match (self, scrutinee) {
            (Self::Tuple(pats, _), PCC::Single(_)) if pats.len() == 1 => {
                let pat = pats.iter().last().unwrap();
                // tuple is actually single
                pat.matches(scrutinee, ctx)
            }
            (Self::Single(pat, _), PCC::Tuple(scrutinees)) if scrutinees.len() == 1 => {
                let scrutinee = scrutinees.iter().last().unwrap();
                // tuple is actually single
                pat.matches(scrutinee)
            }
            (Self::Single(CfgPatternItem::Empty, _), PCC::Tuple(_)) => {
                // empty pattern matches anything
                true
            }
            (Self::Tuple(_, span), PCC::Single(_)) => {
                ctx.add_error_str(
                    "Can not match tuple pattern against a single cfg value.",
                    *span,
                );
                false
            }
            (
                Self::Single(_, span)
                | Self::Not(_, span)
                | Self::And(_, span)
                | Self::Or(_, span)
                | Self::Grouped(_, span),
                PCC::Tuple(_),
            ) => {
                ctx.add_error_str(
                    "Can not match single pattern against multiple cfg values.",
                    *span,
                );
                false
            }
            (Self::Tuple(pats, span), PCC::Tuple(scrutinees)) => {
                if scrutinees.len() != pats.len() {
                    ctx.add_error_str(
                        "Number of patterns and number of scrutinees do not match.",
                        *span,
                    );
                    false
                } else {
                    pats.iter()
                        .zip(scrutinees.iter())
                        .all(|(pat, scrutinee)| pat.matches(&PCC::Single(scrutinee.clone()), ctx))
                }
            }
            (Self::Single(pat, _), PCC::Single(scrutinee)) => pat.matches(scrutinee),
            (Self::Grouped(pat, _), PCC::Single(_)) => pat.matches(scrutinee, ctx),
            (Self::Not(pat, _), PCC::Single(_)) => !pat.matches(scrutinee, ctx),
            (Self::And(pats, _), PCC::Single(_)) => {
                pats.iter().all(|pat| pat.matches(scrutinee, ctx))
            }
            (Self::Or(pats, _), PCC::Single(_)) => {
                pats.iter().any(|pat| pat.matches(scrutinee, ctx))
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
            Self::Str(v) => match &scrutinee {
                PCS::Empty | PCS::Some => false,
                PCS::Values(values) => values.iter().any(|value| value == v),
            },
            Self::Number(n) => match &scrutinee {
                PCS::Empty | PCS::Some => false,
                PCS::Values(values) => values
                    .iter()
                    .any(|value| value.parse::<usize>().map(|v| v == *n).unwrap_or(false)),
            },
        }
    }
}
