use dotenv::dotenv;
use futures_lite::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use lapin_async_global_executor::*;
use log::info;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> anyhow::Result<()> {
  if std::env::var("RUST_LOG").is_err() {
    std::env::set_var("RUST_LOG", "info");
  }

  dotenv().ok();
  env_logger::builder().init();

  let amqp_url = std::env::var("AMQP_ADDR")
    .unwrap_or_else(|_| "amqp://admin:123456@rabbitmq:5672/".into());
  let queue =
    std::env::var("AMQP_QUEUE").unwrap_or_else(|_| "cockroach_change_feed".into());
  let endpoint = std::env::var("ENDPOINT")
    .unwrap_or_else(|_| "http://localhost:3000/cache/:table".into());

  async_global_executor::block_on(async {
    let conn = Connection::connect(
      &amqp_url,
      ConnectionProperties::default().with_async_global_executor(),
    )
    .await?;

    info!("AMQP CONNECTED");

    let channel = conn.create_channel().await?;

    let mut consumer = channel
      .basic_consume(
        &queue,
        "cockroach_changefeed_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default(),
      )
      .await?;

    info!("will consume");

    while let Some(delivery) = consumer.next().await {
      let (channel, delivery) = delivery.expect("error in consumer");

      let url = endpoint.replace(":table", delivery.routing_key.as_str());

      let res = surf::post(&url)
        .content_type(surf::http::mime::JSON)
        .body(surf::Body::from_bytes(delivery.data))
        .await;

      match res {
        Ok(response) => {
          if response.status() == surf::StatusCode::Ok {
            channel
              .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
              .await?;

            continue;
          }
        }
        _ => (),
      }

      channel
        .basic_reject(delivery.delivery_tag, BasicRejectOptions::default())
        .await?;
    }

    Ok(())
  })
}
