use crate::cheatcodes::spy_events::{Event, NamedEvent, UnnamedEvent};
use crate::state::CheatcodeState;
use blockifier::execution::call_info::{CallInfo, OrderedEvent};
use cairo_felt::Felt252;
use conversions::StarknetConversions;
use starknet_api::core::ContractAddress;

pub fn collect_emitted_events_from_spied_contracts(
    call_info: &CallInfo,
    cheatcode_state: &mut CheatcodeState,
) -> Vec<Event> {
    let mut all_events: Vec<(ContractAddress, &OrderedEvent)> = vec![];
    let mut stack: Vec<(Option<ContractAddress>, &CallInfo)> = vec![(None, call_info)];

    while let Some((parent_address, current_call)) = stack.pop() {
        let code_address = current_call
            .call
            .code_address
            .unwrap_or_else(|| parent_address.unwrap());

        for spy_on in &mut cheatcode_state.spies {
            if spy_on.does_spy(code_address) {
                let mut emitted_events: Vec<(ContractAddress, &OrderedEvent)> = current_call
                    .execution
                    .events
                    .iter()
                    .map(|event| (code_address, event))
                    .collect();
                emitted_events.sort_by(|(_, event1), (_, event2)| event1.order.cmp(&event2.order));
                all_events.extend(emitted_events);
                break;
            }
        }

        stack.extend(
            current_call
                .inner_calls
                .iter()
                .map(|inner_call| (Some(code_address), inner_call))
                .rev(),
        );
    }

    // creates cheatcodes::spy_events::Event from (ContractAddress, blockifier::src::execution::entry_point::OrderedEvent)
    // event name is removed from the keys (it is located under the first index)
    all_events
        .iter()
        .map(|(address, ordered_event)| {
            let keys: Vec<Felt252> = ordered_event
                .event
                .keys
                .iter()
                .map(|key| key.0.to_felt252())
                .collect();

            let data: Vec<Felt252> = ordered_event
                .event
                .data
                .0
                .iter()
                .map(StarknetConversions::to_felt252)
                .collect();

            return if let Some(name) = keys.get(0) {
                Event::Named(NamedEvent {
                    from: *address,
                    name: name.clone(),
                    keys: keys[1..].to_owned(),
                    data,
                })
            } else {
                Event::Unnamed(UnnamedEvent {
                    from: *address,
                    keys,
                    data,
                })
            };
        })
        .collect::<Vec<Event>>()
}
