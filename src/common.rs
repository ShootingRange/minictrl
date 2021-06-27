use async_graphql::*;

#[derive(Clone, Debug, Copy, Eq, PartialEq, Enum)]
pub enum SideType {
    #[graphql(name = "standard")]
    Standard,
    #[graphql(name = "never_knife")]
    NeverKnife,
    #[graphql(name = "always_knife")]
    AlwaysKnife,
}
