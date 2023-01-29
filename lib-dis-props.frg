#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "lib_framework_helpers.frg"


pred only_send_to_allowed_sources {
	all c: Ctrl, a : labeled_objects[Src + Type, sensitive], f : labeled_objects[Sink, sink] | 
        (flows_to[c, a, f]) 
        implies {
			(some all_scopes[f, c]) and 
			(all o: Object, scope: all_scopes[f, c] | 
			flows_to[c, o, scope]
            implies {
                (some o & safe_sources[c])
                or always_happens_before[c, o, safe_sources[c], scope]
                or (some safe : safe_sources[c] |
                    flows_to[c, safe, o])
            })
		}
}

pred guarded_storage_release {
	all c: Ctrl, a : labeled_objects[Src + Type, from_storage], f :
	labeled_objects[Sink, sink] + Return | 
        (flows_to[c, a, f]) 
        implies {
			(some all_scopes[f, c]) and 
			{o: Src | some scope: all_scopes[f, c] |
			flows_to[c, o, scope] 
                and no arg_call_site.o & c.flow[Src] // roots of the flow
            } in labels.request_generated 
		}
}

test expect {
    // Happens-before properties
    only_send_to_allowed: {
        only_send_to_allowed_sources
    } for Flows is theorem

    guarded_release: {
        guarded_storage_release
    } for Flows is theorem
}
