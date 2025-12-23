use crate::{BoxedZngParser, ParseContext, Spanned, Token, ZngParser, spanned};

/// a type that can be matched against a Pattern
pub trait Matchable: core::fmt::Debug + Clone + PartialEq + Eq {
    /// A pattern type to match Self against
    type Pattern: MatchPattern;
    /// compare self to `Pattern`
    fn eval(&self, pattern: &Self::Pattern, ctx: &mut ParseContext) -> bool;
}

/// a type that can be matched against a Pattern
pub trait MatchableParse<'src>: Matchable {
    /// return a parser for the item as it would appear in an `#if` or `#match` statement
    fn parser() -> impl ZngParser<'src, Self>;

    /// return an optional combined parser for use in `#if` statements
    #[allow(clippy::complexity)]
    fn combined()
    -> Option<BoxedZngParser<'src, (Spanned<Self>, Spanned<<Self as Matchable>::Pattern>)>> {
        None
    }
}

pub trait MatchPattern: core::fmt::Debug + Clone + PartialEq + Eq {
    /// a pattern that matches the existence of "some" value
    fn default_some(span: crate::Span) -> Self;
}

/// a Pattern tha can be matched against
pub trait MatchPatternParse<'src>: MatchPattern {
    /// return a parser for for the pattern
    fn parser() -> impl ZngParser<'src, Self>;
}

pub trait BodyItem: core::fmt::Debug + Clone + PartialEq + Eq {
    /// The type Self turns into when added into the Spec
    type Processed;

    /// Transform self into `Processed` type
    fn process(self, ctx: &mut ParseContext) -> Self::Processed;
}

/// a type that hold the body of a conditional statement
pub trait ConditionBody<Pattern: MatchPattern, Item: BodyItem>: core::fmt::Debug {
    /// the pattern that guards this body
    fn pattern(&self) -> &Pattern;
}

/// a trait that marks the Cardinality of items inside a body (One? or Many?)
pub trait ConditionBodyCardinality<Item: BodyItem>:
    core::fmt::Debug + Clone + PartialEq + Eq
{
    /// the type that hold the body of the conditional statement
    type Body<Pattern: MatchPattern>: ConditionBody<Pattern, Item> + Clone + PartialEq + Eq;
    /// the type that holds the items in the body of a conditional statement
    type Block: core::fmt::Debug + Clone + PartialEq + Eq + IntoIterator<Item = Spanned<Item>>;
    /// the type that hold the items of a passing body
    type EvalResult: IntoIterator<Item = Spanned<<Item as BodyItem>::Processed>>;
    /// transform a single item into Self::Block
    fn single_to_block(item: Spanned<Item>) -> Self::Block;
    /// create a new empty Self::Bock
    fn empty_block() -> Self::Block;
    /// create a new Self::Body from a block and the pattern that guards it
    fn new_body<Pattern: MatchPattern>(
        pattern: Spanned<Pattern>,
        block: Self::Block,
    ) -> Self::Body<Pattern>;
    /// transform a block into it's processed result
    fn pass_block(block: &Self::Block, ctx: &mut ParseContext) -> Self::EvalResult;
    /// transform a body into it's processed result
    fn pass_body<Pattern: MatchPattern>(
        body: &Self::Body<Pattern>,
        ctx: &mut ParseContext,
    ) -> Self::EvalResult;
}

/// a trait for a conditional item in a parsed spec
pub trait ConditionalItem<Item: BodyItem, Cardinality: ConditionBodyCardinality<Item>> {
    /// Evaluate the statement and produce resulting items of the first arm that passes
    fn eval(&self, ctx: &mut ParseContext) -> Option<Cardinality::EvalResult>;
}

/// a body of a conditional statement that holds 0..N Items
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionBodyMany<Pattern: MatchPattern, Item: BodyItem> {
    pub pattern: Spanned<Pattern>,
    pub block: Vec<Spanned<Item>>,
}

/// a body of a conditional statement that holds 0..1 items
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionBodySingle<Pattern: MatchPattern, Item: BodyItem> {
    pub pattern: Spanned<Pattern>,
    pub block: Option<Spanned<Item>>,
}

impl<Pattern: MatchPattern, Item: BodyItem> ConditionBody<Pattern, Item>
    for ConditionBodyMany<Pattern, Item>
{
    fn pattern(&self) -> &Pattern {
        &self.pattern.inner
    }
}

