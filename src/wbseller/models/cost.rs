use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AcceptanceCostsResponse {
    pub result: AcceptanceCostsResult,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AcceptanceCostsResult {
    pub costs: Vec<Cost>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Cost {
    pub coefficient: f64,
    pub cost: f64,
    pub date: String,

    #[serde(rename = "deliveryAndStorage")]
    pub delivery_and_storage: DeliveryAndStorage,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DeliveryAndStorage {
    #[serde(rename = "deliveryAndStorageExpr")]
    pub delivery_and_storage_expr: String,

    #[serde(rename = "deliveryCoef")]
    pub delivery_coef: String,

    #[serde(rename = "deliveryValueBase")]
    pub delivery_value_base: String,

    #[serde(rename = "deliveryValueLiter")]
    pub delivery_value_liter: String,

    #[serde(rename = "storageCoef")]
    pub storage_coef: String,

    #[serde(rename = "storageLiter")]
    pub storage_liter: String,

    #[serde(rename = "storageValue")]
    pub storage_value: String,

    #[serde(rename = "storageVolumeCut")]
    pub storage_volume_cut: String,
}
