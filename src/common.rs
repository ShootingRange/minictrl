#[derive(Clone, Debug, DbEnum)]
pub enum Side {
    Standard,
    NeverKnife,
    AlwaysKnife,
}
