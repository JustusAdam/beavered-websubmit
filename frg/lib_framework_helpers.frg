
fun labeled_objects_inc_fp[lbls : set Label, labels_set: set Object->Label] : set Object {
    labels_set.lbls
}

fun all_store_scopes[f: Sink, c: Ctrl, flow_set: set Src->CallArgument, labels_set: set Object->Label] : set Object {
    let call_site = f.arg_call_site |
	let direct = labeled_objects[arguments[call_site], scopes_store, labels_set] |
    {some direct => direct
    else {f = Return =>
        labeled_objects[fp_fun_rel.c, request_generated, labels_set]
        else
        { scope : labeled_objects[CallArgument, scopes_store, labels_set] |
            flows_to[scope.arg_call_site, f, flow_set]
        } 
    }
    }
}

fun all_scopes[f: Sink, c: Ctrl, flow_set: set Src->CallArgument, labels_set: set Object->Label] : set Object {
    let call_site = f.arg_call_site |
	let direct = labeled_objects[arguments[call_site], scopes, labels_set] |
    {some direct => direct
    else {f = Return =>
        labeled_objects[fp_fun_rel.c, request_generated, labels_set]
        else
        { scope : labeled_objects[CallArgument, scopes, labels_set] |
            flows_to[scope.arg_call_site, f, flow_set]
        } 
    }
    }
}

fun safe_sources[c: Ctrl, flow_set: set Src->CallArgument, labels_set: set Object->Label] : set Src {
    labeled_objects_inc_fp[request_generated, labels_set] // all request_generated
	+ types.(labeled_objects[Type, server_state, labels_set]) // all server_state
	+ labels_set.(from_storage + safe_source) // all from_storage + safe_source
}

fun all_recipients[f: CallSite, ctrl: Ctrl, flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] : set Src {
    *(flow_set + arg_call_site).(all_store_scopes[arg_call_site.f, ctrl, flow_set, labels])
}
