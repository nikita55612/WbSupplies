use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ListSuppliesResponse {
    pub result: ListSuppliesResult,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ListSuppliesResult {
    pub data: Vec<Supply>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Supply {
    #[serde(default, rename = "acceptanceCost")]
    pub acceptance_cost: f64,

    #[serde(default, rename = "acceptanceLiterBase")]
    pub acceptance_liter_base: f64,

    #[serde(default, rename = "acceptanceLiterValue")]
    pub acceptance_liter_value: f64,

    #[serde(default, rename = "actualWarehouseAddress")]
    pub actual_warehouse_address: Option<String>,

    #[serde(default, rename = "actualWarehouseID")]
    pub actual_warehouse_id: Option<i64>,

    #[serde(default, rename = "actualWarehouseMapID")]
    pub actual_warehouse_map_id: Option<i64>,

    #[serde(default, rename = "actualWarehouseName")]
    pub actual_warehouse_name: String,

    #[serde(default, rename = "boxTypeId")]
    pub box_type_id: i64,

    #[serde(default, rename = "boxTypeName")]
    pub box_type_name: String,

    #[serde(default, rename = "canShowQuantity")]
    pub can_show_quantity: bool,

    #[serde(default, rename = "changeDate")]
    pub change_date: String,

    #[serde(default, rename = "createDate")]
    pub create_date: String,

    #[serde(default, rename = "detailsQuantity")]
    pub details_quantity: i64,

    #[serde(default, rename = "factDate")]
    pub fact_date: Option<String>,

    #[serde(default, rename = "feedbackAllowed")]
    pub feedback_allowed: bool,

    #[serde(default, rename = "feedbackArrangementAllowed")]
    pub feedback_arrangement_allowed: bool,

    #[serde(default, rename = "feedbackDispatchmentAllowed")]
    pub feedback_dispatchment_allowed: bool,

    #[serde(default, rename = "hasBoxes")]
    pub has_boxes: bool,

    #[serde(default, rename = "hasPass")]
    pub has_pass: bool,

    #[serde(default, rename = "hasUnloadProblems")]
    pub has_unload_problems: bool,

    #[serde(default, rename = "incomeQuantity")]
    pub income_quantity: i64,

    #[serde(default, rename = "isSplitFeedbackForWarehouse")]
    pub is_split_feedback_for_warehouse: bool,

    #[serde(default, rename = "isWrongDate")]
    pub is_wrong_date: bool,

    #[serde(default, rename = "monopalletAcceptanceCost")]
    pub monopallet_acceptance_cost: f64,

    #[serde(default, rename = "monopalletQuantity")]
    pub monopallet_quantity: Option<i64>,

    #[serde(default, rename = "oldAcceptanceCost")]
    pub old_acceptance_cost: Option<f64>,

    #[serde(default, rename = "oldFeedbackAllowed")]
    pub old_feedback_allowed: bool,

    #[serde(default, rename = "oldPaidAcceptanceCoefficient")]
    pub old_paid_acceptance_coefficient: Option<f64>,

    #[serde(default, rename = "paidAcceptanceCoefficient")]
    pub paid_acceptance_coefficient: Option<f64>,

    #[serde(default, rename = "passMonopalletQuantity")]
    pub pass_monopallet_quantity: Option<i64>,

    #[serde(default, rename = "preorderId")]
    pub preorder_id: Option<i64>,

    #[serde(default, rename = "rejectReason")]
    pub reject_reason: Option<String>,

    #[serde(default, rename = "statusId")]
    pub status_id: i64,

    #[serde(default, rename = "statusName")]
    pub status_name: String,

    #[serde(default, rename = "supplierAssignName")]
    pub supplier_assign_name: Option<String>,

    #[serde(default, rename = "supplierAssignUUID")]
    pub supplier_assign_uuid: Option<String>,

    #[serde(default, rename = "supplierBoxAmount")]
    pub supplier_box_amount: Option<i64>,

    #[serde(default, rename = "supplyDate")]
    pub supply_date: Option<String>,

    #[serde(default, rename = "supplyId")]
    pub supply_id: Option<i64>,

    #[serde(default, rename = "tariffPallet")]
    pub tariff_pallet: Option<f64>,

    #[serde(default, rename = "tariffVolume")]
    pub tariff_volume: Option<f64>,

    #[serde(default, rename = "transitCost")]
    pub transit_cost: Option<f64>,

    #[serde(default, rename = "transitWarehouseAddress")]
    pub transit_warehouse_address: Option<String>,

    #[serde(default, rename = "transitWarehouseId")]
    pub transit_warehouse_id: Option<i64>,

    #[serde(default, rename = "transitWarehouseMapID")]
    pub transit_warehouse_map_id: Option<i64>,

    #[serde(default, rename = "transitWarehouseName")]
    pub transit_warehouse_name: Option<String>,

    #[serde(default, rename = "userUid")]
    pub user_uid: String,

    #[serde(default, rename = "virtualType")]
    pub virtual_type: Option<String>,

    #[serde(default, rename = "volume")]
    pub volume: Option<f64>,

    #[serde(default, rename = "warehouseAddress")]
    pub warehouse_address: String,

    #[serde(default, rename = "warehouseId")]
    pub warehouse_id: i64,

    #[serde(default, rename = "warehouseMapID")]
    pub warehouse_map_id: i64,

    #[serde(default, rename = "warehouseName")]
    pub warehouse_name: String,
}
