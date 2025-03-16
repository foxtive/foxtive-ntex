use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct IdsVecDto {
    pub ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub struct IdAsUuid {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct IdPathParam {
    pub id: String,
}

#[cfg(feature = "validator")]
#[derive(Deserialize, validator::Validate)]
pub struct ReasonPayload {
    #[validate(length(min = 3, max = 1500))]
    pub reason: String,
}
