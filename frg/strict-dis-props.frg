#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "strict_framework_helpers.frg"

pred only_send_to_allowed_sources {
	all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels], f : labeled_objects[Sink, sink, labels] | 
        (flows_to[c, a, f, flow]) 
        implies {
			(some all_scopes[f, c, labels]) and 
			(all o: Object, scope: all_scopes[f, c, labels] | 
			flows_to_ctrl[c, o, scope, flow]
            implies {
                (some o & safe_sources[c, flow, labels]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c, flow, labels], scope, flow] // obj must go through something in safe before scope
                or (some safe : safe_sources[c, flow, labels] |
                    flows_to_ctrl[c, safe, o, flow]) // safe must have flowed to obj at some point
            })
		}
}

test expect {
    // Happens-before properties
    only_send_to_allowed: {
        only_send_to_allowed_sources
    } for Flows is theorem
}
