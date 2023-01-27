#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"

pred only_send_to_allowed_sources {
	all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : labeled_objects[Sink, sink] | 
        (flows_to[c, a, f]) 
        implies {
			(some all_scopes[f, c]) and 
			(all o: Object, scope: all_scopes[f, c] | 
			flows_to_ctrl[c, o, scope]
            implies {
                (some o & safe_sources[c]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c], scope] // obj must go through something in safe before scope
                or (some safe : safe_sources[c] |
                    flows_to_ctrl[c, safe, o]) // safe must have flowed to obj at some point
            })
		}
}

test expect {
    // Happens-before properties
    only_send_to_allowed: {
        only_send_to_allowed_sources
    } for Flows is theorem
}
