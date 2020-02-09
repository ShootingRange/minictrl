#[macro_use]
extern crate diesel;
extern crate ipnetwork;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate diesel_derive_enum;

pub mod common;
pub mod get5;
pub(crate) mod database;
pub mod actors;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
