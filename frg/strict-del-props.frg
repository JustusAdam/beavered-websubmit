// The below fn does not work because of https://github.com/brownsys/paralegal/issues/114.
pred flows_to_noskip[src: one Src, f : Sink, flow: set Src->Sink, labels: set Object->Label] {
    let safe_functions = labeled_objects[Function, into_iter, labels] |
	let next_functions = labeled_objects[Function, next, labels] |
    let safe_arg_call_sites = arg_call_site & Sink->(function.safe_functions + { n : function.next_functions | n->n in ctrl_flow}) |
    let rel = ^((flow + safe_arg_call_sites)) | {
        some flow[src] // a exists in cs
        and (src -> f in rel)
    }
}

    //arg_call_site & Sink->(function.`into_iter + { n : function.`next | n->n
    //in forget_user.ctrl_flow})
    
pred premise[t: one Type, flow: set Src->Sink, labels: set Object->Label] {
    some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels] | 
    some src: to_source[ctrl, t] |
    flows_to[src, store, flow]
}

pred conclusion[t: Type, auth: one Object, cleanup: one Ctrl, flow: set Src->Sink, labels: set Object->Label] {
    some f: labeled_objects[CallArgument, deletes, labels], ot : t + t.otype | 
    some src : to_source[cleanup, ot] |
    some auth_src : to_source[cleanup, auth]| {
        unconditional[src]
        (flows_to[auth_src, src, flow] or auth_src = src)
        flows_to_noskip[src, f, flow, labels]
    }
}

// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred property[flow: set Src->Sink, labels: set Object->Label] {
    some cleanup : Ctrl |
    some auth: labeled_objects[sources_of[cleanup], auth_witness, labels] |
    all t: labeled_objects[Type, sensitive, labels] |
        premise[t, flow, labels]
        implies
        conclusion[t, auth, cleanup, flow, labels]
}

test expect {
    vacuity: {
        some t: labeled_objects[Type, sensitive, labels] |
            premise[t, flow, labels]
     } for Flows is sat
    // oxymoron: {
    //     some t: labeled_objects[Type, sensitive, labels] |
    //     some c: Ctrl |
    //     some auth: labeled_objects[types[sources_of[c]], auth_witness, labels] |
    //         conclusion[t, auth, c, flow, labels]
    // } is sat
}

