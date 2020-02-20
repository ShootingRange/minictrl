use crate::actors::database::{DbActorError, DbExecutor};
use crate::database::models::{Server, NewServer};
use actix::{Handler, Message};
use diesel::prelude::*;
use std::net::IpAddr;
use ipnetwork::IpNetwork;

pub struct CreateServer {
    pub host: IpAddr,
    pub port: u16,
    pub r#type: Option<String>,
}

impl Message for CreateServer {
    type Result = Result<Server, DbActorError>;
}

impl Handler<CreateServer> for DbExecutor {
    type Result = Result<Server, DbActorError>;

    fn handle(&mut self, msg: CreateServer, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::servers::dsl::*;

        let server = NewServer {
            host: IpNetwork::from(msg.host),
            port: msg.port as i32,
            type_: msg.r#type,
        };

        diesel::insert_into(servers)
            .values(&server)
            .get_result::<Server>(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))
    }
}

pub struct FindServerById {
    pub id: i32,
}

impl Message for FindServerById {
    type Result = Result<Server, DbActorError>;
}

impl Handler<FindServerById> for DbExecutor {
    type Result = Result<Server, DbActorError>;

    fn handle(&mut self, msg: FindServerById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::servers::dsl::*;

        match servers.filter(id.eq(msg.id)).first::<Server>(&self.conn) {
            Ok(t) => Ok(t),
            Err(err) => Err(DbActorError::DatabaseError(err)),
        }
    }
}

pub struct UpdateServer {
    // TODO
}

impl Message for UpdateServer {
    type Result = Result<Server, DbActorError>;
}

pub struct DeleteServerById {
    pub id: i32,
}

impl Message for DeleteServerById {
    type Result = Result<bool, DbActorError>;
}

impl Handler<DeleteServerById> for DbExecutor {
    type Result = Result<bool, DbActorError>;

    fn handle(&mut self, msg: DeleteServerById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::servers::dsl::*;

        diesel::delete(servers.filter(id.eq(msg.id)))
            .execute(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))
            .map(|size| size > 0)
    }
}
