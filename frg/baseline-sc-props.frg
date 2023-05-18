#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"

// Calls to store a value also are influenced by the authenticated user 
// and thus likely make it possible to associate the stored value with 
// the user.
pred stores_to_authorized[flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] {
    all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels_set], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores, labels_set] | flows_to[c, a, r, flow_set]) 
        implies some_authorized[all_recipients[f, c, flow_set, labels_set], c, labels_set]
}


pred find_erroneous_stores_to_authorized_int[ef: ErroneousFlow] {
    some c : Ctrl | {
    (not stores_to_authorized[flow, labels])
    (stores_to_authorized[(flow - (c->ef.minimal_subflow)), labels]) }
}

pred find_erroneous_stores_to_authorized {
    some ef: ErroneousFlow {
        find_erroneous_stores_to_authorized_int[ef]
    }
}

pred find_incomplete_stores_to_authorized_int[il: IncompleteLabel] {
    (not stores_to_authorized[flow, labels])
    (stores_to_authorized[flow, (labels + il.missing_labels)]) 
}

pred find_incomplete_stores_to_authorized {
    some il: IncompleteLabel {
        find_incomplete_stores_to_authorized_int[il]
    }
}

// run {
//     find_incomplete_stores_to_authorized
// } for 1 IncompleteLabel, 0 ErroneousFlow for Flows


// run {
//     find_erroneous_stores_to_authorized
// } for 1 ErroneousFlow, 0 IncompleteLabel for Flows



test expect {
    // Storage properties
    stores_are_safe: {
        stores_to_authorized[flow, labels]
    } for Flows is theorem
}