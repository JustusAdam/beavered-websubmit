
// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred property[flow_set: set Src->CallArgument, labels_set: set Object->Label] {
    some cleanup : Ctrl |
        #{v_or_t: labeled_objects[Src + Type, sensitive, labels] |
            (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels], v: v_or_t + types.v_or_t | flows_to[ctrl, v, store, flow])}
        =
        #{v : labeled_objects[Src, from_storage, labels] |
            (some f: labeled_objects[CallArgument, deletes, labels] | 
                flows_to[cleanup, v, f, flow])}
}

//run {} for Flows

