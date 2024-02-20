use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use utxorpc::spec::cardano::{Tx, TxOutput};

use crate::{
    domain::{Domain, UsagePaymentV1},
    driven::event_dispatch::EventWrapper,
};

struct Config {
    u5c_endpoint: String,
    dcu_policy: String,
}

fn is_dcu_payment(tx: &Tx, config: &Config) -> bool {
    tx.outputs
        .iter()
        .flat_map(|txo| txo.assets.iter())
        .any(|x| x.policy_id == &config.dcu_policy)
}

pub async fn run(domain: Arc<Mutex<Domain>>, config: Config) -> Result<()> {
    let mut subscription = { domain.lock().await.event_dispatch.subscribe() };

    while let Ok(EventWrapper(evt, _)) = subscription.recv().await {
        let mut domain = domain.lock().await;
        domain.handle(evt).await?;
    }

    let mut client = utxorpc::ClientBuilder::new()
        .uri(&config.u5c_endpoint)?
        .build::<utxorpc::CardanoSyncClient>()
        .await;

    let mut tip = client.follow_tip(vec![]).await?;

    while let Ok(event) = tip.event().await {
        let txs: Vec<_> = match event {
            utxorpc::TipEvent::Apply(x) => x
                .body
                .unwrap()
                .tx
                .into_iter()
                .filter(|x| is_dcu_payment(x, &config))
                .collect(),
            utxorpc::TipEvent::Undo(_) => todo!(),
            utxorpc::TipEvent::Reset(_) => todo!(),
        };

        for tx in txs {
            domain.lock().await.handle(
                UsagePaymentV1 {
                    entry: tx.hash.into(),
                    epoch: 1,
                    namespace: tx
                        .auxiliary
                        .unwrap()
                        .metadata
                        .iter()
                        .find(|x| x.label == 1791)
                        .unwrap()
                        .value,
                    units: tx.outputs.first().unwrap().coin,
                }
                .into(),
            );
        }
    }

    Ok(())
}
