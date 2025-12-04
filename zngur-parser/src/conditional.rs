use crate::{ParseContext, ProcessedItemOrAlias, Spanned, Token, ZngParser, spanned};
use chumsky::prelude::*;
use std::marker::PhantomData;

/// a type that can be matched against a Pattern
pub trait Matchable<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
    /// A pattern type to match Self against
    type Pattern: MatchPattern<'src>;
    /// return a parser for the item as it would appear in an `#if` or `#match` statment
    fn parser() -> impl ZngParser<'src, Self>;
    /// compare self to `Pattern`
    fn eval(&self, pattern: &Self::Pattern, ctx: &mut ParseContext) -> bool;
}

/// a Pattern tha can be matched against
pub trait MatchPattern<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
    /// return a parser for for the pattern
    fn parser() -> impl ZngParser<'src, Self>;
}

/// a trait for an item that can appear inside the block of a conditional statment
pub trait BodyItem<'src>: Sized + core::fmt::Debug + Clone + PartialEq + Eq {
    /// The type Self turns into when added into the Spec
    type Processed;

    /// Transform self into `Processed` type
    fn process(self, ctx: &mut ParseContext) -> Self::Processed;

    /// return a parser for the item
    fn parser() -> impl ZngParser<'src, Self>;
}

/// a type that hold the body of a conditional statment
pub trait ConditionBody<'src, Pattern: MatchPattern<'src> + 'src, Item: BodyItem<'src> + 'src>:
    core::fmt::Debug + Clone + PartialEq + Eq
{
    /// the pattern that guards this body
    fn pattern(&self) -> &Pattern;
    /// Take the items from inside the body consuming self
    fn take_items(self) -> impl IntoIterator<Item = Spanned<Item>>;
}

/// a trait that marks the Cardinality of items inside a body (One? or Many?)
pub trait ConditionBodyCardinality<'src, Item: BodyItem<'src> + 'src>:
    core::fmt::Debug + Clone + PartialEq + Eq
{
    /// the type that hold the body of the conditional statment
    type Body<Pattern: MatchPattern<'src> + 'src>: ConditionBody<'src, Pattern, Item>;
    /// the type that holds the items in the body of a conditional statment
    type Block: core::fmt::Debug + Clone + PartialEq + Eq + IntoIterator<Item = Spanned<Item>>;
    /// the type that hold the items of a passing body
    type EvalResult: IntoIterator<Item = Spanned<<Item as BodyItem<'src>>::Processed>>;
    /// transform a single item into Self::Block
    fn single_to_block(item: Spanned<Item>) -> Self::Block;
    /// create a new empty Self::Bock
    fn empty_block() -> Self::Block;
    /// create a new Self::Body from a block and the pattern that guards it
    fn new_body<Pattern: MatchPattern<'src> + 'src>(
        pattern: Spanned<Pattern>,
        block: Self::Block,
    ) -> Self::Body<Pattern>;
    /// transform a block into it's processed result
    fn pass_block(block: Self::Block, ctx: &mut ParseContext) -> Self::EvalResult;
    /// trasnform a body into it's processed result
    fn pass_body<Pattern: MatchPattern<'src> + 'src>(
        body: Self::Body<Pattern>,
        ctx: &mut ParseContext,
    ) -> Self::EvalResult;
}

/// a trait for a conditional item in a parsed spec
/// evaluated through dynamic dispatch to allow different types to appear in the arms of the same
/// conditional statment `#if type_1 = pat {} #else if type_2 = pat {}`
pub trait ConditionalItem<
    'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item>,
>
{
    /// Evaluate the statment and produce resulting items of the first arm that passes
    fn eval(self: Box<Self>, ctx: &mut ParseContext) -> Option<Cardinality::EvalResult>;
}

/// a body of a conditional statment that holds 0..N Items
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConditionBodyMany<'src, Pattern: MatchPattern<'src>, Item: BodyItem<'src>> {
    pub pattern: Spanned<Pattern>,
    pub block: Vec<Spanned<Item>>,
    _pd: PhantomData<&'src Item>,
}

