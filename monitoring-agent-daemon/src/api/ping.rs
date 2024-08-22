use actix_web::{get, web, HttpResponse, Responder};

use crate::api::{common::set_cors_headers, response::PingResponse, StateApi};

/**
 * Get ping.
 * 
 * `state`: The state object.
 * 
 * Returns ping data.
 */
#[get("/")]
pub async fn get_ping(state: web::Data<StateApi>) -> impl Responder {
    let mut response_builder = HttpResponse::Ok();
    set_cors_headers(&mut response_builder, &state.server_config);
    response_builder.json(PingResponse::new("Ok","monitoring-agent-daemon"))  
}