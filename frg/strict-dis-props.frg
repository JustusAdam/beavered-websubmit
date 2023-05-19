pred property[flow_set: set Src->Sink, labels_set: set Object->Label] {
	all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels_set]|
    all f : labeled_objects[sinks_of[c], sink, labels_set] | 
        (flows_to[to_source[c, a], f, flow_set]) 
        implies {
			(some all_scopes[f.arg_call_site, c, flow_set, labels_set]) and 
			(all o: sources_of[c] + sinks_of[c], scope: all_scopes[f.arg_call_site, c, flow_set, labels_set] | 
			flows_to_ctrl[o, scope, flow_set]
            implies {
                (some o & safe_sources[c, flow_set, labels_set]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c, flow_set, labels_set], scope, flow_set] // obj must go through something in safe before scope
                or (some safe : safe_sources[c, flow_set, labels_set] |
                    flows_to_ctrl[to_source[c, safe], o, flow_set]) // safe must have flowed to obj at some point
            })
		}
}

