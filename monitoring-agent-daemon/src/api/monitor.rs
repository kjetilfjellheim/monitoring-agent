use actix_web::{get, web, HttpResponse, Responder};

use crate::api::StateApi;
use crate::api::response::MonitorResponse;

#[get("/monitors/status")]
pub async fn get_monitor_status(state: web::Data<StateApi>) -> impl Responder {
    let monitor_statuses = state.monitoring_service.get_all_monitorstatuses();
    HttpResponse::Ok().json(MonitorResponse::from_monitor_status_messages(&monitor_statuses))    
}