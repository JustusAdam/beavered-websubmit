#lang forge

open "analysis_result.frg"
open "basic_helpers.frg"
open "framework_helpers.frg"

// Calls to store a value also are influenced by the authenticated user 
// and thus likely make it possible to associate the stored value with 
// the user.

pred stores_to_authorized[flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] {
    all c: Ctrl, a : labeled_objects[Type, sensitive, labels_set], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores, labels_set] | flows_to[c, a, r, flow_set]) 
        implies authorized[recipients[f, c, flow_set, labels_set], c, labels_set]
}

pred only_send_to_allowed_sources[flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] {
    all c: Ctrl, o : Object | 
        all scope : labeled_objects_with_types[c, Object, scopes, labels_set] |
            flows_to[c, o, scope, flow_set]
            implies {
                (some o & labeled_objects_with_types[c, Object, safe_source, labels_set]) // either it is safe itself
                or always_happens_before[c, o, labeled_objects_with_types[c, Object, safe_source, labels_set], scope, flow_set] // obj must go through something in safe before scope
                or (some safe : labeled_objects_with_types[c, Object, safe_source, labels_set] |
                    flows_to[c, safe, o, flow_set]) // safe must have flowed to obj at some point
            }
}

// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter[flow_set: set Ctrl->Src->CallArgument, labels_set: set Object->Label] {
    some c: Ctrl |
    all t: Type |
        sensitive in t.labels_set and (some f: labeled_objects[CallArgument, sink, labels_set] | flows_to[Ctrl, t, f, flow_set])
        implies (some f: labeled_objects[CallArgument, deletes, labels_set], ot: t.otype + t | flows_to[c, ot, f, flow_set] )
}


// pred find_erroneous_one_deleter_int[ef: ErroneousFlow] {
//     (not one_deleter[flow, labels])
//     (one_deleter[(flow - ef.minimal_subflow), labels]) 
// }

// pred find_erroneous_one_deleter {
//     some ef: ErroneousFlow {
//         find_erroneous_one_deleter_int[ef]
//     }
// }

pred find_incomplete_one_deleter_int[if: IncompleteFlow] {
    (not one_deleter[flow, labels])
    (one_deleter[flow, (labels + if.missing_labels)]) 
    all o: Object, fp: FormalParameter {
        (o->fp in if.missing_callsites) => {
            (some o.(if.missing_labels))
            ((o.(if.missing_labels)) in fp.labels)
        }
    }
}

pred find_incomplete_one_deleter {
    some if: IncompleteFlow {
        find_incomplete_one_deleter_int[if]
    }
}

run {
    find_incomplete_one_deleter
} for 1 IncompleteFlow for Flows



test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter[flow, labels]
    } for Flows is theorem

    // Storage properties
    stores_are_safe: {
        stores_to_authorized[flow, labels]
    } for Flows is theorem


    // Happens-before properties
    only_send_to_allowed: {
        only_send_to_allowed_sources[flow, labels]
    } for Flows is theorem
}