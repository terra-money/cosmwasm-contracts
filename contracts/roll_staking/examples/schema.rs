use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use roll_staking::msg::{
    ConfigResponse, HandleMsg, InitMsg, QueryMsg, RollResponse, StakerResponse,
};
use roll_staking::state::{ConfigState, RollState, StakerState};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ConfigState), &out_dir);
    export_schema(&schema_for!(StakerState), &out_dir);
    export_schema(&schema_for!(RollState), &out_dir);
    export_schema(&schema_for!(StakerResponse), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(RollResponse), &out_dir);
}
