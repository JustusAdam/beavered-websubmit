#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"

//run {} for Flows

// I'm using an explicit `next and `into_iter here, but this could be done via
// labels as well to make it cleaner.
pred flows_to_noskip[ctrls: Ctrl, o: one Type + Src, f : (CallArgument + CallSite), flow_set: set Ctrl->Src->CallArgument] {
    some c: ctrls |
    let a = to_source[c, o] |
    let safe_functions = labeled_objects[Function, into_iter, labels] |
	let next_functions = labeled_objects[Function, next, labels] |
    let safe_arg_call_sites = arg_call_site & Sink->(function.safe_functions + { n : function.next_functions | n->n in c.ctrl_flow}) |
    let rel = ^((c.flow_set + safe_arg_call_sites)) | {
        some c.flow_set[a] // a exists in cs
        and (a -> f in rel)
    }
}

    //arg_call_site & Sink->(function.`into_iter + { n : function.`next | n->n in forget_user.ctrl_flow})

// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter[flow_set: set Ctrl->Src->CallArgument] {
    some cleanup : Ctrl |
    some auth: labeled_objects[cleanup.types[Src], auth_witness, labels] |
    all t: labeled_objects[Type, sensitive, labels] |
        (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels] | flows_to[ctrl, t, store, flow_set]) 
        implies
        (some f: labeled_objects[CallArgument, deletes, labels], ot : t + t.otype | 
         let src = to_source[cleanup, ot] | {
            unconditional[cleanup, src]
            flows_to[cleanup, auth, src, flow_set]
            flows_to_noskip[cleanup, ot, f, flow_set]
         })
}

test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter[flow]
    } for Flows is theorem

}