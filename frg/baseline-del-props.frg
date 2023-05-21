

// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred property[flow_set: set CallArgument, labels_set: set Object->Label] {
    some cleanup : Ctrl |
    all t: labeled_objects[Type, sensitive, labels_set] |
        (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels_set] | flows_to[to_source[cleanup, t], store, flow_set]) 
        implies
        (some f: labeled_objects[CallArgument, deletes, labels_set], ot : t + t.otype | 
            flows_to[to_source[cleanup, ot], f, flow_set])
}
