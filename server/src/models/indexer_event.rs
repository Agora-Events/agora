//! # Strongly-typed Soroban event decoding (Issue #1174)
//!
//! Soroban RPC returns contract events whose topics and data payloads are
//! base64-XDR `ScVal`s. This module decodes those into typed Rust structs so
//! the pipeline never matches on raw base64 strings.
//!
//! ## Pipeline flow
//!
//! ```text
//! RPC event ──> decode_topic() ──> EventKind (typed)
//!           └──> normalize_payload() ──> JSON payload
//!                         └──> IndexedEvent::decode() ──> typed structs
//! ```
//!
//! Decoding is deliberately *tolerant*: an undecodable event is routed to
//! [`EventKind::Unhandled`] instead of failing the whole batch, so a schema
//! drift in one topic can never stall the indexer.

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use stellar_xdr::curr::{Limits, PublicKey, ReadXdr, ScAddress, ScMapEntry, ScVal};

/// The typed, high-level contract event kinds the indexer understands.
///
/// The Soroban contracts emit an `AgoraEvent` enum whose symbols map onto
/// these kinds (both camel-case on-chain names and the snake_case entrypoint
/// aliases referenced in the issue are recognised).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// `event_registry::EventRegistered` / `register_event`
    RegisterEvent,
    /// `ticket_payment::PaymentProcessed` / `process_purchase`
    ProcessPurchase,
    /// `ticket_payment::BulkRefundProcessed | PartialRefundProcessed | CancellationRefundClaimed` / `refund`
    Refund,
    /// `ticket_payment::TicketTransferred` / `transfer_ticket`
    TransferTicket,
    /// `event_registry::EventStatusUpdated | EventCancelled`
    EventStatusUpdate,
    /// `event_registry::CollateralStaked`
    CollateralStaked,
    /// `event_registry::CollateralUnstaked`
    CollateralUnstaked,
    /// Anything else — decode succeeded but no state change is defined.
    Unhandled,
}

/// Typed payload extracted from a `PaymentProcessed` event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PurchasePayload {
    pub payment_id: String,
    pub event_id: String,
    pub buyer: Option<String>,
    pub owner: Option<String>,
    /// Token amount in stroops, when the RPC decoded it.
    pub amount: Option<i128>,
}

/// Typed payload extracted from a refund-style event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RefundPayload {
    pub payment_id: String,
    pub event_id: String,
}

/// Typed payload extracted from a `TicketTransferred` event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TransferPayload {
    pub payment_id: String,
    pub event_id: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Typed payload extracted from an `EventRegistered` event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RegisterPayload {
    pub event_id: String,
    pub organizer: Option<String>,
}

/// A decoded contract event carrying both the typed kind and a normalized,
/// JSON-serialisable payload for state application and the WebSocket
/// broadcaster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedEvent {
    /// Opaque RPC pagination id — the natural de-duplication key.
    pub id: String,
    /// Ledger sequence of the emitting block.
    pub ledger: u32,
    /// Emitting contract id.
    pub contract_id: String,
    /// Highest ledger the RPC reported when this event was fetched.
    pub latest_ledger: u32,
    /// Strongly-typed event kind.
    pub kind: EventKind,
    /// Raw on-chain symbol that was decoded.
    pub topic_name: String,
    /// Normalized JSON payload (addresses already converted to strkeys).
    pub value: Value,
}

impl IndexedEvent {
    /// Build a typed event from a raw RPC event + the `latest_ledger` value
    /// the same `getEvents` response carried.
    pub fn decode(
        id: String,
        ledger: u32,
        contract_id: String,
        topic: &[String],
        value: &Value,
        latest_ledger: u32,
    ) -> Self {
        let topic_name = decode_topic_symbol(topic.first());
        let kind = event_kind_for_topic(&topic_name);
        let payload = normalize_payload(value);

        Self {
            id,
            ledger,
            contract_id,
            latest_ledger,
            kind,
            topic_name,
            value: payload,
        }
    }

