use anyhow::Result;
use hyper::StatusCode;
use serde_json::to_string;
use uuid::Uuid;

use axum::Json;

use crate::routes::utils::generate_error_message;
use tairitsu_database::functions::user as functions;
use tairitsu_utils::{
    crypt_generate,
    types::proto::{
        frontend::UuidData, RequestPackage, RequestPackage::Login as RequestType, ResponsePackage,
        ResponseStruct::Token as ResponseType,
    },
};

pub async fn register(Json(item): Json<RequestPackage>) -> Result<String, (StatusCode, String)> {
    let item = match &item {
        RequestType(item) => item.to_owned(),
        _ => return Err(generate_error_message("Invalid request".to_string())),
    };

    let mut storage = functions::filter_by_name(item.name.clone())
        .await
        .map_err(|e| generate_error_message(e.to_string()))?;
    if storage.name == item.name.clone() {
        Err(generate_error_message("Name already exists".to_string()))
    } else {
        let new_token = Uuid::new_v4();
        storage.name = item.name;
        storage.password_hash = crypt_generate(item.password_hash)
            .map_err(|e| generate_error_message(e.to_string()))?;
        storage.token = new_token.to_owned();
        functions::insert(storage)
            .await
            .map_err(|e| generate_error_message(e.to_string()))?;

        let ret = ResponsePackage::Data(vec![ResponseType(UuidData { uuid: new_token })]);
        to_string(&ret).map_err(|e| generate_error_message(e.to_string()))
    }
}
