#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "lib_framework_helpers.frg"
// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter[flow_set: set Ctrl->Src->CallArgument] {
    some cleanup : Ctrl |
        #{v_or_t: labeled_objects[Src + Type, sensitive, labels] |
            (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels], v: v_or_t + ctrl.types.v_or_t | flows_to[ctrl, v, store, flow_set])}
        =
        #{v : labeled_objects[Src, from_storage, labels] |
            (some f: labeled_objects[CallArgument, deletes, labels] | 
                flows_to[cleanup, v, f, flow_set])}
}

//run {} for Flows


test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter[flow]
    } for Flows is theorem

}