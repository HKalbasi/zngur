use crate::{ParseContext, ProcessedItemOrAlias, Spanned, Token, ZngParser, spanned};
use chumsky::prelude::*;
use std::marker::PhantomData;

pub trait Matchable<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
    type Pattern: MatchPattern<'src>;
    fn parser() -> impl ZngParser<'src, Self>;
    fn eval<
        'a,
        Item: MatchItem<'a> + 'a,
        Items: IntoIterator<Item = (Self::Pattern, Vec<Spanned<Item>>)>,
    >(
        &self,
        arms: Items,
        ctx: &mut ParseContext,
    ) -> Vec<Spanned<Item::Processed>>;
}

pub trait MatchPattern<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
    fn parser() -> impl ZngParser<'src, Self>;
}

pub trait MatchItem<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
    type Processed;

    fn process(self, ctx: &mut ParseContext) -> Self::Processed;
    fn parser() -> impl ZngParser<'src, Self>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedMatchArmBody<'src, Item: MatchItem<'src>> {
    WithItems {
        items: Vec<Spanned<Item>>,
        _pd: PhantomData<&'src Item>,
    },
    Empty,
}

impl<'src, Item: MatchItem<'src>> ParsedMatchArmBody<'src, Item> {
    pub fn with_items(items: Vec<Spanned<Item>>) -> Self {
        ParsedMatchArmBody::WithItems {
            items,
            _pd: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMatchArm<'src, Pattern: MatchPattern<'src>, Item: MatchItem<'src>> {
    pub pattern: Pattern,
    pub body: ParsedMatchArmBody<'src, Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMatch<'src, Scrutinee: Matchable<'src>, Item: MatchItem<'src>> {
    pub scrutinee: Spanned<Scrutinee>,
    pub arms: Vec<Spanned<ParsedMatchArm<'src, <Scrutinee as Matchable<'src>>::Pattern, Item>>>,
}

impl<'src, Scrutinee: Matchable<'src>, Item: MatchItem<'src>> ParsedMatch<'src, Scrutinee, Item> {
    pub fn eval(self, ctx: &mut ParseContext) -> Vec<Spanned<Item::Processed>> {
        let mut items = Vec::new();
        let arms = self.arms.into_iter().map(|arm| {
            let arm = arm.inner;
            (
                arm.pattern,
                match arm.body {
                    ParsedMatchArmBody::WithItems { items, .. } => items,
                    ParsedMatchArmBody::Empty => Vec::new(),
                },
            )
        });
        let result = self
            .scrutinee
            .inner
            .eval(arms, ctx);
        if !result.is_empty() {
            items.reserve(result.len());
            items.extend(result);
        }
        items
    }
}

pub fn match_item<'src, Scrutinee: Matchable<'src> + 'src, Item: MatchItem<'src> + 'src>()
-> impl ZngParser<'src, ParsedMatch<'src, Scrutinee, Item>> {
    let match_arm = Scrutinee::Pattern::parser()
        .then(
            just(Token::ArrowArm).ignore_then(choice((
                spanned(<Item as MatchItem>::parser())
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::BraceOpen), just(Token::BraceClose))
                    .map(ParsedMatchArmBody::with_items),
                spanned(<Item as MatchItem>::parser())
                    .map(|item| ParsedMatchArmBody::with_items(vec![item])),
                just([Token::BraceOpen, Token::BraceClose]).map(|_t| ParsedMatchArmBody::Empty),
            ))),
        )
        .map(
            |(pattern, body)| ParsedMatchArm::<<Scrutinee as Matchable>::Pattern, Item> {
                pattern,
                body,
            },
        )
        .boxed();

    just([Token::Sharp, Token::KwMatch])
        .ignore_then(spanned(<Scrutinee as Matchable>::parser()))
        .then(
            spanned(match_arm)
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|(scrutinee, arms)| ParsedMatch { scrutinee, arms })
        .boxed()
}

impl<'src> MatchItem<'src> for crate::ParsedTypeItem<'src> {
    type Processed = Self;

    fn process(self, _ctx: &mut ParseContext) -> Self::Processed {
        self
    }

    fn parser() -> impl ZngParser<'src, Self> {
        crate::inner_type_item().boxed()
    }
}

impl<'src> MatchItem<'src> for crate::ParsedItem<'src> {
    type Processed = ProcessedItemOrAlias<'src>;

    fn process(self, ctx: &mut ParseContext) -> Self::Processed {
        crate::process_parsed_item(self, ctx)
    }

    fn parser() -> impl ZngParser<'src, Self> {
        crate::item()
    }
}
