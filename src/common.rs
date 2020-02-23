#[derive(Clone, Debug, DbEnum, juniper::GraphQLEnum)]
#[PgType = "side"]
#[DieselType = "Side"]
pub enum SideType {
    Standard,
    NeverKnife,
    AlwaysKnife,
}
