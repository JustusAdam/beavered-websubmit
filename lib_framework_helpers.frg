#lang forge
open "analysis_result.frg"
open "basic_helpers.frg"


fun labeled_objects_inc_fp[c: Ctrl, lbls : set Label] : set Object {
    (fp_ann.lbls).c + labels.lbls
}

fun all_scopes[f: Sink, c: Ctrl] : set Object {
    let call_site = f.arg_call_site |
	let direct = labeled_objects[arguments[call_site], scopes] |
    {some direct => direct
    else {f = Return =>
        (fp_ann.request_generated).c
        else
        { scope : labeled_objects[Object, scopes] |
            flows_to[c, scope, call_site]
        }
    }
    }
}

fun safe_sources[c: Ctrl] : set Src {
    labeled_objects_inc_fp[c,request_generated] + c.types.(labeled_objects[Type, server_state]) + labels.from_storage
}

fun all_recipients[f: CallSite, ctrl: Ctrl] : set Src {

    *(ctrl.flow + arg_call_site).(all_scopes[arg_call_site.f, ctrl])
}