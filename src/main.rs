mod infrai_client;
mod retry_service;

use infrai_client::InfraiClient;
use retry_service::{delivery_decision, verify_webhook, DeliveryDecision, WorkOrderEvent};

fn main() -> Result<(), String> {
    let client = InfraiClient::from_env().map_err(|e| format!("client setup: {:?}", e))?;
    let event = WorkOrderEvent { work_order_id: "WO-1042".into(), photo_count: 3, dispatch_status: "assigned".into(), technician_follow_up: true };
    let decision = delivery_decision(&event, Some(503), 0);
    if let DeliveryDecision::QueueRetry { attempt } = decision {
        let payload = format!("{{\"work_order_id\":\"{}\",\"photo_count\":{},\"dispatch_status\":\"{}\",\"technician_follow_up\":{}}}", event.work_order_id, event.photo_count, event.dispatch_status, event.technician_follow_up);
        client.publish("fieldservice-webhooks", &payload).map_err(|e| format!("publish: {:?}", e))?;
        println!("queued retry {} for {}", attempt, event.work_order_id);
    }
    println!("signature accepted: {}", verify_webhook("field-secret", "{event}", "0000000000000000"));
    Ok(())
}
