use fuel_core::{config::Config, service::FuelService};
use fuel_gql_client::client::{schema::node_info::NodeInfo, FuelClient};

#[tokio::test]
async fn node_info() {
    let node_config = Config::local_node();
    let srv = FuelService::new_node(node_config.clone()).await.unwrap();
    let client = FuelClient::from(srv.bound_address);

    let NodeInfo {
        utxo_validation,
        predicates,
        vm_backtrace,
        min_byte_price,
        min_gas_price,
        max_depth,
        max_tx,
        ..
    } = client.node_info().await.unwrap();

    assert_eq!(utxo_validation, node_config.utxo_validation);
    assert_eq!(predicates, node_config.predicates);
    assert_eq!(vm_backtrace, node_config.vm.backtrace);
    assert_eq!(min_gas_price, node_config.txpool.min_gas_price.into());
    assert_eq!(min_byte_price, node_config.txpool.min_byte_price.into());
    assert_eq!(max_depth, node_config.txpool.max_depth.into());
    assert_eq!(max_tx, node_config.txpool.max_tx.into());
}
