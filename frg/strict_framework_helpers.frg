// This file defines helper functions that 

fun flow_roots[c: Ctrl, flow_set: set Src->CallArgument] : set Src->Sink {
	{ src: sources_of[c], sink: Sink | src->sink in ^(flow_set + arg_call_site) and no arg_call_site.src & flow_set[Src] }
}


// fun all_scopes[f: Sink, labels_set: set Object->Label] : set Object {
// 	labeled_objects[arguments[f.arg_call_site], scopes, labels_set] + {
// 		arg : types.(labeled_objects[Type, safe_source, labels_set]) & FormalParameter | {
// 			some (f & Return)
// 		}
// 	}
// }

// fun all_recipients[f: CallSite, flow_set: set Src->CallArgument, labels_set: set Object->Label] : set Src {
//     ^(flow_set + arg_call_site).(all_scopes[f, flow_set, labels_set]) & Src
// }
pred some_authorized[principal: Src, labels_set: set Object->Label] {
    some principal & labeled_objects[Object, auth_witness, labels_set]
}

fun direct_scopes[f: Sink, labels_set: set Object->Label] : set Object {
	labeled_objects[arguments[f.arg_call_site], scopes, labels_set] + {
		arg : types.(labeled_objects[Type, safe_source, labels_set]) & FormalParameter | {
			some (f & Return)
		}
	}
}

fun all_scopes_store[f: CallSite, c: Ctrl, flow_set: set Src->Sink, labels_set: set Object->Label] : set Object {
	let direct = labeled_objects[arguments[f], scopes_store, labels_set] |
    {some direct => direct
    else {f = Return =>
        types.(labeled_objects[Type, safe_source, labels_set]) & sources_of[c]
        else
        { scope : labeled_objects[Object, scopes_store, labels_set] |
            flows_to[to_source[c, scope], f, flow_set]
        }
    }
    }
}
fun all_recipients[f: CallSite, c:Ctrl, flow_set: set Src->CallArgument, labels_set: set Object->Label] : set Src {
    ^(flow_set + arg_call_site).(all_scopes_store[f, c, flow_set, labels_set]) & Src
}

fun safe_sources[cs: Ctrl, flow: set Src->CallArgument, labels: set Object->Label] : set Object {
	labeled_objects_with_types[Object, safe_source, labels] // Either directly labeled with safe_source 
	+ {
		// Or it is safe_source_with_bless and has been flowed to by bless_safe_source
		elem : labeled_objects_with_types[Object, safe_source_with_bless, labels] | {
			some bless : to_source[cs, labeled_objects_with_types[Object, bless_safe_source, labels]] | {
				some elem_sink : to_sink[cs, elem] | 
				flows_to_ctrl[bless, elem_sink, flow]
			}
		}
	}
// 	+ 
// 	// The following makes edit-dis-2-a pass as expected but takes >3 min to run and causes edits (1-b, 1-c, 2-b, 2-c) to pass unexpectedly.
// 	// Or it was flowed to by safe_source_with_bless and has been flowed to by bless_safe_source
// 	{
// 		o: Object | {
// 			some elem : labeled_objects_with_types[Object, safe_source_with_bless, labels_set] | {
// 				flows_to_ctrl[to_source[cs, elem], o, flow_set]
// 			}
// 			some bless : labeled_objects_with_types[Object, bless_safe_source, labels_set] | {
// 				flows_to_ctrl[to_source[cs, bless], o, flow_set]
// 			}
// 			no labeled_objects_with_types[o, scopes, labels_set]
// 			no  labeled_objects_with_types[o, not_safe_source, labels_set]
// 		}
// 	}
}
