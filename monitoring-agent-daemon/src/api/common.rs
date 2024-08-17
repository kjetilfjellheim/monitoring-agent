use actix_web::HttpResponseBuilder;

use crate::common::configuration::ServerConfig;

/**
 * Set CORS headers.
 * 
 * `response_builder`: The response builder.
 * `server_config`: The server configuration.
 */
pub fn set_cors_headers(response_builder: &mut HttpResponseBuilder, server_config: &ServerConfig) {
    if let Some(header) = &server_config.access_control_allow_origin { response_builder.append_header(("Access-Control-Allow-Origin", header.to_string())); }
    if let Some(header) = &server_config.access_control_allow_headers { response_builder.append_header(("Access-Control-Allow-Headers", header.to_string())); }
    if let Some(header) = &server_config.access_control_allow_methods { response_builder.append_header(("Access-Control-Allow-Methods", header.to_string())); }
    if let Some(header) = server_config.access_control_allow_credentials { response_builder.append_header(("Access-Control-Allow-Credentials", header.to_string())); }
    if let Some(header) = server_config.access_control_max_age { response_builder.append_header(("Access-Control-Allow-Credentials", header)); }
}