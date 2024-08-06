use actix_web::{get, web, HttpResponse, Responder};

use crate::api::StateApi;

use super::response::ProcessResponse;

/**
 * Get list of processes.
 * 
 * `state`: The state object.
 * 
 * Returns the current memory information or an error.
 */
#[get("/processes")]
pub async fn get_processes(state: web::Data<StateApi>) -> impl Responder {
    let procs = state.monitoring_service.get_processes();    
    match procs {
        Ok(procs) => HttpResponse::Ok().json(ProcessResponse::from_processes(&procs)),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}

/**
 * Get process.
 * 
 * `state`: The state object.
 * 
 * Returns the specified process.
 */
#[get("/processes/{pid}")]
pub async fn get_process(state: web::Data<StateApi>, path: web::Path<u32>) -> impl Responder {
    let pid: u32 = path.into_inner();
    let procs = state.monitoring_service.get_process(pid);    
    match procs {
        Ok(procs) => HttpResponse::Ok().json(ProcessResponse::from_process(&procs)),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}

/**
 * Get threads.
 * 
 * `state`: The state object.
 * 
 * Returns the specified process.
 */
#[get("/processes/{pid}/threads")]
pub async fn get_threads(state: web::Data<StateApi>, path: web::Path<u32>) -> impl Responder {
    let pid: u32 = path.into_inner();
    let procs = state.monitoring_service.get_process_threads(pid);    
    match procs {
        Ok(procs) => HttpResponse::Ok().json(ProcessResponse::from_processes(&procs)),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}
