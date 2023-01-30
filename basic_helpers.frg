#lang forge

open "analysis_result.frg"

// sig ErroneousFlow {
//     minimal_subflow: set Ctrl->Src->CallArgument
// }

sig IncompleteFlow {
    missing_labels: set CallArgument->Label,
    missing_callsites: set CallArgument->FormalParameter
}

pred flows_to[cs: Ctrl, o: Object, f : (CallArgument + CallSite), flow_set: set Ctrl->Src->CallArgument] {
    some c: cs |
    some a : Object | {
        o = a or o in Type and a->o in c.types
        some (c.flow_set.a + a.(c.flow_set)) // a exists in cs
        and (a -> f in ^(c.flow_set + arg_call_site))
    }
}

fun labeled_objects[obs: Object, ls: Label, labels_set: set Object->Label] : set Object {
    labels_set.ls & obs
}

// Returns all objects labelled either directly or indirectly
// through types.
fun labeled_objects_with_types[cs: Ctrl, obs: Object, ls: Label, labels_set: set Object->Label] : set Object {
    labeled_objects[obs, ls, labels_set] + (cs.types).(labeled_objects[obs, ls, labels_set])
}

// verifies that for an type o, it flows into first before flowing into next
pred always_happens_before[cs: Ctrl, o: Object, first: (CallArgument + CallSite), next: (CallArgument + CallSite), flow_set: set Ctrl->Src->CallArgument] {
    not (
        some c: cs | 
        some a: Object | {
            o = a or o in Type and a->o in c.types
            a -> next in ^(c.flow_set + arg_call_site - 
                (first->CallSite + CallArgument->first))
        }
    )
}

fun arguments[f : CallSite] : set CallArgument {
    arg_call_site.f
}