/// a body of a conditional statment that holds 0..1 items
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConditionBodySingle<'src, Pattern: MatchPattern<'src>, Item: BodyItem<'src>> {
    pub pattern: Spanned<Pattern>,
    pub block: Option<Spanned<Item>>,
    _pd: PhantomData<&'src Item>,
}

impl<'src, Pattern: MatchPattern<'src> + 'src, Item: BodyItem<'src> + 'src>
    ConditionBody<'src, Pattern, Item> for ParsedConditionBodyMany<'src, Pattern, Item>
{
    fn pattern(&self) -> &Pattern {
        &self.pattern.inner
    }
    fn take_items(self) -> impl IntoIterator<Item = Spanned<Item>> {
        self.block
    }
}

impl<'src, Pattern: MatchPattern<'src> + 'src, Item: BodyItem<'src> + 'src>
    ConditionBody<'src, Pattern, Item> for ParsedConditionBodySingle<'src, Pattern, Item>
{
    fn pattern(&self) -> &Pattern {
        &self.pattern.inner
    }
    fn take_items(self) -> impl IntoIterator<Item = Spanned<Item>> {
        self.block
    }
}

/// Marker type for a conditional statment that contexualy only accepts one item
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SingleItemBody;

impl<'src, Item: BodyItem<'src> + 'src> ConditionBodyCardinality<'src, Item> for SingleItemBody {
    type Body<Pattern: MatchPattern<'src> + 'src> = ParsedConditionBodySingle<'src, Pattern, Item>;
    type Block = Option<Spanned<Item>>;
    type EvalResult = Option<Spanned<<Item as BodyItem<'src>>::Processed>>;

    fn single_to_block(item: Spanned<Item>) -> Self::Block {
        Some(item)
    }

    fn empty_block() -> Self::Block {
        None
    }

    fn new_body<Pattern: MatchPattern<'src> + 'src>(
        pattern: Spanned<Pattern>,
        block: Self::Block,
    ) -> Self::Body<Pattern> {
        ParsedConditionBodySingle {
            pattern,
            block,
            _pd: PhantomData,
        }
    }

    fn pass_block(block: Self::Block, ctx: &mut ParseContext) -> Self::EvalResult {
        block.map(|item| {
            let span = item.span;
            Spanned {
                span,
                inner: item.inner.process(ctx),
            }
        })
    }

    fn pass_body<Pattern: MatchPattern<'src> + 'src>(
        body: Self::Body<Pattern>,
        ctx: &mut ParseContext,
    ) -> Self::EvalResult {
        Self::pass_block(body.block, ctx)
    }
}

/// Marker type for a conditional statment that contexualy accepts any number of items
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ManyItemBody;

impl<'src, Item: BodyItem<'src> + 'src> ConditionBodyCardinality<'src, Item> for ManyItemBody {
    type Body<Pattern: MatchPattern<'src> + 'src> = ParsedConditionBodyMany<'src, Pattern, Item>;
    type Block = Vec<Spanned<Item>>;
    type EvalResult = Vec<Spanned<<Item as BodyItem<'src>>::Processed>>;

    fn single_to_block(item: Spanned<Item>) -> Self::Block {
        vec![item]
    }

    fn empty_block() -> Self::Block {
        Vec::new()
    }

    fn new_body<Pattern: MatchPattern<'src> + 'src>(
        pattern: Spanned<Pattern>,
        block: Self::Block,
    ) -> Self::Body<Pattern> {
        ParsedConditionBodyMany {
            pattern,
            block,
            _pd: PhantomData,
        }
    }

    fn pass_block(block: Self::Block, ctx: &mut ParseContext) -> Self::EvalResult {
        block
            .into_iter()
            .map(|item| {
                let span = item.span;
                Spanned {
                    span,
                    inner: item.inner.process(ctx),
                }
            })
            .collect()
    }

    fn pass_body<Pattern: MatchPattern<'src> + 'src>(
        body: Self::Body<Pattern>,
        ctx: &mut ParseContext,
    ) -> Self::EvalResult {
        Self::pass_block(body.block, ctx)
    }
}

