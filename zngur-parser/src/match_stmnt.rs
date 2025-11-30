pub(crate) mod parse {

    use crate::{Spanned, ZngParser, spanned, Token};
    use chumsky::prelude::*;
    use std::marker::PhantomData;

    pub trait MatchableParser<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
        type Pattern: MatchPatternParser<'src>;
        fn parser() -> impl ZngParser<'src, Self>;
    }

    pub trait MatchPatternParser<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
        fn parser() -> impl ZngParser<'src, Self>;
    }

    pub trait MatchItemParser<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
        fn parser() -> impl ZngParser<'src, Self>;
    }

    /// Match on config keys and features
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParsedMatchCfg<'src> {
        /// a list of confg key paths
        pub item: Vec<Vec<&'src str>>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ParsedMatchCfgPatternItem<'src> {
        Empty, // a `_` pattern
        Some,  // the config has "some" value for the key
        None,  // the config has "no" value for the key
        Value(Spanned<&'src str>),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParsedMatchCfgPattern<'src>(pub Vec<ParsedMatchCfgPatternItem<'src>>);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ParsedMatchEnumBody<'src, Item: MatchItemParser<'src>> {
        Items(Vec<Spanned<Item>>, PhantomData<&'src Item>),
        Empty,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParsedMatchArm<'src, Pattern: MatchPatternParser<'src>, Item: MatchItemParser<'src>> {
        pub pattern: Pattern,
        pub body: ParsedMatchEnumBody<'src, Item>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParsedMatch<'src, Scrutinee: MatchableParser<'src>, Item: MatchItemParser<'src>> {
        pub scrutinee: Spanned<Scrutinee>,
        pub arms:
            Vec<Spanned<ParsedMatchArm<'src, <Scrutinee as MatchableParser<'src>>::Pattern, Item>>>,
    }

    impl<'src> MatchPatternParser<'src> for ParsedMatchCfgPattern<'src> {
        fn parser() -> impl ZngParser<'src, Self> {
            choice((
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
            .collect::<Vec<_>>()
            .map(ParsedMatchCfgPattern)
        }
    }

    impl<'src> MatchableParser<'src> for ParsedMatchCfg<'src> {
        type Pattern = ParsedMatchCfgPattern<'src>;
        fn parser() -> impl ZngParser<'src, Self> {
            just([Token::Sharp, Token::Ident("cfg")])
                .ignore_then(
                    (select! {Token::Ident(c) => c})
                        .separated_by(just(Token::Dot))
                        .at_least(1)
                        .collect::<Vec<_>>()
                        .separated_by(just(Token::Comma))
                        .at_least(1)
                        .collect::<Vec<_>>()
                        .delimited_by(just(Token::ParenOpen), just(Token::ParenClose)),
                )
                .map(|item| ParsedMatchCfg { item })
        }
    }

    pub fn match_item<
        'src,
        Scrutinee: MatchableParser<'src> + 'src,
        Item: MatchItemParser<'src> + 'src,
    >() -> impl ZngParser<'src, ParsedMatch<'src, Scrutinee, Item>> {
        let match_arm = Scrutinee::Pattern::parser()
            .then(
                just(Token::ArrowArm).ignore_then(choice((
                    spanned(<Item as MatchItemParser>::parser())
                        .repeated()
                        .collect::<Vec<_>>()
                        .delimited_by(just(Token::BraceOpen), just(Token::BraceClose))
                        .map(|items| ParsedMatchEnumBody::Items(items, PhantomData)),
                    spanned(<Item as MatchItemParser>::parser())
                        .map(|item| ParsedMatchEnumBody::Items(vec![item], PhantomData)),
                    just([Token::BraceOpen, Token::BraceClose])
                        .map(|_t| ParsedMatchEnumBody::Empty),
                ))),
            )
            .map(
                |(pattern, body)| ParsedMatchArm::<<Scrutinee as MatchableParser>::Pattern, Item> {
                    pattern,
                    body,
                },
            )
            .boxed();

        just([Token::Sharp, Token::KwMatch])
            .ignore_then(spanned(<Scrutinee as MatchableParser>::parser()))
            .then(
                spanned(match_arm)
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
            )
            .map(|(scrutinee, arms)| ParsedMatch { scrutinee, arms })
            .boxed()
    }

}
