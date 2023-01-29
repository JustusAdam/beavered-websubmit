#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"

fun all_scopes[f: Sink, c: Ctrl] : set Object {
    let call_site = f.arg_call_site |
	let direct = labeled_objects[arguments[call_site], scopes] |
    {some direct => direct
    else {f = Return =>
        labeled_objects[fp_fun_rel.c, request_generated]
        else
        { scope : labeled_objects[Object, scopes] |
            flows_to[c, scope, call_site]
        }
    }
    }
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
