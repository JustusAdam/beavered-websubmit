
pred premise[t: Type, flow: set Srk->Sink, labels: set Object->Label] {
    some ctrl: Ctrl | 
    some store: labeled_objects[sinks_of[ctrl], stores, labels] | 
    some src: to_source[ctrl, t] | flows_to[src, store, flow]
}

// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred property[flow: set Src->Sink, labels: set Object->Label] {
    some cleanup : Ctrl |
    all t: labeled_objects[Type, sensitive, labels] |
        premise[t, flow, labels]
        implies
        (some f: labeled_objects[sinks_of[cleanup], deletes, labels], ot : t + t.otype | 
         some src: to_source[cleanup, ot] |
            flows_to[src, f, flow])
}

