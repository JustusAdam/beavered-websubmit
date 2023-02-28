#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"
// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter[flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] {
    some cleanup : Ctrl |
    all t: labeled_objects[Type, sensitive, labels_set] |
        (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels_set] | flows_to[ctrl, t, store, flow_set]) 
        implies
        (some f: labeled_objects[CallArgument, deletes, labels_set], ot : t + t.otype | 
            flows_to[cleanup, ot, f, flow_set])
}

pred find_erroneous_one_deleter_int[ef: ErroneousFlow] {
    some c : Ctrl | {
    (not one_deleter[flow, labels])
    (one_deleter[(flow - (c->ef.minimal_subflow)), labels]) }
}

pred find_erroneous_one_deleter {
    some ef: ErroneousFlow {
        find_erroneous_one_deleter_int[ef]
    }
}

pred find_incomplete_one_deleter_int[il: IncompleteLabel] {
    (not one_deleter[flow, labels])
    (one_deleter[flow, (labels + il.missing_labels)]) 
}

pred find_incomplete_one_deleter {
    some il: IncompleteLabel {
        find_incomplete_one_deleter_int[il]
    }
}

pred valid_additive_repair[ar : AdditiveRepair] {
    some c : ar.extracallsites {
        c.function in Function
        all ca : arg_call_site.c {

        }
    }   
}

pred one_deleter_additive_repair[ar : AdditiveRepair] {
    some c : Ctrl | {
        (not one_deleter[flow, labels])
        (one_deleter[ar.new_flow, new_labels])
    }
}

run {
    find_incomplete_one_deleter
} for 1 IncompleteLabel, 0 ErroneousFlow for Flows


// run {
//     find_erroneous_one_deleter
// } for 1 ErroneousFlow, 0 IncompleteLabel for Flows

test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter[flow, labels]
    } for Flows is theorem

}

