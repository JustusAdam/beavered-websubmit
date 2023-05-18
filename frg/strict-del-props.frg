
// I'm using an explicit `next and `into_iter here, but this could be done via
// labels_set as well to make it cleaner.
pred flows_to_noskip[ctrls: Ctrl, o: one Type + Src, f : (CallArgument + CallSite), flow_set: set Src->Sink, labels_set: set Object->Label] {
    some c: ctrls |
    let c_flow = flow_for_ctrl[c, flow_set] |
    let a = to_source[c, o] |
    let safe_functions = labeled_objects[Function, into_iter, labels_set] |
	let next_functions = labeled_objects[Function, next, labels_set] |
    let safe_arg_call_sites = arg_call_site & Sink->(function.safe_functions + { n : function.next_functions | n->n in c.ctrl_flow}) |
    let rel = ^((c_flow + safe_arg_call_sites)) | {
        some c_flow[a] // a exists in cs
        and (a -> f in rel)
    }
}

    //arg_call_site & Sink->(function.`into_iter + { n : function.`next | n->n in forget_user.ctrl_flow})

// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred property[flow_set: set Src->Sink, labels_set: set Object->Label] {
    some cleanup : Ctrl |
    some auth: labeled_objects[cleanup.types[Src], auth_witness, labels_set] |
    all t: labeled_objects[Type, sensitive, labels_set] |
        (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels_set] | flows_to[ctrl, t, store, flow_set]) 
        implies
        (some f: labeled_objects[CallArgument, deletes, labels_set], ot : t + t.otype | 
         let src = to_source[cleanup, ot] | {
            unconditional[cleanup, src]
            flows_to[cleanup, auth, src, flow_set]
            flows_to_noskip[cleanup, ot, f]
         })
}

