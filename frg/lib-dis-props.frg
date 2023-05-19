
pred only_send_to_allowed_sources[flow_set: set Src->CallArgument, labels_set: set Object->Label] {
	all c: Ctrl |
        all a : labeled_objects_with_types[Src + Type, sensitive, labels_set], f : labeled_objects[Sink, sink, labels_set] | 
        (flows_to[to_source[c, a], f, flow_set]) 
        implies {
			(all o: sources_of[c], scope: all_scopes[f, c, flow_set, labels_set] | 
			flows_to[o, scope, flow_set]
            implies {
                (some o & safe_sources[c, flow_set, labels_set])
                or always_happens_before[c, o, safe_sources[c, flow_set, labels_set], scope, flow_set]
                or (some safe : safe_sources[c, flow_set, labels_set] |
                    flows_to[to_source[c, safe], o, flow_set])
            })
		}
}


pred guarded_storage_release[flow_set: set Src->CallArgument, labels_set: set Object->Label] {
	all c: Ctrl, a : labeled_objects[Src + Type, from_storage, labels_set], f :
	labeled_objects[Sink, sink, labels_set] + Return | 
        (flows_to[to_source[c, a], f, flow_set]) 
        implies {
			(all o: sources_of[c], scope: all_scopes[f, c, flow_set, labels_set] | 
			flows_to[o, scope, flow_set]
            implies {
                (some o & safe_sources[c, flow_set, labels_set])
                or always_happens_before[c, o, safe_sources[c, flow_set, labels_set], scope, flow_set]
                or (some safe : safe_sources[c, flow_set, labels_set] |
                    flows_to[to_source[c, safe], o, flow_set])
            })
		}
}

pred property[flow_set: set Src->CallArgument, labels_set: set Object->Label] {
    only_send_to_allowed_sources[flow_set, labels_set]
    guarded_storage_release[flow_set, labels_set]
}