impl<Pattern: MatchPattern, Item: BodyItem> ConditionBody<Pattern, Item>
    for ConditionBodySingle<Pattern, Item>
{
    fn pattern(&self) -> &Pattern {
        &self.pattern.inner
    }
}

/// Marker type for a conditional statement that contextually only accepts one item
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SingleItem;

impl<Item: BodyItem> ConditionBodyCardinality<Item> for SingleItem {
    type Body<Pattern: MatchPattern> = ConditionBodySingle<Pattern, Item>;
    type Block = Option<Spanned<Item>>;
    type EvalResult = Option<Spanned<<Item as BodyItem>::Processed>>;

    fn single_to_block(item: Spanned<Item>) -> Self::Block {
        Some(item)
    }

    fn empty_block() -> Self::Block {
        None
    }

    fn new_body<Pattern: MatchPattern>(
        pattern: Spanned<Pattern>,
        block: Self::Block,
    ) -> Self::Body<Pattern> {
        ConditionBodySingle { pattern, block }
    }

    fn pass_block(block: &Self::Block, ctx: &mut ParseContext) -> Self::EvalResult {
        block.clone().map(|item| {
            let span = item.span;
            Spanned {
                span,
                inner: item.inner.process(ctx),
            }
        })
    }

    fn pass_body<Pattern: MatchPattern>(
        body: &Self::Body<Pattern>,
        ctx: &mut ParseContext,
    ) -> Self::EvalResult {
        Self::pass_block(&body.block, ctx)
    }
}

/// Marker type for a conditional statement that contextually accepts any number of items
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NItems;

impl<Item: BodyItem> ConditionBodyCardinality<Item> for NItems {
    type Body<Pattern: MatchPattern> = ConditionBodyMany<Pattern, Item>;
    type Block = Vec<Spanned<Item>>;
    type EvalResult = Vec<Spanned<<Item as BodyItem>::Processed>>;

    fn single_to_block(item: Spanned<Item>) -> Self::Block {
        vec![item]
    }

    fn empty_block() -> Self::Block {
        Vec::new()
    }

    fn new_body<Pattern: MatchPattern>(
        pattern: Spanned<Pattern>,
        block: Self::Block,
    ) -> Self::Body<Pattern> {
        ConditionBodyMany { pattern, block }
    }

    fn pass_block(block: &Self::Block, ctx: &mut ParseContext) -> Self::EvalResult {
        block
            .iter()
            .cloned()
            .map(|item| {
                let span = item.span;
                Spanned {
                    span,
                    inner: item.inner.process(ctx),
                }
            })
            .collect()
    }

    fn pass_body<Pattern: MatchPattern>(
        body: &Self::Body<Pattern>,
        ctx: &mut ParseContext,
    ) -> Self::EvalResult {
        Self::pass_block(&body.block, ctx)
    }
}

/// a guard for an #if statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionGuard<Scrutinee: Matchable> {
    Single {
        scrutinee: Spanned<Scrutinee>,
        pattern: Spanned<<Scrutinee as Matchable>::Pattern>,
        span: Span,
    },
    And(Vec<ConditionGuard<Scrutinee>>, Span),
    Or(Vec<ConditionGuard<Scrutinee>>, Span),
    Not(Box<ConditionGuard<Scrutinee>>, Span),
    Grouped(Box<ConditionGuard<Scrutinee>>, Span),
}

impl<Scrutinee: Matchable> ConditionGuard<Scrutinee> {
    fn eval(&self, ctx: &mut ParseContext) -> bool {
        match self {
            Self::Single {
                scrutinee, pattern, ..
            } => scrutinee.inner.eval(&pattern.inner, ctx),
            Self::And(items, _) => items.iter().all(|item| item.eval(ctx)),
            Self::Or(items, _) => items.iter().any(|item| item.eval(ctx)),
            Self::Not(item, _) => !item.eval(ctx),
            Self::Grouped(item, _) => item.eval(ctx),
        }
    }
}

/// a branch of an `#if` statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionBranch<
    Scrutinee: Matchable,
    Item: BodyItem,
    Cardinality: ConditionBodyCardinality<Item>,
> {
    pub scrutinee: Spanned<Scrutinee>,
    pub body:
        <Cardinality as ConditionBodyCardinality<Item>>::Body<<Scrutinee as Matchable>::Pattern>,
}

