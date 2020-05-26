mod msg;
mod querier;
mod query;

pub use msg::{create_swap_msg, TerraMsg, TerraMsgWrapper};
pub use querier::TerraQuerier;
pub use query::{
    ExchangeRateResponse, ExchangeRatesResponse, RewardsWeightResponse,
    SeigniorageProceedsResponse, SwapResponse, TaxCapResponse, TaxProceedsResponse,
    TaxRateResponse, TerraQuery, TerraQueryWrapper, TobinTaxResponse,
};

// This export is added to all contracts that import this package, signifying that they require
// "terra" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_terra() {}
