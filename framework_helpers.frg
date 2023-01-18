#lang forge

open "basic_helpers.frg"

// This file defines helper functions that 

fun recipients[f: CallSite, ctrl: Ctrl] : set Src {
    ctrl.flow.(labeled_objects[arguments[f], scopes])
}

pred authorized[principal: Src, c: Ctrl] {
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
