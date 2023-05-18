

fun all_recipients[f: CallSite, ctrl: Ctrl, flow_set: set Src->Sink, labels_set: set Object->Label] : set Src {
    ^(ctrl.flow_set + arg_call_site).(all_scopes[f, ctrl, flow_set, labels_set])
}

fun sources_in[c: Ctrl] {
    fp_fun_rel.c + c.calls
}

fun all_scopes[f: CallSite, c: Ctrl, flow_set: set Src->Sink, labels_set: set Object->Label] : set Object {
    let call_site = f |
	let direct = labeled_objects[arguments[call_site], scopes, labels_set] |
    {some direct => direct
    else {f = Return =>
        types.(labeled_objects[Type, safe_source, labels_set]) & sources_in[c]
        else
        { scope : labeled_objects[Object, scopes, labels_set] |
            flows_to[c, scope, call_site, flow_set]
        }
    }
    }
}
pred some_authorized[principal: Src, labels_set: set Object->Label] {
    some principal & types.(labeled_objects[Type, auth_witness, labels_set])
}


pred stores_to_authorized[flow_set: set Src->Sink, labels_set: set Object->Label] {
    all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels_set], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores, labels_set] | flows_to[c, a, r, flow_set]) 
        implies some_authorized[all_recipients[f, c], labels_set]
}

