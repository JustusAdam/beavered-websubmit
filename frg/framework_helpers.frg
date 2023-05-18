#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"

// This file defines helper functions that 

fun flow_roots[c: Ctrl, flow_set: set Ctrl->Src->CallArgument] : set Src->Sink {
	{ src: Src, sink: Sink | src->sink in ^(c.flow_set + arg_call_site) and no arg_call_site.src & c.flow_set[Src] }
}

fun all_recipients[f: CallSite, ctrl: Ctrl, flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] : set Src {
    ^(ctrl.flow_set + arg_call_site).(labeled_objects[arguments[f], scopes, labels_set]) & Src
}

fun all_scopes[f: Sink, c: Ctrl, labels_set: set Object->Label] : set Object {
	labeled_objects[arguments[f.arg_call_site], scopes, labels_set] + {
		arg : c.types.(labeled_objects[Type, safe_source, labels_set]) & FormalParameter | {
			some (f & Return)
		}
	}
}

pred some_authorized[principal: Src, c: Ctrl, labels_set: set Object->Label] {
    some principal & c.types.(labeled_objects[Type, auth_witness, labels_set])
}

fun safe_sources[cs: Ctrl, flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] : set Object {
	labeled_objects_with_types[cs, Object, safe_source, labels_set] // Either directly labeled with safe_source 
	+ {
		// Or it is safe_source_with_bless and has been flowed to by bless_safe_source
		elem : labeled_objects_with_types[cs, Object, safe_source_with_bless, labels_set] | {
			some bless : labeled_objects_with_types[cs, Object, bless_safe_source, labels_set] | {
				flows_to_ctrl[cs, bless, elem, flow_set]
			}
		}
	}
}
