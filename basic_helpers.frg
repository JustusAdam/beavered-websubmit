#lang forge

open "analysis_result.frg"

fun to_source[c: one Ctrl, o: one Type + Src] : Src {
    {a : Src |
        o = a or o in Type and a->o in c.types}
}

pred flows_to[cs: Ctrl, o: one Type + Src, f : (CallArgument + CallSite)] {
    some c: cs |
    let a = to_source[c, o] | {
        some c.flow[a] // a exists in cs
        and (a -> f in ^(c.flow + arg_call_site))
    }
}

fun labeled_objects[obs: Object, ls: Label] : set Object {
    labels.ls & obs
}

// Returns all objects labelled either directly or indirectly
// through types.
fun labeled_objects_with_types[cs: Ctrl, obs: Object, ls: Label] : set Object {
    labeled_objects[obs, ls] + (cs.types).(labeled_objects[obs, ls])
}

// verifies that for an type o, it flows into first before flowing into next
pred always_happens_before[cs: Ctrl, o: Object, first: (CallArgument + CallSite), next: (CallArgument + CallSite)] {
    not (
        some c: cs | 
        some a: Object | {
            o = a or o in Type and a->o in c.types
            a -> next in ^(c.flow + arg_call_site - 
                (first->CallSite + CallArgument->first))
        }
    )
}

fun arguments[f : CallSite] : set CallArgument {
    arg_call_site.f
}


