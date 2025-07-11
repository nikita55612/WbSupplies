#[inline]
pub fn preorder_id_to_url(id: i64) -> String {
    format!(
        "https://seller.wildberries.ru/supplies-management/all-supplies/supply-detail?preorderId={id}&supplyId"
    )
}
