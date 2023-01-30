#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"

// This file defines helper functions that 

fun recipients[f: CallSite, ctrl: Ctrl, flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] : set Src {
    ctrl.flow_set.(labeled_objects[arguments[f], scopes, labels_set])
}

pred authorized[principal: Src, c: Ctrl, labels_set: set Object->Label] {
    some principal & c.types.(labeled_objects[Type, auth_witness, labels_set])
}