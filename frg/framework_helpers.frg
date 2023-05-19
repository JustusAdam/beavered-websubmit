// This file defines helper functions that 

fun flow_roots[c: Ctrl, flow_set: set Src->CallArgument] : set Src->Sink {
	{ src: sources_of[c], sink: Sink | src->sink in ^(flow_set + arg_call_site) and no arg_call_site.src & flow_set[Src] }
}

fun all_recipients[f: CallSite, flow_set: set Src->CallArgument, labels_set: set Object->Label] : set Src {
    ^(flow_set + arg_call_site).(labeled_objects[arguments[f], scopes, labels_set]) & Src
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
	labeled_objects_with_types[Object, safe_source, labels_set] // Either directly labeled with safe_source 
	+ {
		// Or it is safe_source_with_bless and has been flowed to by bless_safe_source
		elem : labeled_objects_with_types[Object, safe_source_with_bless, labels_set] | {
			some bless : labeled_objects_with_types[Object, bless_safe_source, labels_set] | {
				flows_to_ctrl[to_source[cs, bless], elem, flow_set]
			}
		}
	}
}
