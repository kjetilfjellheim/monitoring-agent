use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::api::common::set_cors_headers;
use crate::api::request::HistoricalParams;
use crate::api::response::{LoadavgHistoricalResponse, LoadavgResponse};
use crate::api::StateApi;

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
        Ok(loadavg) => {
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(LoadavgResponse::from_loadavg(&loadavg))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}

/**
 * Get historical load average.
 *
 * `state`: The state object.
 *
 * Returns the historical load average or an error.
 */
#[get("/loadavg/historical")]
pub async fn get_historical_loadavg(
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
     * Get the historical load average.
     */
    let loadavg = db_service.get_historical_loadavg(params.0);
    /*
     * Return the response.
     */
    match loadavg {
        Ok(loadavg) => {
            let mut response_builder = HttpResponse::Ok();
            set_cors_headers(&mut response_builder, &state.server_config);
            response_builder.json(LoadavgHistoricalResponse::from_loadavg_historical(&loadavg))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error occured: {err:?}")),
    }
}
