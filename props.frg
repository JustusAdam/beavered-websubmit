#lang forge

open "analysis_result.frg"


pred flows_to[cs: Ctrl, o: Object, f : CallArgument] {
    some c: cs |
    some a : Src | {
        o = a or o in Type and a->o in c.types
        a -> f in c.flow
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
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : function.send | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_all[recipients_all[f, c], c]
}

pred authorized_all0[principal: Src, c: Ctrl] {
    principal in c.types.(labeled_objects[Type, auth_witness + safe_source])
}

pred outputs_to_authorized_all0 {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : function.send | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_all0[recipients_all[f, c], c]
}

expect {
    vacuity_Flows: {
        Flows
    } is sat
}

// Somehow this vacuity test passes, but the test for a failing property does
// not. Curiously also when I drop the premise into the evaluator it comes up
// empty.
expect {
    vacuity_outputs_to_authorized_premise: {
        Flows
        some c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : function.send | 
            (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
    } is sat
    new_authorization_fails_without_safe_presenter_source: {
        Flows 
        not outputs_to_authorized_all0
    } is sat
}
expect {
    new_authorization: {
        Flows implies outputs_to_authorized_all
    } is theorem
}

expect {
    vacuity_one_deleter_premise: {
        Flows
        some c:Ctrl |
        some t: Type |
            sensitive in t.labels and (some f: labeled_objects[CallArgument, stores] | flows_to[Ctrl, t, f])
    } is sat
}


test expect {
    data_is_deleted: {
        Flows implies one_deleter
    } is theorem
    stores_are_safe: {
        Flows implies stores_to_authorized
    } is theorem
    outputs_are_safe: {
        Flows 
        not outputs_to_authorized
    } is sat
    // outputs_are_safe_with_exception: {
    //    Flows implies outputs_to_authorized_with_exception
    // } is theorem 
}