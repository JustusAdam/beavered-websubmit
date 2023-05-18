#lang forge
open "analysis_result.frg"
open "basic_helpers.frg"


fun labeled_objects_inc_fp[c: Ctrl, lbls : set Label, labels_set: set Object->Label] : set Object {
    (fp_ann.lbls).c + labels_set.lbls
}

fun all_scopes[f: Sink, c: Ctrl, flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] : set Object {
    let call_site = f.arg_call_site |
	let direct = labeled_objects[arguments[call_site], scopes, labels_set] |
    {some direct => direct
    else {f = Return =>
        (fp_ann.request_generated).c
        else
        { scope : labeled_objects[CallArgument, scopes, labels_set] |
            flows_to[c, scope.arg_call_site, f, flow_set]
        } 
    }
    }
}

fun safe_sources[c: Ctrl, flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] : set Src {
    labeled_objects_inc_fp[c,request_generated, labels_set] // all request_generated
	+ c.types.(labeled_objects[Type, server_state, labels_set]) // all server_state
	+ labels_set.(from_storage + safe_source) // all from_storage + safe_source
}

fun all_recipients[f: CallSite, ctrl: Ctrl, flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] : set Src {

    *(ctrl.flow + arg_call_site).(all_scopes[arg_call_site.f, ctrl, flow, labels])
}