/// a branch of an `#if` statment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConditionBranch<
    'src,
    Scrutinee: Matchable<'src> + 'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item>,
> {
    pub scrutinee: Spanned<Scrutinee>,
    pub body: <Cardinality as ConditionBodyCardinality<'src, Item>>::Body<
        <Scrutinee as Matchable<'src>>::Pattern,
    >,
}

/// a complete `#if {} #else if #else {}` statment
pub struct ParsedCondition<
    'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item>,
> {
    pub arms: Vec<Box<dyn ConditionalItem<'src, Item, Cardinality> + 'src>>,
    pub fallback: Option<Cardinality::Block>,
}

/// a `#match` statment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConditionMatch<
    'src,
    Scrutinee: Matchable<'src> + 'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item>,
> {
    pub scrutinee: Spanned<Scrutinee>,
    pub arms: Vec<
        Spanned<
            <Cardinality as ConditionBodyCardinality<'src, Item>>::Body<
                <Scrutinee as Matchable<'src>>::Pattern,
            >,
        >,
    >,
}

impl<
    'src,
    Scrutinee: Matchable<'src> + 'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item>,
> ConditionalItem<'src, Item, Cardinality>
    for ParsedConditionBranch<'src, Scrutinee, Item, Cardinality>
{
    fn eval(
        self: Box<Self>,
        ctx: &mut ParseContext,
    ) -> Option<<Cardinality as ConditionBodyCardinality<'src, Item>>::EvalResult> {
        let pattern = self.body.pattern();
        if self.scrutinee.inner.eval(pattern, ctx) {
            return Some(
                <Cardinality as ConditionBodyCardinality<'src, Item>>::pass_body(self.body, ctx),
            );
        }
        None
    }
}

impl<
    'src,
    Scrutinee: Matchable<'src> + 'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item>,
> ConditionalItem<'src, Item, Cardinality>
    for ParsedConditionMatch<'src, Scrutinee, Item, Cardinality>
{
    fn eval(
        self: Box<Self>,
        ctx: &mut ParseContext,
    ) -> Option<<Cardinality as ConditionBodyCardinality<'src, Item>>::EvalResult> {
        for arm in self.arms {
            let pattern = arm.inner.pattern();
            if self.scrutinee.inner.eval(pattern, ctx) {
                return Some(
                    <Cardinality as ConditionBodyCardinality<'src, Item>>::pass_body(
                        arm.inner, ctx,
                    ),
                );
            }
        }
        None
    }
}

impl<'src, Item: BodyItem<'src> + 'src, Cardinality: ConditionBodyCardinality<'src, Item>>
    ConditionalItem<'src, Item, Cardinality> for ParsedCondition<'src, Item, Cardinality>
{
    fn eval(
        self: Box<Self>,
        ctx: &mut ParseContext,
    ) -> Option<<Cardinality as ConditionBodyCardinality<'src, Item>>::EvalResult> {
        for arm in self.arms {
            if let Some(result) = arm.eval(ctx) {
                return Some(result);
            }
        }
        if let Some(fallback) = self.fallback {
            return Some(
                <Cardinality as ConditionBodyCardinality<'src, Item>>::pass_block(fallback, ctx),
            );
        }
        None
    }
}

/// a trait that helps build combined parsers for ConditionalItem's that accept `#if {} #else {}` or `#match`
pub trait Condition<
    'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item>,
>
{
    fn if_parser() -> impl ZngParser<'src, Box<dyn ConditionalItem<'src, Item, Cardinality> + 'src>>;
    fn match_parser()
    -> impl ZngParser<'src, Box<dyn ConditionalItem<'src, Item, Cardinality> + 'src>>;
}

/// a psraser for aa block used in a SingleItemBody
pub fn block_for_single<'src, Item: BodyItem<'src>>() -> impl ZngParser<'src, Option<Spanned<Item>>>
{
    spanned(<Item as BodyItem>::parser())
        .repeated()
        .at_most(1)
        .collect::<Vec<_>>()
        .map(|items| {
            // should be 1 or zero becasue of `.at_most(1)`
            items.into_iter().next()
        })
}

