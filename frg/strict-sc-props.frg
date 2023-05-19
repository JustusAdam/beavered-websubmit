

pred property[flow_set: set Src->Sink, labels_set: set Object->Label] {
    all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels_set], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores, labels_set] | flows_to[to_source[c, a], r, flow_set]) 
        implies some_authorized[all_recipients[f, c, flow_set, labels_set], labels_set]
}

