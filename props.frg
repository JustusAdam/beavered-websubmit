#lang forge

open "analysis_result.frg"


pred flows_to[cs: Ctrl, o: Object, f : CallArgument] {
    some c: cs |
    some a : Src | {
        o = a or o in Type and a->o in c.types
        a -> f in ^(c.flow + arg_call_site)
    }
}

fun labeled_objects[obs: Object, ls: Label] : set Object {
    labels.ls & obs
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
pred always_happens_before[cs: Ctrl, o: Object, first: CallArgument, next: CallArgument] {
    not (
        some c: cs | 
        some a: Src | {
            o = a or o in Type and a->o in c.types
            a -> next in ^(c.flow + arg_call_site - first->CallSite)
        }
    )
}

// TODO: This property is not tested.
pred only_send_to_allowed_sources {
    all c: Ctrl, o : Object, scope : labeled_objects[CallArgument, scopes] |
        flows_to[c, o, scope]
        implies (some safe : labeled_objects[CallSite, safe_source] |
            always_happens_before[c, o, safe, scope] or always_happens_before[c, safe, o, scope])
}

pred one_deleter {
    some c:Ctrl |
    all t: Type |
        sensitive in t.labels and (some f: labeled_objects[CallArgument, stores] | flows_to[Ctrl, t, f])
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

pred outputs_to_authorized_all {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_all[recipients_all[f, c], c]
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

// This fails. Unsure why.
test expect {
    vacuity_one_deleter_premise: {
        some c:Ctrl |
        some t: Type |
            sensitive in t.labels and (some f: labeled_objects[CallArgument, stores] | flows_to[Ctrl, t, f])
    } for Flows is sat
}

test expect {
    data_is_deleted: {
        one_deleter
    } for Flows is theorem
    stores_are_safe: {
        stores_to_authorized
    } for Flows is theorem
    outputs_are_safe: {
        not outputs_to_authorized
    } for Flows is sat
    // outputs_are_safe_with_exception: {
    //    Flows implies outputs_to_authorized_with_exception
    // } is theorem 
}