/// a parser for a block used in a ManyItemBody
pub fn block_for_many<'src, Item: BodyItem<'src>>() -> impl ZngParser<'src, Vec<Spanned<Item>>> {
    spanned(<Item as BodyItem>::parser())
        .repeated()
        .collect::<Vec<_>>()
}

/// parser for a guarded block used in `#if`
pub fn guarded_block<
    'src,
    Scrutinee: Matchable<'src> + 'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item>,
>(
    block: impl ZngParser<'src, Cardinality::Block>,
    f: impl Fn(
        Spanned<<Scrutinee as Matchable<'src>>::Pattern>,
        <Cardinality as ConditionBodyCardinality<'src, Item>>::Block,
    ) -> <Cardinality as ConditionBodyCardinality<'src, Item>>::Body<
        <Scrutinee as Matchable<'src>>::Pattern,
    > + Clone,
) -> impl ZngParser<'src, ParsedConditionBranch<'src, Scrutinee, Item, Cardinality>> {
    spanned(<Scrutinee as Matchable>::parser())
        .then_ignore(just(Token::Eq))
        .then(spanned(
            <<Scrutinee as Matchable>::Pattern as MatchPattern>::parser(),
        ))
        .then(block.delimited_by(just(Token::BraceOpen), just(Token::BraceClose)))
        .map(move |((scrutinee, pattern), block)| ParsedConditionBranch {
            scrutinee,
            body: f(pattern, block),
        })
}

/// parser for an `#if {} #else if {} #else {}` statment
pub fn if_stmnt<
    'src,
    Scrutinee: Matchable<'src> + 'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item> + 'src,
>(
    guard: impl ZngParser<'src, ParsedConditionBranch<'src, Scrutinee, Item, Cardinality>> + 'src,
    fallback: impl ZngParser<'src, Cardinality::Block> + 'src,
) -> impl ZngParser<'src, ParsedCondition<'src, Item, Cardinality>> {
    just([Token::Sharp, Token::KwIf])
        .ignore_then(guard.clone())
        .then(
            just([Token::Sharp, Token::KwElse, Token::KwIf])
                .ignore_then(guard)
                .repeated()
                .collect::<Vec<_>>(),
        )
        .then(
            just([Token::Sharp, Token::KwElse])
                .ignore_then(fallback)
                .or_not(),
        )
        .map(|((if_block, else_if_blocks), else_block)| {
            let mut arms: Vec<Box<dyn ConditionalItem<Item, Cardinality> + 'src>> =
                vec![Box::new(if_block)];
            arms.extend(else_if_blocks.into_iter().map(|arm| {
                let boxed: Box<dyn ConditionalItem<Item, Cardinality> + 'src> = Box::new(arm);
                boxed
            }));
            ParsedCondition {
                arms,
                fallback: else_block,
            }
        })
        .boxed()
}

/// a paraser for the arm of a `#match` statment
fn match_arm<
    'src,
    Scrutinee: Matchable<'src> + 'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item> + 'src,
>(
    block: impl ZngParser<'src, Cardinality::Block> + 'src,
) -> impl ZngParser<
    'src,
    <Cardinality as ConditionBodyCardinality<'src, Item>>::Body<
        <Scrutinee as Matchable<'src>>::Pattern,
    >,
