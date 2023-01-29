#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "lib_framework_helper.frg"
// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter {
    some cleanup : Ctrl |
        #{v_or_t: labeled_objects[Src + Type, sensitive] |
            (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores], v: v_or_t + ctrl.types.v_or_t | flows_to[ctrl, v, store])}
        =
        #{v : labeled_objects[Src, from_storage] |
            (some f: labeled_objects[CallArgument, deletes] | 
                flows_to[cleanup, v, f])}
}

//run {} for Flows


test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter
    } for Flows is theorem

}