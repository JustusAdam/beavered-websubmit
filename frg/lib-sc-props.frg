pred some_authorized[principal: Src, c: Ctrl, labels_set: set Object->Label] {
    some principal & labeled_objects_inc_fp[request_generated, labels]
}


// Calls to store a value also are influenced by the authenticated user 
// and thus likely make it possible to associate the stored value with 
// the user.
pred property[flow_set: set Src->CallArgument, labels_set: set Object->Label] {
    all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels_set], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores, labels_set] | flows_to[c, a, r, flow_set]) 
        implies some_authorized[all_recipients[f, c, flow, labels_set], c, labels_set]
}

