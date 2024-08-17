use actix_web::{get, web, HttpResponse, Responder};

use crate::api::common::set_cors_headers;
use crate::api::StateApi;
use crate::api::response::MonitorResponse;

#[get("/monitors/status")]
pub async fn get_monitor_status(state: web::Data<StateApi>) -> impl Responder {
    let monitor_statuses = state.monitoring_service.get_all_monitorstatuses();
    let mut response_builder = HttpResponse::Ok();
    set_cors_headers(&mut response_builder, &state.server_config);
    response_builder.json(MonitorResponse::from_monitor_status_messages(&monitor_statuses))  
            
          
}