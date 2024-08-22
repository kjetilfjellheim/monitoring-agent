use actix_web::{get, web, HttpResponse, Responder};

use crate::api::response::StatResponse;

use super::{common::set_cors_headers, StateApi};

/**
 * Get stat cpu statistics
 * 
 * `state`: The state object.
 * 
 * Returns the current memory information or an error.
 */
#[get("/stat/current")]
pub async fn get_stat(state: web::Data<StateApi>) -> impl Responder {
    let procs = state.monitoring_service.get_stat();    
    match procs {
        Ok(procs) => { 
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(StatResponse::from_stat(&procs))
        },
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}