#[cfg(not(target_arch = "wasm32"))]
mod querier;

#[cfg(not(target_arch = "wasm32"))]
mod swap;

#[cfg(not(target_arch = "wasm32"))]
mod treasury;

#[cfg(not(target_arch = "wasm32"))]
pub use querier::{mock_dependencies, TerraMockQuerier};
#[cfg(not(target_arch = "wasm32"))]
pub use swap::SwapQuerier;
#[cfg(not(target_arch = "wasm32"))]
pub use treasury::TreasuryQuerier;
