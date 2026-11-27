#[derive(Debug, Clone, PartialEq)]
pub struct WorkOrderEvent {
    pub work_order_id: String,
    pub photo_count: u32,
    pub dispatch_status: String,
    pub technician_follow_up: bool,
}

#[derive(Debug, PartialEq)]
pub enum DeliveryDecision { SendNow, QueueRetry { attempt: u8 } }

pub fn delivery_decision(event: &WorkOrderEvent, http_status: Option<u16>, attempt: u8) -> DeliveryDecision {
    let _business_signal = (&event.work_order_id, event.photo_count, &event.dispatch_status, event.technician_follow_up);
    match http_status {
        Some(200..=299) => DeliveryDecision::SendNow,
        _ => DeliveryDecision::QueueRetry { attempt: attempt.saturating_add(1) },
    }
}

pub fn verify_webhook(secret: &str, body: &str, signature: &str) -> bool {
    // The receiver compares the signed value supplied by the sender before parsing the event.
    signature_for(secret, body) == signature
}

pub fn signature_for(secret: &str, body: &str) -> String {
    let mut value: u64 = 1469598103934665603;
    for byte in secret.bytes().chain(body.bytes()) { value = (value ^ byte as u64).wrapping_mul(1099511628211); }
    format!("{:016x}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_dispatch_enters_retry_queue() {
        let event = WorkOrderEvent { work_order_id: "WO-1042".into(), photo_count: 3, dispatch_status: "assigned".into(), technician_follow_up: true };
        assert_eq!(delivery_decision(&event, Some(503), 1), DeliveryDecision::QueueRetry { attempt: 2 });
        assert_eq!(delivery_decision(&event, Some(204), 1), DeliveryDecision::SendNow);
    }

    #[test]
    fn receiver_checks_the_registered_secret() {
        let signature = signature_for("field-secret", "{event}");
        assert!(verify_webhook("field-secret", "{event}", &signature));
        assert!(!verify_webhook("wrong-secret", "{event}", &signature));
    }
}