/// a complete `#if {} #else if #else {}` statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionIf<
    Scrutinee: Matchable,
    Item: BodyItem,
    Cardinality: ConditionBodyCardinality<Item>,
> {
    pub arms: Vec<ConditionBranch<Scrutinee, Item, Cardinality>>,
    pub fallback: Option<Cardinality::Block>,
}

/// a `#match` statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionMatch<
    Scrutinee: Matchable,
    Item: BodyItem,
    Cardinality: ConditionBodyCardinality<Item>,
> {
    pub scrutinee: Spanned<Scrutinee>,
    pub arms: Vec<
        Spanned<
            <Cardinality as ConditionBodyCardinality<Item>>::Body<
                <Scrutinee as Matchable>::Pattern,
            >,
        >,
    >,
}

impl<Scrutinee: Matchable, Item: BodyItem, Cardinality: ConditionBodyCardinality<Item>>
    ConditionalItem<Item, Cardinality> for ConditionBranch<Scrutinee, Item, Cardinality>
{
    fn eval(
        &self,
        ctx: &mut ParseContext,
    ) -> Option<<Cardinality as ConditionBodyCardinality<Item>>::EvalResult> {
        let pattern = self.body.pattern();
        if self.scrutinee.inner.eval(pattern, ctx) {
            return Some(<Cardinality as ConditionBodyCardinality<Item>>::pass_body(
                &self.body, ctx,
            ));
        }
        None
    }
}

impl<Scrutinee: Matchable, Item: BodyItem, Cardinality: ConditionBodyCardinality<Item>>
    ConditionalItem<Item, Cardinality> for ConditionMatch<Scrutinee, Item, Cardinality>
{
    fn eval(
        &self,
        ctx: &mut ParseContext,
    ) -> Option<<Cardinality as ConditionBodyCardinality<Item>>::EvalResult> {
        for arm in &self.arms {
            let pattern = arm.inner.pattern();
            if self.scrutinee.inner.eval(pattern, ctx) {
                return Some(<Cardinality as ConditionBodyCardinality<Item>>::pass_body(
                    &arm.inner, ctx,
                ));
            }
        }
        None
    }
}

impl<Scrutinee: Matchable, Item: BodyItem, Cardinality: ConditionBodyCardinality<Item>>
    ConditionalItem<Item, Cardinality> for ConditionIf<Scrutinee, Item, Cardinality>
{
    fn eval(
        &self,
        ctx: &mut ParseContext,
    ) -> Option<<Cardinality as ConditionBodyCardinality<Item>>::EvalResult> {
        for arm in &self.arms {
            if let Some(result) = arm.eval(ctx) {
                return Some(result);
            }
        }
        if let Some(fallback) = &self.fallback {
            return Some(<Cardinality as ConditionBodyCardinality<Item>>::pass_block(
                fallback, ctx,
            ));
        }
        None
    }
}

/// a conditional item behind an if or match statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition<
    Scrutinee: Matchable,
    Item: BodyItem,
    Cardinality: ConditionBodyCardinality<Item>,
> {
    If(ConditionIf<Scrutinee, Item, Cardinality>),
    Match(ConditionMatch<Scrutinee, Item, Cardinality>),
}

impl<Scrutinee: Matchable, Item: BodyItem, Cardinality: ConditionBodyCardinality<Item>>
    ConditionalItem<Item, Cardinality> for Condition<Scrutinee, Item, Cardinality>
{
    fn eval(
        &self,
        ctx: &mut ParseContext,
    ) -> Option<<Cardinality as ConditionBodyCardinality<Item>>::EvalResult> {
        match self {
            Self::If(item) => item.eval(ctx),
            Self::Match(item) => item.eval(ctx),
        }
    }
}

/// a trait that helps build combined parsers for ConditionalItem's that accept `#if {} #else {}` or `#match`
pub trait Conditional<'src, Item: BodyItem, Cardinality: ConditionBodyCardinality<Item>> {
    type Scrutinee: MatchableParse<'src>;
    fn if_parser(
        item_parser: impl ZngParser<'src, Item> + 'src,
    ) -> BoxedZngParser<'src, Condition<Self::Scrutinee, Item, Cardinality>>;
    fn match_parser(
        item_parser: impl ZngParser<'src, Item> + 'src,
    ) -> BoxedZngParser<'src, Condition<Self::Scrutinee, Item, Cardinality>>;
}
