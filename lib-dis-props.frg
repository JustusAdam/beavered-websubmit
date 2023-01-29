#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"

fun all_scopes[f: Sink, c: Ctrl] : set Object {
	labeled_objects[arguments[f.arg_call_site], scopes]
}

fun safe_sources[c: Ctrl] : set Src {
    labels.request_generated + c.types.(labeled_objects[Type, server_state]) + labels.from_storage
}

pred only_send_to_allowed_sources {
	all c: Ctrl, a : labeled_objects[Src + Type, sensitive], f : labeled_objects[Sink, sink] | 
        (flows_to[c, a, f]) 
        implies {
			(some all_scopes[f, c]) and 
			(all o: Object, scope: all_scopes[f, c] | 
			flows_to[c, o, scope]
            implies {
                (some o & safe_sources[c]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c], scope] // obj must go through something in safe before scope
                or (some safe : safe_sources[c] |
                    flows_to[c, safe, o]) // safe must have flowed to obj at some point
            })
		}
}


pred guarded_storage_release {
	all c: Ctrl, a : labeled_objects[Src + Type, from_storage], f : labeled_objects[Sink, sink] | 
        (flows_to[c, a, f]) 
        implies {
			(some all_scopes[f, c]) and 
			{o: Src | some scope: all_scopes[f, c] |
			flows_to[c, o, scope] 
                and no arg_call_site.o & c.flow[Src] // roots of the flow
            } = 
            { o: Src | some src: arguments[to_source[c, a]] |
			    flows_to[c, o, src] 
            }
		}
}

test expect {
    // Happens-before properties
    only_send_to_allowed: {
        only_send_to_allowed_sources
    } for Flows is theorem
}
