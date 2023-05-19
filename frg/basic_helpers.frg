fun to_source[c: Ctrl, o: one Type + Src] : Src {
    {src : sources_of[c] |
        o in Type and src->o in types or o = src
    }
}

fun to_sink[o: one Type + Src] : Sink {
    arg_call_site.(to_source[o])
}

fun sources_of[c: Ctrl]: set Src {
    c.calls + fp_fun_rel.c
}

fun sinks_of[c: Ctrl]: set Sink {
    arg_call_site.(c.calls) + Return
}

// This predicate needs work.  Right now it just asserts
// that this call site is not influenced by control flow, but it should actually
// ensure that function for
// cs is called in every control flow path through c.
pred unconditional[cs: one CallSite] {
    no ctrl_flow.cs
}


pred flows_to[src: one Src, f : one Sink, flow_set: Src->CallArgument] {
    some flow_set[src] // a exists in cs
    (src -> f in ^(flow_set + arg_call_site))
}

pred flows_to_ctrl[src: one Src, f : one Sink, flow_set: set Src->CallArgument] {
    let total_flow = ^(flow_set + ctrl_flow + arg_call_site) |
    ((src -> f in total_flow)
    or
    (some f.arg_call_site and (src -> f.arg_call_site in total_flow)))
}

fun labeled_objects[obs: Object, ls: Label, labels_set: set Object->Label] : set Object {
    labels_set.ls & obs
}

// Returns all objects labelled either directly or indirectly
// through types.
fun labeled_objects_with_types[obs: Object, ls: Label, labels_set: set Object->Label] : set Object {
    labeled_objects[obs, ls, labels_set] + types.(labeled_objects[obs, ls, labels_set])
}
// verifies that for an type o, it flows into first before flowing into next
pred always_happens_before[cs: Ctrl, o: Object, first: (CallArgument + CallSite), next: (CallArgument + CallSite), flow_set: set Src->CallArgument] {
    not (
        let c_flow = flow_set |
        some a: Object | {
            o = a or o in Type and a->o in types
            a -> next in ^(c_flow + arg_call_site - 
                (first->CallSite + CallArgument->first))
        }
    )
}

fun arguments[f : CallSite] : set CallArgument {
    arg_call_site.f
}


