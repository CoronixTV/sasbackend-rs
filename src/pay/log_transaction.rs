use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{logger, user::DBUser, TAX_FACTOR};

use super::process_payment::PaymentRequest;

#[derive(Deserialize, Serialize)]
struct TransactionLog {
    time: String,
    from: String,
    to: String,
    amount: String,
}

pub async fn log_transaction(
    payload: &PaymentRequest,
    sender: DBUser,
    receiver: DBUser,
    bank: DBUser,
) -> Result<(), surrealdb::Error> {
    let time = logger::curr_time();
    let transaction_reciever = TransactionLog {
        time: time.clone(),
        from: sender.id.id.to_string(),
        to: receiver.id.id.to_string(),
        amount: payload.amount.clone(),
    };
    let transaction_reciever = serde_json::to_string(&transaction_reciever).unwrap();
    let transaction_sender = TransactionLog {
        time: time.clone(),
        from: sender.id.id.to_string(),
        to: receiver.id.id.to_string(),
        amount: (Decimal::from_str(&payload.amount).unwrap()
            * Decimal::from_str(TAX_FACTOR).unwrap())
        .to_string(),
    };
    let transaction_sender = serde_json::to_string(&transaction_sender).unwrap();
    let transaction_bank = TransactionLog {
        time,
        from: sender.id.id.to_string(),
        to: receiver.id.id.to_string(),
        amount: (Decimal::from_str(&payload.amount).unwrap()
            * Decimal::from_str(TAX_FACTOR).unwrap()
            - Decimal::from_str(&payload.amount).unwrap())
        .to_string(),
    };
    let transaction_bank = serde_json::to_string(&transaction_bank).unwrap();
    let mut sender_transactions: Vec<String> = sender
        .transactions
        .clone()
        .split("###")
        .map(|s| s.to_string())
        .collect();
    let mut receiver_transactions: Vec<String> = receiver
        .transactions
        .clone()
        .split("###")
        .map(|s| s.to_string())
        .collect();
    let mut bank_transactions: Vec<String> = bank
        .transactions
        .clone()
        .split("###")
        .map(|s| s.to_string())
        .collect();
    sender_transactions.push(transaction_sender.clone());
    receiver_transactions.push(transaction_reciever.clone());
    bank_transactions.push(transaction_bank.clone());
    let sender_transactions: String = sender_transactions.join("###");
    let receiver_transactions: String = receiver_transactions.join("###");
    let bank_transactions: String = bank_transactions.join("###");
    sender
        .update_value("transactions", &sender_transactions)
        .await?;
    receiver
        .update_value("transactions", &receiver_transactions)
        .await?;
    bank.update_value("transactions", &bank_transactions)
        .await?;
    Ok(())
}

