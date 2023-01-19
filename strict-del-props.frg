#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"


// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter {
    some cleanup : Ctrl |
    some auth: labeled_objects[cleanup.types[Src], auth_witness] |
    all t: labeled_objects[Type, sensitive] |
        (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores] | flows_to[ctrl, t, store]) 
        implies
        (some f: labeled_objects[CallArgument, deletes], ot : t + t.otype | 
            flows_to[cleanup, auth, to_source[cleanup, ot]] and
            flows_to[cleanup, ot, f])
}

test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter
    } for Flows is theorem

}