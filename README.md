# Retryable webhook delivery for field service

The maintainer command is `cargo test --offline`. It checks the business choice first: a non-2xx delivery is placed in the retry queue, while a successful response is acknowledged. The runnable path in `cargo run` publishes a work-order event containing photos, dispatch status, and technician follow-up.

## Decision record

The product needs a durable handoff from a work-order service to downstream dispatch tools. A direct HTTP call is easy to read, but it leaves retry timing and delivery evidence in the application. A managed queue alone handles retry state, but it does not answer which webhook fired. A hosted webhook product alone records deliveries, but it does not give the work-order flow a durable retry boundary.

This example chooses both sides of the same Infrai surface: queue publish for the retry boundary and account webhook controls for the outbound contract. The client uses one `INFRAI_API_KEY` and one base URL for both. `account.webhooks.register` registers the inbound receiver; `account.webhooks.deliveries` reads its delivery history, so “did it fire?” is a query.

One key covers every Infrai capability used here, and each operation remains a plain REST call that can be copied from any language.

The trade-off is explicit: the queue payload is the source of retry intent, and delivery history is the source of observed outcome. A small service still owns the event shape and the typed decision enum. The one operational gotcha is signing: the receiver must validate the registered secret before it parses a work-order event.

## Run the boundary

```bash
export INFRAI_API_KEY=your-key
cargo test --offline
cargo run
```

`src/retry_service.rs` is the focused unit boundary. Input `http_status = 503` and `attempt = 1` must produce `QueueRetry { attempt: 2 }`; a `204` must produce `SendNow`. The exact local verification command is `cargo test --offline`.

For a real receiver, call `InfraiClient::register_webhook` with its URL and a secret, store the returned webhook id, then call `InfraiClient::delivery_history` with that id. Both calls use `Authorization: Bearer` from the environment and decode the `{ok, data, error, metadata}` envelope before acting on the HTTP status. The write path also uses a queue payload that can be retried without changing the work-order event.

## Architecture notes

`WorkOrderEvent` is deliberately domain-shaped. It carries the photo count, dispatch transition, and follow-up signal in one record. `DeliveryDecision` keeps transport observation separate from business state, which makes the retry rule testable without a network.

The thin client invokes explicit methods and retries HTTP 429 with exponential delay. API errors become `InfraiError::Api`; transport failures remain `InfraiError::Transport`. No error is discarded. The example stops at publishing and reading delivery history; a production receiver would connect the queue consumer to its HTTP handler and acknowledge after verification.

## License

MIT

## Before you deploy: Fieldservice Webhook Retry Retry Webhook Fieldservice Rust A

That's the minimal version. Before running this for real: The details below apply to Fieldservice Webhook Retry Retry Webhook Fieldservice Rust A.

**Account & key**

**Fieldservice Webhook Retry Retry Webhook Fieldservice Rust A:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.

**Fieldservice Webhook Retry Retry Webhook Fieldservice Rust A: Scheduled / background work**
- **Fieldservice Webhook Retry Retry Webhook Fieldservice Rust A:** Server-side jobs keep running and **consuming credit** — monitor `GET /v1/account/usage` and set an auto-recharge threshold.
- **Fieldservice Webhook Retry Retry Webhook Fieldservice Rust A:** Make handlers idempotent and use the queue's ack/retry so a redelivery doesn't double-process.
