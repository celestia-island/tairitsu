use anyhow::Result;
use hyper::StatusCode;
use serde_json::to_string;

use axum::Json;

use crate::routes::utils::generate_error_message;
use tairitsu_database::functions::user as functions;
use tairitsu_utils::types::proto::{
    frontend::UuidData, RequestPackage, RequestPackage::Verify as RequestType, ResponsePackage,
    ResponseStruct::Token as ResponseType,
};

pub async fn verify(Json(item): Json<RequestPackage>) -> Result<String, (StatusCode, String)> {
    let item = match &item {
        RequestType(item) => item.to_owned(),
        _ => return Err(generate_error_message("Invalid request".to_string())),
    };

    let storage = functions::filter_by_name(item.name)
        .await
        .map_err(|e| generate_error_message(e.to_string()))?;
    if item.token == storage.token {
        let ret = ResponsePackage::Data(vec![ResponseType(UuidData {
            uuid: storage.token,
        })]);
        to_string(&ret).map_err(|e| generate_error_message(e.to_string()))
    } else {
        Err(generate_error_message("Invalid token".to_string()))
    }
}
