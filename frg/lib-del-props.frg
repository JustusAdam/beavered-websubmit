// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred property[flow: set Src->CallArgument, labels: set Object->Label] {
    some cleanup : Ctrl |
        #{v_or_t: labeled_objects[Src + Type, sensitive, labels] |
            (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels], v: v_or_t + types.v_or_t| 
            some src: to_source[ctrl, v] | 
            flows_to[src, store, flow])}
        =
        #{v : labeled_objects[sources_of[cleanup], from_storage, labels] |
            (some f: labeled_objects[CallArgument, deletes, labels] | 
                flows_to[v, f, flow])}
}

//run {} for Flows

expect {
    vacuity_1: {
        some cleanup : Ctrl |
        some v_or_t: labeled_objects[Src + Type, sensitive, labels] |
            (some ctrl: Ctrl, store: labeled_objects[CallArgument, stores, labels], v: v_or_t + types.v_or_t| 
            some src: to_source[ctrl, v] | 
            flows_to[src, store, flow])
    } is sat
    vacuity_2: {
        some cleanup : Ctrl |
        some v : labeled_objects[sources_of[cleanup], from_storage, labels] |
            (some f: labeled_objects[CallArgument, deletes, labels] | 
                flows_to[v, f, flow])
    } is sat
}
