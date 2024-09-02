use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::api::common::set_cors_headers;
use crate::api::{HistoricalParams, StateApi};
use crate::api::response::{MeminfoHistoricalResponse, MeminfoResponse};

/**
 * Get current memory information.
 * 
 * `state`: The state object.
 * 
 * Returns the current memory information or an error.
 */
#[get("/meminfo/current")]
pub async fn get_current_meminfo(state: web::Data<StateApi>) -> impl Responder {
    let procsmeminfo = state.monitoring_service.get_current_meminfo();
    match procsmeminfo {
        Ok(procsmeminfo) => {
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(MeminfoResponse::from_meminfo(&procsmeminfo))            
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}
/**
 * Get historical meminfo.
 *
 * `state`: The state object.
 *
 * Returns the memory use.
 */
#[get("/meminfo/historical")]
pub async fn get_historical_meminfo(
    state: web::Data<StateApi>,
    req: HttpRequest,
) -> impl Responder {
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
    let meminfo = db_service.get_historical_meminfo(params.0);
    /*
     * Return the response.
     */
    match meminfo {
        Ok(meminfo) => {
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(MeminfoHistoricalResponse::from_meminfo_historical(&meminfo))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}
