#[derive(Clone, Debug, DbEnum)]
#[PgType = "side"]
#[DieselType = "Side"]
pub enum SideType {
    Standard,
    NeverKnife,
    AlwaysKnife,
}
