#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"
// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter {
    some c:Ctrl |
    some s: labeled_objects[Src, auth_whitness] |
    all t: labeled_objects[Type, sensitive] |
        (some f: labeled_objects[CallArgument, sink] | flows_to[Ctrl, t, f])
        implies (some f: labeled_objects[CallArgument, deletes], ot: t.otype + t | 
            flows_to[c, s, to_source[ot]] and
            flows_to[c, ot, f] )
}




test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter
    } for Flows is theorem

}