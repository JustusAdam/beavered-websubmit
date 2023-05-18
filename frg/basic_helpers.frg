fun to_source[o: one Type + Src] : Src {
    {src : Src |
        o in Type and src->o in types or o = src
    }
}

fun to_sink[o: one Type + Src] : Sink {
    arg_call_site.(to_source[o])
}

// This predicate needs work.  Right now it just asserts
// that this call site is not influenced by control flow, but it should actually
// ensure that function for
// cs is called in every control flow path through c.
pred unconditional[cs: one CallSite] {
    no ctrl_flow.cs
}

fun flow_for_ctrl[ctrl: one Ctrl, flow_set: set Src->CallArgument]: set Src->CallArgument {
    (ctrl.calls + fp_fun_rel.ctrl)->Sink & flow_set
}

fun ctrl_flow_for_ctrl[ctrl: one Ctrl, flow_set: set CallSite->CallSite]: set CallSite->CallSite {
    (ctrl.calls->CallSite) & flow_set
}

pred flows_to[cs: Ctrl, o: one Type + Src, f : (CallArgument + CallSite), flow_set: Src->CallArgument] {
    some c: cs |
    let c_flow = flow_for_ctrl[c, flow_set] |
    let a = to_source[c, o] | {
        some c_flow[a] // a exists in cs
        and (a -> f in ^(c_flow + arg_call_site))
    }
}

pred flows_to_ctrl[cs: Ctrl, o: Object, f : (CallArgument + CallSite), flow_set: set Src->CallArgument] {
    some c: cs |
    let c_flow = flow_for_ctrl[c, flow_set] |
    let c_ctrl_flow = ctrl_flow_for_ctrl[c, flow_set] |
    let total_flow = ^(c_flow + c_ctrl_flow + arg_call_site) |
    some a : Src | {
        o = a or o in Type and a->o in c.types
        ((a -> f in total_flow)
		or
		(some f.arg_call_site and (a -> f.arg_call_site in total_flow)))
    }
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
        some c: cs | 
        let c_flow = flow_for_ctrl[c, flow_set] |
        some a: Object | {
            o = a or o in Type and a->o in c.types
            a -> next in ^(c_flow + arg_call_site - 
                (first->CallSite + CallArgument->first))
        }
    )
}

fun arguments[f : CallSite] : set CallArgument {
    arg_call_site.f
}


