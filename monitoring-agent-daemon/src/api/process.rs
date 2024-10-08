use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::api::{common::set_cors_headers, response::{ProcessMeminfoHistoricalResponse, StatmResponse}, HistoricalParams, StateApi};

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
        Ok(procs) => { 
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(ProcessResponse::from_processes(&procs))
        },
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
        Ok(procs) => { 
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);            
            response_builder.json(ProcessResponse::from_process(&procs)) 
        },
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

/**
 * Get current historical statm for a process.
 * 
 * `state`: The state object.
 * `path`: The path object.
 * 
 * Returns the current statm.
 */
#[get("/processes/{pid}/statm/historical")]
pub async fn get_historical_statm(state: web::Data<StateApi>, path: web::Path<u32>, req: HttpRequest) -> impl Responder {
    let pid: u32 = path.into_inner();
    /*
     * If the database service is not found, return a 404. 
     */
    let Some(db_service) = state.database_service.as_ref() else {
        return HttpResponse::NotFound().body("Database service not found");
    };
    /*
     * Parse the query string.
     */
    let params = match web::Query::<HistoricalParams>::from_query(req.query_string()) {
        Ok(params) => params,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Error parsing query string: {err:?}"))
        }
    };
    /*
     * Get the historical meminfo.
     */
    let meminfo = db_service.get_process_memory_use(pid, params.0);
    /*
     * Return the response.
     */
    match meminfo {
        Ok(meminfo) => {
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(ProcessMeminfoHistoricalResponse::from_process_meminfo_historical(&meminfo))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}
