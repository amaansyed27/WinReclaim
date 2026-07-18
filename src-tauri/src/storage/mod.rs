use crate::domain::{CleanupPlan, CleanupReceipt, ScanReport};
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct AppState {
    pub cancel_scan: AtomicBool,
    pub latest_scan: Mutex<Option<ScanReport>>,
    pub plans: Mutex<HashMap<Uuid, CleanupPlan>>,
    pub receipts: Mutex<Vec<CleanupReceipt>>,
}
