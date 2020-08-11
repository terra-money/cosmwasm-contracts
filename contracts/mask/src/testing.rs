use crate::msg::{CustomQuery, CustomQueryWrapper, CustomResponse};

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Binary, Coin, Extern, HumanAddr, StdResult};

/// A drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies_with_custom_querier(
    canonical_length: usize,
    contract_balance: &[Coin],
) -> Extern<MockStorage, MockApi, MockQuerier<CustomQueryWrapper>> {
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    let custom_querier: MockQuerier<CustomQueryWrapper> =
        MockQuerier::new(&[(&contract_addr, contract_balance)])
            .with_custom_handler(|query| Ok(custom_query_execute(&query)));
    Extern {
        storage: MockStorage::default(),
        api: MockApi::new(canonical_length),
        querier: custom_querier,
    }
}

pub fn custom_query_execute(query: &CustomQueryWrapper) -> StdResult<Binary> {
    let msg = match &query.query_data {
        CustomQuery::Ping {} => "pong".to_string(),
        CustomQuery::Capital { text } => text.to_uppercase(),
    };
    to_binary(&CustomResponse { msg })
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{from_binary, Querier};

    #[test]
    fn custom_query_execute_ping() {
        let req = CustomQueryWrapper {
            route: "mask".to_string(),
            query_data: CustomQuery::Ping {},
        };

        let res = custom_query_execute(&req).unwrap();
        let response: CustomResponse = from_binary(&res).unwrap();
        assert_eq!(response.msg, "pong");
    }

    #[test]
    fn custom_query_execute_capitalize() {
        let req = CustomQueryWrapper {
            route: "mask".to_string(),
            query_data: CustomQuery::Capital {
                text: "fOObaR".to_string(),
            },
        };

        let res = custom_query_execute(&req).unwrap();
        let response: CustomResponse = from_binary(&res).unwrap();
        assert_eq!(response.msg, "FOOBAR");
    }

    #[test]
    fn custom_querier() {
        let deps = mock_dependencies_with_custom_querier(20, &[]);
        let req = CustomQueryWrapper {
            route: "mask".to_string(),
            query_data: CustomQuery::Capital {
                text: "food".to_string(),
            },
        }
        .into();

        let response: CustomResponse = deps.querier.custom_query(&req).unwrap();
        assert_eq!(response.msg, "FOOD");
    }
}
