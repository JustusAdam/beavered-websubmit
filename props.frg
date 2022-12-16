#lang forge

open "analysis_result.frg"


pred flows_to[cs: Ctrl, o: Object, f : (CallArgument + CallSite)] {
    some c: cs |
    some a : Object | {
        o = a or o in Type and a->o in c.types
        some (c.flow.a + a.(c.flow)) // a exists in cs
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

fun recipients[f: CallSite, ctrl: Ctrl] : set Src {
    ctrl.flow.(labeled_objects[arguments[f], scopes])
}

pred authorized[principal: Src, c: Ctrl] {
    some principal & c.types.(labeled_objects[Type, auth_witness])
}

fun arguments[f : CallSite] : set CallArgument {
    arg_call_site.f
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

pred only_send_to_allowed_sources {
    all c: Ctrl, o : Object | 
        all scope : labeled_objects_with_types[c, Object, scopes] |
            flows_to[c, o, scope]
            implies {
                (some o & labeled_objects_with_types[c, Object, safe_source]) // either it is safe itself
                or always_happens_before[c, o, labeled_objects_with_types[c, Object, safe_source], scope] // obj must go through something in safe before scope
                or (some safe : labeled_objects_with_types[c, Object, safe_source] |
                    flows_to[c, safe, o]) // safe must have flowed to obj at some point
            }
}

pred one_deleter {
    some c:Ctrl |
    all t: Type |
        sensitive in t.labels and (some f: labeled_objects[CallArgument, sink] | flows_to[Ctrl, t, f])
        implies (some f: labeled_objects[CallArgument, deletes], ot: t.otype + t | flows_to[c, ot, f] )
}

pred outputs_to_authorized {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c]
}

pred outputs_to_authorized_with_exception {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c] or exception in f.labels
}

pred stores_to_authorized {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c]
}

fun recipients_all[f: CallSite, ctrl: Ctrl] : set Src {
    ctrl.flow.(labeled_objects[arguments[f], scopes])
}

pred authorized_all[principal: Src, c: Ctrl] {
    principal in c.types.(labeled_objects[Type, auth_witness + safe_source + presenter])
}

fun bad_flows[target: CallArgument, c: Ctrl] : set Src->CallArgument {
    let transitive_flow = ^(c.flow + arg_call_site) |
    let good_types = labeled_objects[Type, auth_witness + safe_source + presenter] |
    let good_values = c.types.good_types |
    let sensitive_values = c.types.(labeled_objects[Type, sensitive]) |
    let terminal_values = (Src & transitive_flow.CallArgument) - transitive_flow[Src] |
    let all_bad_terminal_source_values = terminal_values - good_values - sensitive_values |
    let trans_flow_without_good_values = transitive_flow - (good_values->CallArgument) |
    trans_flow_without_good_values & all_bad_terminal_source_values->target
}

pred authorized_paths[target: CallArgument, c: Ctrl] {

}

pred outputs_to_authorized_all {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_paths[labeled_objects[arguments[f], scopes], c]
}


pred authorized_all0[principal: Src, c: Ctrl] {
    principal in c.types.(labeled_objects[Type, auth_witness + safe_source])
}

pred outputs_to_authorized_all0 {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_all0[recipients_all[f, c], c]
}

test expect {
    vacuity_Flows: {
    } for Flows is sat
}

// Somehow this vacuity test passes, but the test for a failing property does
// not. Curiously also when I drop the premise into the evaluator it comes up
// empty.
test expect {
    vacuity_outputs_to_authorized_premise: {
        some c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
            (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
    } for Flows is sat
    new_authorization_fails_without_safe_presenter_source: {
        not outputs_to_authorized_all0
    } for Flows is sat
}
expect {
    new_authorization: {
        outputs_to_authorized_all
    } for Flows is theorem
}

run {} for Flows

// This fails. Unsure why.
test expect {
    there_are_stores: {
        some labeled_objects[CallArgument, sink]
    } for Flows is sat
    vacuity_one_deleter_premise: {
        some c:Ctrl |
        some t: Type |
            sensitive in t.labels and (some f: labeled_objects[CallArgument, sink] | flows_to[Ctrl, t, f])
    } for Flows is sat
}

test expect {
    data_is_deleted: {
        one_deleter
    } for Flows is theorem
    stores_are_safe: {
        stores_to_authorized
    } for Flows is theorem
    outputs_are_not_always_sent_to_apikey: {
        not outputs_to_authorized
    } for Flows is sat
    outputs_without_presenters_are_unsafe: {
        not outputs_to_authorized_all0
    } for Flows is sat
    outputs_with_presenters_are_safe: {
        outputs_to_authorized_all
    } for Flows is theorem
    only_send_to_allowed: {
        only_send_to_allowed_sources
    } for Flows is theorem
    // outputs_are_safe_with_exception: {
    //    Flows implies outputs_to_authorized_with_exception
    // } is theorem 
}