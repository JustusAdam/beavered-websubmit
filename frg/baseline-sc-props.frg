

// Calls to store a value also are influenced by the authenticated user 
// and thus likely make it possible to associate the stored value with 
// the user.
pred property[flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] {
    all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels_set], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores, labels_set] | flows_to[c, a, r, flow_set]) 
        implies some_authorized[all_recipients[f, c, flow_set, labels_set], c, labels_set]
}

