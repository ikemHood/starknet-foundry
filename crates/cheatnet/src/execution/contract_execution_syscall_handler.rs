use crate::execution::cheatable_syscall_handler::CheatableSyscallHandler;
use crate::execution::syscall_interceptor::{
    ChainableHintProcessor, ExecuteHintRequest, HintCompilationInterceptor,
    HintExecutionInterceptor, HintProcessorLogicInterceptor, ResourceTrackerInterceptor,
};
use cairo_felt::Felt252;
use cairo_lang_casm::{
    hints::{Hint, StarknetHint},
    operand::ResOperand,
};
use cairo_lang_runner::{
    casm_run::{extract_relocatable, vm_get_range},
    short_string::as_cairo_short_string,
};
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use std::any::Any;
use std::collections::HashMap;

fn extract_input(
    vm: &mut VirtualMachine,
    input_start: &ResOperand,
    input_end: &ResOperand,
) -> Result<Vec<Felt252>, HintError> {
    let input_start = extract_relocatable(vm, input_start)?;
    let input_end = extract_relocatable(vm, input_end)?;
    vm_get_range(vm, input_start, input_end)
        .map_err(|_| HintError::CustomHint("Failed to read input data".into()))
}

pub struct ContractExecutionSyscallHandler<'a, 'b, 'c>
where
    'c: 'b,
    'b: 'a,
{
    pub child: &'a mut CheatableSyscallHandler<'b, 'c>,
}

impl<'a, 'b, 'c> ContractExecutionSyscallHandler<'a, 'b, 'c> {
    pub fn wrap(child: &'b mut CheatableSyscallHandler<'b, 'c>) -> Self {
        Self { child }
    }
}

impl ChainableHintProcessor for ContractExecutionSyscallHandler<'_, '_, '_> {
    fn get_child(&self) -> Option<&dyn HintProcessorLogicInterceptor> {
        Some(self.child)
    }

    fn get_child_mut(&mut self) -> Option<&mut dyn HintProcessorLogicInterceptor> {
        Some(self.child)
    }
}
impl HintProcessorLogicInterceptor for ContractExecutionSyscallHandler<'_, '_, '_> {}
impl HintExecutionInterceptor for ContractExecutionSyscallHandler<'_, '_, '_> {
    fn intercept_execute_hint(
        &mut self,
        execute_hint_request: &mut ExecuteHintRequest,
    ) -> Option<Result<(), HintError>> {
        let maybe_extended_hint = execute_hint_request.hint_data.downcast_ref::<Hint>();

        return if let Some(Hint::Starknet(StarknetHint::Cheatcode {
            selector,
            input_start,
            input_end,
            ..
        })) = maybe_extended_hint
        {
            let selector = &selector.value.to_bytes_be().1;
            let selector = std::str::from_utf8(selector).unwrap();
            let inputs = match extract_input(execute_hint_request.vm, input_start, input_end) {
                Ok(inputs) => inputs,
                Err(err) => return Some(Err(err)),
            };

            Some(match selector {
                "print" => {
                    for value in inputs {
                        if let Some(short_string) = as_cairo_short_string(&value) {
                            println!(
                                "original value: [{value}], converted to a string: [{short_string}]",
                            );
                        } else {
                            println!("original value: [{value}]");
                        }
                    }
                    Ok(())
                }
                _ => Err(HintError::CustomHint(
                    "Only `print` cheatcode is available in contracts.".into(),
                )),
            })
        } else {
            None
        };
    }
}

impl HintCompilationInterceptor for ContractExecutionSyscallHandler<'_, '_, '_> {}

impl HintProcessorLogic for ContractExecutionSyscallHandler<'_, '_, '_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        HintExecutionInterceptor::execute_hint_chain(self, vm, exec_scopes, hint_data, constants)
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        HintCompilationInterceptor::compile_hint_chain(
            self,
            hint_code,
            ap_tracking_data,
            reference_ids,
            references,
        )
    }
}
impl ResourceTrackerInterceptor for ContractExecutionSyscallHandler<'_, '_, '_> {}

impl ResourceTracker for ContractExecutionSyscallHandler<'_, '_, '_> {
    fn consumed(&self) -> bool {
        ResourceTrackerInterceptor::consumed_chain(self)
    }

    fn consume_step(&mut self) {
        ResourceTrackerInterceptor::consume_step_chain(self);
    }

    fn get_n_steps(&self) -> Option<usize> {
        ResourceTrackerInterceptor::get_n_steps_chain(self)
    }

    fn run_resources(&self) -> &RunResources {
        ResourceTrackerInterceptor::run_resources_chain(self)
    }
}
