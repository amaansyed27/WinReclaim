use crate::domain::{CleanupPlan, CleanupReceipt, ScanReport};
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub cancel_scan: Arc<AtomicBool>,
    pub latest_scan: Arc<Mutex<Option<ScanReport>>>,
    pub plans: Arc<Mutex<HashMap<Uuid, CleanupPlan>>>,
    pub receipts: Arc<Mutex<Vec<CleanupReceipt>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            cancel_scan: Arc::new(AtomicBool::new(false)),
            latest_scan: Arc::new(Mutex::new(None)),
            plans: Arc::new(Mutex::new(HashMap::new())),
            receipts: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
