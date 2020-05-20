mod msg;
mod querier;
mod query;

pub use msg::{SwapMsg, TerraMsg};
pub use querier::TerraQuerier;
pub use query::{
    ExchangeRateResponse, ExchangeRatesResponse, OracleQuery, RewardsWeightResponse,
    SeigniorageProceedsResponse, SimulateSwapResponse, SwapQuery, TaxCapResponse,
    TaxProceedsResponse, TaxRateResponse, TerraQuery, TobinTaxResponse, TreasuryQuery,
};

// This export is added to all contracts that import this package, signifying that they require
// "terra" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_terra() {}
