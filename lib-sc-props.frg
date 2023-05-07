#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "lib_framework_helpers.frg"

pred some_authorized[principal: Src, c: Ctrl] {
    some principal & labeled_objects_inc_fp[c, request_generated, labels]
}


// Calls to store a value also are influenced by the authenticated user 
// and thus likely make it possible to associate the stored value with 
// the user.
pred stores_to_authorized[flow_set: set Ctrl->Src->CallArgument] {
    all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores, labels] | flows_to[c, a, r, flow_set]) 
        implies some_authorized[all_recipients[f, c, flow_set, labels], c]
}

test expect {
    // Storage properties
    stores_are_safe: {
        stores_to_authorized[flow]
    } for Flows is theorem
}