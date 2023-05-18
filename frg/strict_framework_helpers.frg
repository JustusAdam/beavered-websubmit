// This file defines helper functions that 

fun flow_roots[c: Ctrl, flow_set: set Src->CallArgument] : set Src->Sink {
	let c_flow = flow_for_ctrl[c, flow_set] |
	{ src: Src, sink: Sink | src->sink in ^(c_flow + arg_call_site) and no arg_call_site.src & c_flow[Src] }
}

fun all_recipients[f: CallSite, ctrl: Ctrl, flow_set: set Src->CallArgument, labels_set: set Object->Label] : set Src {
	let c_flow = flow_for_ctrl[c, flow_set] |
    ^(c_flow + arg_call_site).(labeled_objects[arguments[f], scopes, labels_set]) & Src
}

fun all_scopes[f: Sink, labels_set: set Object->Label] : set Object {
	labeled_objects[arguments[f.arg_call_site], scopes, labels_set] + {
		arg : types.(labeled_objects[Type, safe_source, labels_set]) & FormalParameter | {
			some (f & Return)
		}
	}
}
pred some_authorized[principal: Src, labels_set: set Object->Label] {
    some principal & types.(labeled_objects[Type, auth_witness, labels_set])
}


fun safe_sources[cs: Ctrl, flow_set: set Src->CallArgument, labels_set: set Object->Label] : set Object {
	labeled_objects_with_types[cs, Object, safe_source, labels_set] // Either directly labeled with safe_source 
	+ {
		// Or it is safe_source_with_bless and has been flowed to by bless_safe_source
		elem : labeled_objects_with_types[cs, Object, safe_source_with_bless, labels_set] | {
			some bless : labeled_objects_with_types[cs, Object, bless_safe_source, labels_set] | {
				flows_to_ctrl[cs, bless, elem, flow_set]
			}
		}
	}
	+ 
	// The following makes edit-dis-2-a pass as expected but takes >3 min to run and causes edits (1-b, 1-c, 2-b, 2-c) to pass unexpectedly.
	// Or it was flowed to by safe_source_with_bless and has been flowed to by bless_safe_source
	{
		o: CallArgument | {
			some elem : labeled_objects_with_types[cs, Object, safe_source_with_bless, labels_set] | {
				flows_to_ctrl[cs, elem, o, flow_set]
			}
			some bless : labeled_objects_with_types[cs, Object, bless_safe_source, labels_set] | {
				flows_to_ctrl[cs, bless, o, flow_set]
			}
			not o in labeled_objects_with_types[cs, Object, scopes, labels_set]
			not o in labeled_objects_with_types[cs, Object, not_safe_source, labels_set]
		}
	}

}
