pred only_send_to_allowed_sources[flow_set: set Src->Sink, labels_set: set Object->Label] {
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

