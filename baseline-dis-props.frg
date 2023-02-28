#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"

pred only_send_to_allowed_sources[flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] {
	all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels_set], f : labeled_objects[Sink, sink, labels_set] | 
        (flows_to[c, a, f, flow_set]) 
        implies {
			(some all_scopes[f, c, labels_set]) and 
			(all o: Object, scope: all_scopes[f, c, labels_set] | 
			flows_to_ctrl[c, o, scope, flow_set]
            implies {
                (some o & safe_sources[c, flow_set, labels_set]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c, flow_set, labels_set], scope, flow_set] // obj must go through something in safe before scope
                or (some safe : safe_sources[c, flow_set, labels_set] |
                    flows_to_ctrl[c, safe, o, flow_set]) // safe must have flowed to obj at some point
            })
		}
}


pred find_erroneous_only_send_allowed_int[ef: ErroneousFlow] {
    some c : Ctrl | {
    (not only_send_to_allowed_sources[flow, labels])
    (only_send_to_allowed_sources[(flow - (c->ef.minimal_subflow)), labels]) }
}

pred find_erroneous_only_send_allowed {
    some ef: ErroneousFlow {
        find_erroneous_only_send_allowed_int[ef]
    }
}

pred find_incomplete_only_send_allowed_int[il: IncompleteLabel] {
    (not only_send_to_allowed_sources[flow, labels])
    (only_send_to_allowed_sources[flow, (labels + il.missing_labels)]) 
}

pred find_incomplete_only_send_allowed {
    some il: IncompleteLabel {
        find_incomplete_only_send_allowed_int[il]
    }
}

run {
    find_incomplete_only_send_allowed
} for 1 IncompleteLabel, 0 ErroneousFlow for Flows


// run {
//     find_erroneous_only_send_allowed
// } for 1 ErroneousFlow, 0 IncompleteLabel for Flows


test expect {
    // Happens-before properties
    only_send_to_allowed: {
        only_send_to_allowed_sources[flow, labels]
    } for Flows is theorem
}
