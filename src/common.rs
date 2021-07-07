use async_graphql::*;
use rand::Fill;

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

pub(crate) fn generate_password() -> anyhow::Result<String> {
    const SIZE: usize = 16;

    let mut rng = rand::thread_rng();
    let mut password_bytes = [0u8; SIZE];
    password_bytes.try_fill(&mut rng)?;

    let password = hex::encode(password_bytes);
    Ok(password)
}
