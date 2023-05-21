
pred only_send_to_allowed_sources[flow: set Src->CallArgument, labels: set Object->Label] {
	all c: Ctrl |
    all a : to_source[c, labeled_objects[FormalParameter + Type, sensitive, labels]], f : labeled_objects[sinks_of[c], sink, labels] | 
        (flows_to[a, f, flow]) 
        implies {
			(all o: sources_of[c], scope: all_scopes[f, c, flow, labels] | 
			flows_to[o, scope, flow]
            implies {
                (some o & safe_sources[c, flow, labels]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c, flow, labels], scope, flow] // obj must go through something in safe before scope. MUST PASS IN ALL SAFE SOURCES HERE!!!
                or (some safe : to_source[c, safe_sources[c, flow, labels]], o_sink : to_sink[c, o] |
                    flows_to_ctrl[safe, o_sink, flow]) // safe must have flowed to obj at some point
            })
		}
}


pred guarded_storage_release[flow: set Src->CallArgument, labels: set Object->Label] {
	all c: Ctrl | 
	all a : to_source[c, labeled_objects[Src + Type, from_storage, labels]], f :
	labeled_objects[sinks_of[c], sink, labels] + Return | 
        (flows_to[a, f, flow]) 
        implies {
			(all o: sources_of[c], scope: all_scopes[f, c, flow, labels] | 
			flows_to[o, scope, flow]
            implies {
                (some o & safe_sources[c, flow, labels]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c, flow, labels], scope, flow] // obj must go through something in safe before scope. MUST PASS IN ALL SAFE SOURCES HERE!!!
                or (some safe : to_source[c, safe_sources[c, flow, labels]], o_sink : to_sink[c, o] |
                    flows_to_ctrl[safe, o_sink, flow]) // safe must have flowed to obj at some point
            })
		}
}

pred property[flow_set: set Src->CallArgument, labels_set: set Object->Label] {
    only_send_to_allowed_sources[flow_set, labels_set]
    guarded_storage_release[flow_set, labels_set]
}