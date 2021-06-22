use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;

mod get5;

fn with_db(
    db: Arc<Database>,
) -> impl Filter<Extract = (Arc<Database>,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn router(db: Arc<Database>) -> BoxedFilter<(impl Reply,)> {
    let get5_config = warp::path("get5")
        .and(warp::path("config"))
        .and(warp::path::param::<i32>())
        .and(warp::get())
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(get5::handler_get5_config)
        .boxed();

    let router = get5_config;

    router
}
