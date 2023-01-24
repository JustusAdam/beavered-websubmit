#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"

//run {} for Flows

// I'm using an explicit `next and `into_iter here, but this could be done via
// labels as well to make it cleaner.
pred flows_to_noskip[ctrls: Ctrl, o: one Type + Src, f : (CallArgument + CallSite)] {
    some c: ctrls |
    let a = to_source[c, o] |
    let safe_functions = `into_iter |
    let safe_arg_call_sites = arg_call_site & Sink->(function.safe_functions + { n : function.`next | n->n in c.ctrl_flow}) |
    let rel = ^((c.flow + safe_arg_call_sites)) | {
        some c.flow[a] // a exists in cs
        and (a -> f in rel)
    }
}

    //arg_call_site & Sink->(function.`into_iter + { n : function.`next | n->n in forget_user.ctrl_flow})

// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter {
    some cleanup : Ctrl |
    some auth: labeled_objects[cleanup.types[Src], auth_witness] |
    all t: labeled_objects[Type, sensitive] |
        (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores] | flows_to[ctrl, t, store]) 
        implies
        (some f: labeled_objects[CallArgument, deletes], ot : t + t.otype | 
         let src = to_source[cleanup, ot] | {
            unconditional[cleanup, src]
            flows_to[cleanup, auth, src]
            flows_to_noskip[cleanup, ot, f]
         })
}

test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter
    } for Flows is theorem

}