    /// Extract the typed purchase payload (may be empty if decode was partial).
    pub fn as_purchase(&self) -> PurchasePayload {
        PurchasePayload {
            payment_id: self
                .value
                .get("payment_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            event_id: self
                .value
                .get("event_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            buyer: self.value.get("buyer").and_then(Value::as_str).map(str::to_string),
            owner: self.value.get("owner").and_then(Value::as_str).map(str::to_string),
            amount: self.value.get("amount").and_then(Value::as_i64).map(|v| v as i128),
        }
    }

    /// Extract the typed refund payload.
    pub fn as_refund(&self) -> RefundPayload {
        RefundPayload {
            payment_id: match self
                .value
                .get("payment_id")
                .and_then(Value::as_str)
            {
                Some(v) => v.to_string(),
                None => self
                    .value
                    .get("stellar_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
            event_id: self
                .value
                .get("event_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        }
    }

    /// Extract the typed transfer payload.
    pub fn as_transfer(&self) -> TransferPayload {
        TransferPayload {
            payment_id: self
                .value
                .get("payment_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            event_id: self
                .value
                .get("event_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            from: self.value.get("from").and_then(Value::as_str).map(str::to_string),
            to: self.value.get("to").and_then(Value::as_str).map(str::to_string),
        }
    }

    /// Extract the typed register payload.
    pub fn as_register(&self) -> RegisterPayload {
        RegisterPayload {
            event_id: self
                .value
                .get("event_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            organizer: self
                .value
                .get("organizer")
                .and_then(Value::as_str)
                .map(str::to_string),
        }
    }
}

/// Decode the first topic element (a base64-XDR `ScVal`) and return its
/// symbolic name, e.g. `"PaymentProcessed"`.
///
/// Handles both a bare `ScVal::Symbol` (how `[contracttype]` enums serialize)
/// and a symbol wrapped inside `ScVal::Vec` (defensive).
pub fn decode_topic_symbol(topic: Option<&String>) -> String {
    let Some(raw) = topic else {
        return String::new();
    };

    let scval = match ScVal::from_xdr_base64(raw, Limits::none()) {
        Ok(v) => v,
        Err(_) => {
            // Some RPC adapters return the topic as a plain symbol string.
            return raw.clone();
        }
    };

    match scval {
        ScVal::Symbol(s) => String::from_utf8_lossy(&s.0).into_owned(),
        ScVal::Vec(Some(v)) => v
            .0
            .iter()
            .find_map(|item| match item {
                ScVal::Symbol(s) => Some(String::from_utf8_lossy(&s.0).into_owned()),
                _ => None,
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// Map a decoded symbol onto a typed [`EventKind`].
pub fn event_kind_for_topic(topic: &str) -> EventKind {
    match topic {
        "EventRegistered" | "register_event" | "event_registered" => EventKind::RegisterEvent,
        "PaymentProcessed" | "process_purchase" | "purchase_confirmed" | "ticket_purchased" => {
            EventKind::ProcessPurchase
        }
        "BulkRefundProcessed"
        | "PartialRefundProcessed"
        | "CancellationRefundClaimed"
        | "refund"
        | "ticket_refunded" => EventKind::Refund,
        "TicketTransferred" | "transfer_ticket" => EventKind::TransferTicket,
        "EventStatusUpdated" | "event_status_updated" | "event_cancelled" => {
            EventKind::EventStatusUpdate
        }
        "CollateralStaked" | "collateral_staked" | "CollateralUnstaked" | "collateral_unstaked" => {
            if topic.starts_with("CollateralUn") || topic.starts_with("collateral_un") {
                EventKind::CollateralUnstaked
            } else {
                EventKind::CollateralStaked
            }
        }
        _ => EventKind::Unhandled,
    }
}

/// Normalize an RPC `value` field into a plain JSON object.
///
/// Soroban RPC may return the payload in any of three shapes:
/// * a raw JSON object (some adapters) — passed through unchanged;
/// * `{ "xdr": "<base64>" }` (current RPC) — decoded;
/// * a base64 string (older RPC) — decoded.
///
/// Undecodable XDR degrades to the raw JSON value instead of failing.
pub fn normalize_payload(value: &Value) -> Value {
    let xdr = match value {
        Value::String(s) => Some(s.as_str()),
        Value::Object(map) => map.get("xdr").and_then(Value::as_str),
        _ => None,
    };

    match xdr {
        Some(xdr) => match ScVal::from_xdr_base64(xdr, Limits::none()) {
            Ok(scval) => scval_to_json(&scval),
            Err(_) => value.clone(),
        },
        None => value.clone(),
    }
}

/// Convert an XDR `ScVal` into a JSON value.
///
/// Addresses are converted to Stellar strkeys so downstream code never deals
/// with raw 32-byte keys.
pub fn scval_to_json(scval: &ScVal) -> Value {
    match scval {
        ScVal::Bool(b) => json!(b),
        ScVal::Void => Value::Null,
        ScVal::U32(v) => json!(v),
        ScVal::I32(v) => json!(v),
        ScVal::U64(v) => json!(v),
        ScVal::I64(v) => json!(v),
        ScVal::Timepoint(v) => json!(v.0),
        ScVal::Duration(v) => json!(v.0),
        ScVal::U128(v) => {
            let n = combine_u128(v.hi, v.lo);
            if v.hi == 0 {
                json!(v.lo)
            } else {
                json!(n.to_string())
            }
        }
        ScVal::I128(v) => {
            if v.hi == 0 {
                json!(v.lo)
            } else {
                json!(combine_i128(v.hi, v.lo).to_string())
            }
        }
        ScVal::U256(v) => json!(format!(
            "{:016x}{:016x}{:016x}{:016x}",
            v.hi_hi, v.hi_lo, v.lo_hi, v.lo_lo
        )),
        ScVal::I256(v) => json!(format!(
            "{:016x}{:016x}{:016x}{:016x}",
            v.hi_hi, v.hi_lo, v.lo_hi, v.lo_lo
        )),
        ScVal::Bytes(b) => json!(general_purpose::STANDARD.encode(&b.0)),
        ScVal::String(s) => json!(String::from_utf8_lossy(&s.0)),
        ScVal::Symbol(s) => json!(String::from_utf8_lossy(&s.0)),
        ScVal::Vec(Some(v)) => {
            let items: Vec<Value> = v.0.iter().map(scval_to_json).collect();
            Value::Array(items)
        }
        ScVal::Map(Some(map)) => {
            let mut obj = serde_json::Map::new();
            for ScMapEntry { key, val } in map.0.iter() {
                let key_str = match key {
                    ScVal::Symbol(s) => String::from_utf8_lossy(&s.0).into_owned(),
                    ScVal::String(s) => String::from_utf8_lossy(&s.0).into_owned(),
                    other => format!("{other:?}"),
                };
                obj.insert(key_str, scval_to_json(val));
            }
            Value::Object(obj)
        }
        ScVal::Address(address) => json!(address_to_strkey(address)),
        ScVal::Error(code) => json!({ "scError": format!("{code:?}") }),
        other => json!(format!("{other:?}")),
    }
}

/// Render an `ScAddress` as a Stellar `G...` strkey (accounts) or a `C{hex}`
/// contract id.
fn address_to_strkey(address: &ScAddress) -> String {
    match address {
        ScAddress::Account(account_id) => match &account_id.0 {
            PublicKey::PublicKeyTypeEd25519(bytes) => {
                let strkey = stellar_strkey::Strkey::PublicKeyEd25519(
                    stellar_strkey::ed25519::PublicKey(bytes.0),
                );
                strkey.to_string().as_str().to_owned()
            }
        },
        ScAddress::Contract(contract_id) => format!("C{}", hex::encode(contract_id.0.clone())),
        other => format!("{other:?}"),
    }
}

/// Combine an XDR `Int128Parts` (`hi: i64`, `lo: u64`) into a Rust `i128`.
fn combine_i128(hi: i64, lo: u64) -> i128 {
    ((hi as i128) << 64) | (lo as i128)
}

/// Combine an XDR `UInt128Parts` (`hi: u64`, `lo: u64`) into a Rust `u128`.
fn combine_u128(hi: u64, lo: u64) -> u128 {
    ((hi as u128) << 64) | (lo as u128)
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::curr::{Limits, ScMap, ScString, ScSymbol, ScVec, WriteXdr};

    fn sym(name: &str) -> ScSymbol {
        ScSymbol(name.parse().unwrap())
    }

    fn str(s: &str) -> ScString {
        ScString(s.parse().unwrap())
    }

    fn vec(items: Vec<ScVal>) -> ScVec {
        ScVec(items.try_into().unwrap())
    }

    fn map(entries: Vec<ScMapEntry>) -> ScMap {
        ScMap(entries.try_into().unwrap())
    }

    fn xdr_b64(scval: &ScVal) -> String {
        scval.to_xdr_base64(Limits::none()).expect("encode xdr")
    }

    #[test]
    fn decodes_symbol_topic() {
        let topic = xdr_b64(&ScVal::Symbol(sym("PaymentProcessed")));
        assert_eq!(decode_topic_symbol(Some(&topic)), "PaymentProcessed");
        assert_eq!(
            event_kind_for_topic(&decode_topic_symbol(Some(&topic))),
            EventKind::ProcessPurchase
        );
    }

    #[test]
    fn decodes_symbol_wrapped_in_vec_topic() {
        let topic = xdr_b64(&ScVal::Vec(Some(vec(vec![ScVal::Symbol(sym(
            "EventRegistered",
        ))]))));
        assert_eq!(decode_topic_symbol(Some(&topic)), "EventRegistered");
        assert_eq!(
            event_kind_for_topic(&decode_topic_symbol(Some(&topic))),
            EventKind::RegisterEvent
        );
    }

    #[test]
    fn falls_back_to_raw_symbol_when_not_xdr() {
        assert_eq!(
            decode_topic_symbol(Some(&"ticket_purchased".to_string())),
            "ticket_purchased"
        );
        assert_eq!(
            event_kind_for_topic("ticket_purchased"),
            EventKind::ProcessPurchase
        );
    }

    #[test]
    fn unknown_topics_are_unhandled_not_fatal() {
        assert_eq!(
            event_kind_for_topic("TotallyNewEvent"),
            EventKind::Unhandled
        );
    }

    #[test]
    fn scval_primitive_to_json() {
        assert_eq!(scval_to_json(&ScVal::Bool(true)), json!(true));
        assert_eq!(scval_to_json(&ScVal::U32(7)), json!(7));
        assert_eq!(scval_to_json(&ScVal::I64(-3)), json!(-3));
        assert_eq!(scval_to_json(&ScVal::U64(1_000_000)), json!(1_000_000));
        assert_eq!(scval_to_json(&ScVal::String(str("abc"))), json!("abc"));
        assert_eq!(scval_to_json(&ScVal::Symbol(sym("PAY"))), json!("PAY"));
    }

    #[test]
    fn scval_map_to_json_object() {
        let map = ScVal::Map(Some(map(vec![
            ScMapEntry {
                key: ScVal::Symbol(sym("payment_id")),
                val: ScVal::String(str("pay-1")),
            },
            ScMapEntry {
                key: ScVal::Symbol(sym("amount")),
                val: ScVal::I64(12_500_000),
            },
        ])));

        let json = scval_to_json(&map);
        assert_eq!(json["payment_id"], "pay-1");
        assert_eq!(json["amount"], 12_500_000);
    }

    #[test]
    fn normalize_payload_decodes_xdr_object() {
        let map = ScVal::Map(Some(map(vec![ScMapEntry {
            key: ScVal::Symbol(sym("event_id")),
            val: ScVal::String(str("evt-42")),
        }])));
        let xdr = xdr_b64(&map);
        let wrapped = json!({ "xdr": xdr });
        assert_eq!(normalize_payload(&wrapped)["event_id"], "evt-42");
    }

    #[test]
    fn normalize_payload_decodes_raw_xdr_string() {
        let map = ScVal::Map(Some(map(vec![ScMapEntry {
            key: ScVal::Symbol(sym("ok")),
            val: ScVal::Bool(true),
        }])));
        let xdr = xdr_b64(&map);
        assert_eq!(normalize_payload(&Value::String(xdr))["ok"], true);
    }

    #[test]
    fn normalize_payload_passes_plain_json_through() {
        let raw = json!({ "event_id": "abc", "buyer": "G1" });
        assert_eq!(normalize_payload(&raw), raw);
    }

    #[test]
    fn index_event_typed_accessors() {
        let payload = json!({
            "payment_id": "pay-9",
            "event_id": "evt-1",
            "buyer": "GABC",
            "owner": "GDEF",
            "amount": 5000000
        });
        let topic = xdr_b64(&ScVal::Symbol(sym("PaymentProcessed")));
        let ev = IndexedEvent::decode(
            "evt-id-1".to_string(),
            123,
            "cpay".to_string(),
            &[topic],
            &payload,
            125,
        );

        assert_eq!(ev.kind, EventKind::ProcessPurchase);
        let purchase = ev.as_purchase();
        assert_eq!(purchase.payment_id, "pay-9");
        assert_eq!(purchase.event_id, "evt-1");
        assert_eq!(purchase.buyer.as_deref(), Some("GABC"));
        assert_eq!(purchase.amount, Some(5_000_000));
    }

    #[test]
    fn refund_topic_maps_to_refund_kind() {
        for name in ["BulkRefundProcessed", "PartialRefundProcessed", "refund", "ticket_refunded"]
        {
            assert_eq!(event_kind_for_topic(name), EventKind::Refund, "{name}");
        }
    }

    #[test]
    fn transfer_topic_maps_to_transfer_kind() {
        assert_eq!(
            event_kind_for_topic("TicketTransferred"),
            EventKind::TransferTicket
        );
        assert_eq!(
            event_kind_for_topic("transfer_ticket"),
            EventKind::TransferTicket
        );
    }
}