> {
    let arm_choices = (
        block.delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        spanned(<Item as BodyItem>::parser())
            .map(<Cardinality as ConditionBodyCardinality<'src, Item>>::single_to_block),
        just([Token::BraceOpen, Token::BraceClose])
            .map(|_| <Cardinality as ConditionBodyCardinality<'src, Item>>::empty_block()),
    );
    spanned(<<Scrutinee as Matchable>::Pattern as MatchPattern>::parser())
        .then(just(Token::ArrowArm).ignore_then(choice(arm_choices)))
        .map(|(pattern, block)| {
            <Cardinality as ConditionBodyCardinality<'src, Item>>::new_body(pattern, block)
        })
}

/// a parser for a `#match` statment
fn match_stmt<
    'src,
    Scrutinee: Matchable<'src> + 'src,
    Item: BodyItem<'src> + 'src,
    Cardinality: ConditionBodyCardinality<'src, Item> + 'src,
>(
    block: impl ZngParser<'src, Cardinality::Block> + 'src,
) -> impl ZngParser<'src, ParsedConditionMatch<'src, Scrutinee, Item, Cardinality>> {
    let arm = match_arm::<Scrutinee, Item, Cardinality>(block);

    just([Token::Sharp, Token::KwMatch])
        .ignore_then(spanned(<Scrutinee as Matchable>::parser()))
        .then(
            spanned(arm)
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
        )
        .map(|(scrutinee, arms)| ParsedConditionMatch { scrutinee, arms })
        .boxed()
}

impl<'src, T: Matchable<'src> + 'src, Item: BodyItem<'src> + 'src>
    Condition<'src, Item, SingleItemBody> for T
{
    fn if_parser()
    -> impl ZngParser<'src, Box<dyn ConditionalItem<'src, Item, SingleItemBody> + 'src>> {
        let block = block_for_single::<Item>();
        let guard = guarded_block::<T, Item, SingleItemBody>(block.clone(), |pattern, item| {
            ParsedConditionBodySingle {
                pattern,
                block: item,
                _pd: PhantomData,
            }
        });

        if_stmnt::<T, Item, SingleItemBody>(guard, block).map(|item| {
            let boxed: Box<dyn ConditionalItem<'src, Item, SingleItemBody> + 'src> = Box::new(item);
            boxed
        })
    }

    fn match_parser()
    -> impl ZngParser<'src, Box<dyn ConditionalItem<'src, Item, SingleItemBody> + 'src>> {
        let block = block_for_single::<Item>();
        match_stmt::<T, Item, SingleItemBody>(block).map(|item| {
            let boxed: Box<dyn ConditionalItem<'src, Item, SingleItemBody> + 'src> = Box::new(item);
            boxed
        })
    }
}

impl<'src, T: Matchable<'src> + 'src, Item: BodyItem<'src> + 'src>
    Condition<'src, Item, ManyItemBody> for T
{
    fn if_parser() -> impl ZngParser<'src, Box<dyn ConditionalItem<'src, Item, ManyItemBody> + 'src>>
    {
        let block = block_for_many::<Item>();
        let guard = guarded_block::<T, Item, ManyItemBody>(block.clone(), |pattern, block| {
            ParsedConditionBodyMany {
                pattern,
                block,
                _pd: PhantomData,
            }
        });

        if_stmnt::<T, Item, ManyItemBody>(guard, block).map(|item| {
            let boxed: Box<dyn ConditionalItem<'src, Item, ManyItemBody> + 'src> = Box::new(item);
            boxed
        })
    }

    fn match_parser()
    -> impl ZngParser<'src, Box<dyn ConditionalItem<'src, Item, ManyItemBody> + 'src>> {
        let block = block_for_many::<Item>();
        match_stmt::<T, Item, ManyItemBody>(block).map(|item| {
            let boxed: Box<dyn ConditionalItem<'src, Item, ManyItemBody> + 'src> = Box::new(item);
            boxed
        })
    }
}

impl<'src> BodyItem<'src> for crate::ParsedTypeItem<'src> {
    type Processed = Self;

    fn process(self, _ctx: &mut ParseContext) -> Self::Processed {
        self
    }

    fn parser() -> impl ZngParser<'src, Self> {
        crate::inner_type_item().boxed()
    }
}

impl<'src> BodyItem<'src> for crate::ParsedItem<'src> {
    type Processed = ProcessedItemOrAlias<'src>;

    fn process(self, ctx: &mut ParseContext) -> Self::Processed {
        crate::process_parsed_item(self, ctx)
    }

    fn parser() -> impl ZngParser<'src, Self> {
        crate::item()
    }
}
