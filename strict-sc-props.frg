#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"

fun all_recipients[f: CallSite, ctrl: Ctrl, flow_set: set Ctrl->Src->CallArgument] : set Src {
    ^(ctrl.flow_set + arg_call_site).(all_scopes[f, ctrl])
}

fun all_scopes[f: CallSite, c: Ctrl, flow_set: set Ctrl->Src->CallArgument] : set Object {
    let call_site = f |
	let direct = labeled_objects[arguments[call_site], scopes, labels] |
    {some direct => direct
    else {f = Return =>
        (c.types).(labeled_objects[Type, safe_source, labels])
        else
        { scope : labeled_objects[Object, scopes, labels] |
            flows_to[c, scope, call_site, flow_set]
        }
    }
    }
}
pred some_authorized[principal: Src, c: Ctrl] {
    some principal & c.types.(labeled_objects[Type, auth_witness, labels])
}


pred stores_to_authorized[flow_set: set Ctrl->Src->CallArgument] {
    all c: Ctrl, a : labeled_objects[FormalParameter + Type, sensitive, labels], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores, labels] | flows_to[c, a, r, flow_set]) 
        implies some_authorized[all_recipients[f, c], c]
}


test expect {
    // Storage properties
    stores_are_safe: {
        stores_to_authorized[flow]
    } for Flows is theorem
}