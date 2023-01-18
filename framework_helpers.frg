#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"

// This file defines helper functions that 

fun recipients[f: CallSite, ctrl: Ctrl] : set Src {
    ctrl.flow.(labeled_objects[arguments[f], scopes])
}

pred authorized[principal: Src, c: Ctrl] {
    some principal & c.types.(labeled_objects[Type, auth_witness])
}