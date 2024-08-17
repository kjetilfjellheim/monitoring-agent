use actix_web::{get, web, HttpResponse, Responder};

use crate::api::{common::set_cors_headers, response::StatmResponse, StateApi};

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
        Ok(procs) => {
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(ProcessResponse::from_processes(&procs))
        },
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}

/**
 * Get current statm.
 * 
 * `state`: The state object.
 * `path`: The path object.
 * 
 * Returns the current statm.
 */
#[get("/processes/{pid}/statm")]
pub async fn get_current_statm(state: web::Data<StateApi>, path: web::Path<u32>) -> impl Responder {
    let pid: u32 = path.into_inner();
    let procs_statm = state.monitoring_service.get_current_statm(pid);
    match procs_statm {
        Ok(procs_statm) => {
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(StatmResponse::from_current_statm(&procs_statm))
        },
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}
