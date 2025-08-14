use std::time::Duration;

use anyhow::Result;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use tokio::sync::mpsc;
use tokio::{task, time};

use crate::config::MqttConfig;

use super::message_types::{TimeseriesData, TimeseriesSchema};

pub async fn run_mqtt(
    cfg: &MqttConfig,
) -> Result<(
    mpsc::Receiver<TimeseriesSchema>,
    mpsc::Receiver<TimeseriesData>,
)> {
    let (schema_tx, schema_rx) = mpsc::channel::<TimeseriesSchema>(256);
    let (data_tx, data_rx) = mpsc::channel::<TimeseriesData>(1024);

    // MQTT options
    let mut mqttoptions = MqttOptions::new(&cfg.client_id, &cfg.hostname, cfg.port);
    mqttoptions.set_keep_alive(Duration::from_millis(5000));
    mqttoptions.set_credentials(&cfg.user, &cfg.password);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 64);

    client
        .subscribe(&cfg.schema_topic, QoS::ExactlyOnce)
        .await?;
    client.subscribe(&cfg.data_topic, QoS::ExactlyOnce).await?;

    // Drive the event loop in a task
    tokio::spawn(async move {
        while let Ok(event) = eventloop.poll().await {
            if let Err(e) = handle_message(event, &schema_tx, &data_tx).await {
                // You might want to log with tracing here
                eprintln!("mqtt handler error: {e:?}");
            }
        }
        // event loop ended -> broker connection closed; let supervisors restart us
    });

    // Subscribe
    Ok((schema_rx, data_rx))
}

async fn handle_message(
    event: Event,
    schema_tx: &mpsc::Sender<TimeseriesSchema>,
    data_tx: &mpsc::Sender<TimeseriesData>,
) -> Result<()> {
    match event {
        Event::Incoming(Incoming::Publish(p)) => {
            let topic = p.topic.as_str();
            if topic.ends_with("/schema") {
                if let Ok(doc) = serde_json::from_slice::<TimeseriesSchema>(&p.payload) {
                    schema_tx.send(doc).await.ok();
                }
            } else if topic.ends_with("/data") {
                if let Ok(doc) = serde_json::from_slice::<TimeseriesData>(&p.payload) {
                    data_tx.send(doc).await.ok();
                }
            }
        }
        _ => {}
    }
    return Ok(());
}
