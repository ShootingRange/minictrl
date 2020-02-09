#[macro_use]
extern crate diesel;
extern crate ipnetwork;
#[macro_use]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate juniper;

pub mod common;
pub mod get5;
pub mod database;
pub mod actors;
pub mod web;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
