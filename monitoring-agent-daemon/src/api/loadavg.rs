use actix_web::{get, web, HttpResponse, Responder};

use crate::api::StateApi;
use crate::api::response::LoadavgResponse;

/**
 * Get current load average.
 * 
 * `state`: The state object.
 * 
 * Returns the load average or an error.
 */
#[get("/loadavg/current")]
pub async fn get_current_loadavg(state: web::Data<StateApi>) -> impl Responder {
    let loadavg = state.monitoring_service.get_current_loadavg();
    match loadavg {
        Ok(loadavg) => HttpResponse::Ok().json(LoadavgResponse::from_loadavg(&loadavg)),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}