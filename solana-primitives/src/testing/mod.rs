//! Testing utilities and mock implementations

#[cfg(feature = "testing")]
pub mod mock_client;
#[cfg(feature = "testing")]
pub mod test_data;

#[cfg(feature = "testing")]
pub use mock_client::MockRpcClient;
#[cfg(feature = "testing")]
pub use test_data::TestDataGenerator;