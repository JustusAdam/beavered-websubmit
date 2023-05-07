#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"

pred only_send_to_allowed_sources[flow_set: set Ctrl->Src->CallArgument] {
	all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels], f : labeled_objects[Sink, sink, labels] | 
        (flows_to[c, a, f, flow_set]) 
        implies {
			(some all_scopes[f, c, labels]) and 
			(all o: Object, scope: all_scopes[f, c, labels] | 
			flows_to_ctrl[c, o, scope, flow_set]
            implies {
                (some o & safe_sources[c, flow_set, labels]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c, flow_set, labels], scope, flow_set] // obj must go through something in safe before scope
                or (some safe : safe_sources[c, flow_set, labels] |
                    flows_to_ctrl[c, safe, o, flow_set]) // safe must have flowed to obj at some point
            })
		}
}

test expect {
    // Happens-before properties
    only_send_to_allowed: {
        only_send_to_allowed_sources[flow]
    } for Flows is theorem
}