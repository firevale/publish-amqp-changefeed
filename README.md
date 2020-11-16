# publish-amqp-changefeed

#### Consume cockroach changefeed from AMQP, forward to a Restful http endpoint 

## Configuration

Configuration is done through environment variables:

- **AMQP_URL**: e.g. `amqp://rabbitmq//`
- **AMQP_QUEUE**: rabbitmq queue to consume
- **ENDPOINT**: e.g. `http://localhost:3000/amqp_changefeed/:table`, :table will be replace with table name of cockroachdb

## Running from source

#### Install Rust

```shell
curl https://sh.rustup.rs -sSf | sh
```

#### Run

```shell
AMQP_URL="amqp://localhost//" \
AMQP_QUEUE="cockroach_change_feed" \
ENDPOINT="http://localhost:3000/amqp_changefeed/:table" \
RUST_LOG=info \
cargo run
```

## Running as docker container

```shell
docker run --rm -it --net=host \
-e AMQP_URL="amqp://localhost//" \
-e AMQP_QUEUE="cockroach_change_feed" \
-e ENDPOINT="http://localhost:3000/amqp_changefeed/:table" \
-e RUST_LOG=info \
firevale/publish-amqp-changefeed
```