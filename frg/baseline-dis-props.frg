

pred property[flow: set Src->CallArgument, labels: set Object->Label] {
	all c: Ctrl |
    all a : labeled_objects[FormalParameter + Type, sensitive, labels], f : labeled_objects[sinks_of[c], sink, labels] | 
        (flows_to[to_source[c, a], f, flow]) 
        implies {
			(some all_scopes[f, labels]) and 
			(all o: Src, scope: all_scopes[f, labels] | 
			flows_to[o, scope, flow]
            implies {
                (some o & safe_sources[c, flow, labels]) // either it is safe itself
                or always_happens_before[c, o, safe_sources[c, flow, labels], scope, flow] // obj must go through something in safe before scope. MUST PASS IN ALL SAFE SOURCES HERE!!!
                or (some safe : to_source[c, safe_sources[c, flow, labels]], o_sink : to_sink[c, o] |
                    flows_to_ctrl[safe, o_sink, flow]) // safe must have flowed to obj at some point
            })
		}
}
