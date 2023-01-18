#lang forge

open "framework_helpers.frg"

// Calls to store a value also are influenced by the authenticated user 
// and thus likely make it possible to associate the stored value with 
// the user.
pred stores_to_authorized {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c]
}

pred only_send_to_allowed_sources {
    all c: Ctrl, o : Object | 
        all scope : labeled_objects_with_types[c, Object, scopes] |
            flows_to[c, o, scope]
            implies {
                (some o & labeled_objects_with_types[c, Object, safe_source]) // either it is safe itself
                or always_happens_before[c, o, labeled_objects_with_types[c, Object, safe_source], scope] // obj must go through something in safe before scope
                or (some safe : labeled_objects_with_types[c, Object, safe_source] |
                    flows_to[c, safe, o]) // safe must have flowed to obj at some point
            }
}

// Asserts that there exists one controller which calls a deletion
// function on every value (or an equivalent type) that is ever stored.
pred one_deleter {
    some c:Ctrl |
    all t: Type |
        sensitive in t.labels and (some f: labeled_objects[CallArgument, sink] | flows_to[Ctrl, t, f])
        implies (some f: labeled_objects[CallArgument, deletes], ot: t.otype + t | flows_to[c, ot, f] )
}



test expect {
    // Deletion properties
    data_is_deleted: {
        one_deleter
    } for Flows is theorem

    // Storage properties
    stores_are_safe: {
        stores_to_authorized
    } for Flows is theorem

    // Output properties
    outputs_with_presenters_are_safe: {
        outputs_to_authorized_all
    } for Flows is theorem

    // Happens-before properties
    only_send_to_allowed: {
        only_send_to_allowed_sources
    } for Flows is theorem
}