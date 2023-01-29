#lang forge
open "analysis_result.frg"
open "basic_helpers.frg"
fun all_scopes[f: Sink, c: Ctrl] : set Object {
    let call_site = f.arg_call_site |
	let direct = labeled_objects[arguments[call_site], scopes] |
    {some direct => direct
    else {f = Return =>
        labeled_objects[fp_fun_rel.c, request_generated]
        else
        { scope : labeled_objects[Object, scopes] |
            flows_to[c, scope, call_site]
        }
    }
    }
}

fun safe_sources[c: Ctrl] : set Src {
    labels.request_generated + c.types.(labeled_objects[Type, server_state]) + labels.from_storage
}

fun all_recipients[f: CallSite, ctrl: Ctrl] : set Src {

    *(ctrl.flow + arg_call_site).(all_scopes[arg_call_site.f, ctrl])
}