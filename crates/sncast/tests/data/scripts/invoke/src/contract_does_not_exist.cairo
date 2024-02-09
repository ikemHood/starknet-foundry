use sncast_std::{invoke, InvokeResult, ScriptCommandError, RPCError, StarknetError, ScriptCommandErrorTrait};
use starknet::{ContractAddress, Felt252TryIntoContractAddress};
use traits::Into;

fn main() {
    let map_contract_address = 0x123.try_into().expect('Invalid contract address value');
    let invoke_result = invoke(map_contract_address, 'put', array![0x10, 0x1], Option::None, Option::None).unwrap_err();
    invoke_result.print();

    assert(
        ScriptCommandError::RPCError(
            RPCError::StarknetError(StarknetError::ContractError)
        ) == invoke_result,
        'ohno'
    )
}

