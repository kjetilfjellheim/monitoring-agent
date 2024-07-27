use actix_web::{get, web, HttpResponse, Responder};

use crate::api::StateApi;
use crate::api::response::CpuinfoResponse;

/**
 * Get cpu information.
 * 
 * `state`: The state object.
 * 
 * Returns the cpu information or an error.
 */
#[get("/cpuinfo/current")]
pub async fn get_current_cpuinfo(state: web::Data<StateApi>) -> impl Responder {
    let procscpuinfo = state.monitoring_service.get_current_cpuinfo();
    match procscpuinfo {
        Ok(procscpuinfo) => HttpResponse::Ok().json(CpuinfoResponse::from_cpuinfo(&procscpuinfo)),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}