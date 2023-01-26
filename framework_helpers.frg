#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"

// This file defines helper functions that 

fun flow_roots[c: Ctrl] : set Src->Sink {
	{ src: Src, sink: Sink | src->sink in ^(c.flow + arg_call_site) and no arg_call_site.src & c.flow[Src] }
}

fun all_recipients[f: CallSite, ctrl: Ctrl] : set Src {
    ^(ctrl.flow + arg_call_site).(labeled_objects[arguments[f], scopes]) & Src
}

pred some_authorized[principal: Src, c: Ctrl] {
    some principal & c.types.(labeled_objects[Type, auth_witness])
}

fun safe_sources[cs: Ctrl] : set Object {
	labeled_objects_with_types[cs, Object, safe_source] // Either directly labeled with safe_source 
	+ {
		// Or it is safe_source_with_bless and has been flowed to by bless_safe_source
		elem : labeled_objects_with_types[cs, Object, safe_source_with_bless] | {
			some bless : labeled_objects_with_types[cs, Object, bless_safe_source] | {
				flows_to_ctrl[cs, bless, elem]
			}
		}
	}
}
