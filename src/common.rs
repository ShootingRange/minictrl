use async_graphql::*;

#[derive(Clone, Debug, Copy, Eq, PartialEq, Enum, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum SideType {
    #[graphql(name = "standard")]
    Standard,
    #[graphql(name = "never_knife")]
    NeverKnife,
    #[graphql(name = "always_knife")]
    AlwaysKnife,
}
