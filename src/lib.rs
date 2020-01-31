#[macro_use]
extern crate diesel;
extern crate ipnetwork;
extern crate serde;
extern crate serde_json;

pub mod get5;
pub(crate) mod models;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
