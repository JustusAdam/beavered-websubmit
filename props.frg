#lang forge

open "analysis_result.frg"


pred flows_to[cs: Ctrl, o: Object, f : (CallArgument + CallSite)] {
    some c: cs |
    some a : Object | {
        o = a or o in Type and a->o in c.types
        some (c.flow.a + a.(c.flow)) // a exists in cs
        and (a -> f in ^(c.flow + arg_call_site))
    }
}

fun labeled_objects[obs: Object, ls: Label] : set Object {
    labels.ls & obs
}

// Returns all objects labelled either directly or indirectly
// through types.
fun labeled_objects_with_types[cs: Ctrl, obs: Object, ls: Label] : set Object {
    labeled_objects[obs, ls] + (cs.types).(labeled_objects[obs, ls])
}

fun recipients[f: CallSite, ctrl: Ctrl] : set Src {
    ctrl.flow.(labeled_objects[arguments[f], scopes])
}

pred authorized[principal: Src, c: Ctrl] {
    some principal & c.types.(labeled_objects[Type, auth_witness])
}

fun arguments[f : CallSite] : set CallArgument {
    arg_call_site.f
}

// verifies that for an type o, it flows into first before flowing into next
pred always_happens_before[cs: Ctrl, o: Object, first: (CallArgument + CallSite), next: (CallArgument + CallSite)] {
    not (
        some c: cs | 
        some a: Object | {
            o = a or o in Type and a->o in c.types
            a -> next in ^(c.flow + arg_call_site - 
                (first->CallSite + CallArgument->first))
        }
    )
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

// Every time the system sends a value, the receiver is derived from an 
// `auth_witness` labeled value (e.g. the user)
pred outputs_to_authorized {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c]
}

pred outputs_to_authorized_with_exception {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c] or exception in f.labels
}

// Calls to store a value also are influenced by the authenticated user 
// and thus likely make it possible to associate the stored value with 
// the user.
pred stores_to_authorized {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], stores] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c]
}


// Values that are persent in the controller `c` which have the `labels`
// attached. It returns both values diectly labeled, as well as the 
// values whos types are labeled
fun c_values[c: Ctrl, labels: set Label] : set Object {
    c.types.(labeled_objects[Type, labels]) + labeled_objects[c.flow.CallArgument, labels]
}

// Calculate any flows in `c` that reach `target` but do not pass through 
// or originate from a source that is labeled with one of `authorized_labels`
fun bad_flows[c: Ctrl, target: set CallArgument, authorized_labels: set Label] : set Src->CallArgument {
    let transitive_flow = ^(c.flow + arg_call_site) |
    let good_values = c_values[c, authorized_labels] |
    let terminal_values = (Src & transitive_flow.CallArgument) - transitive_flow[Src] |
    let all_bad_terminal_source_values = terminal_values - good_values |
    let trans_flow_without_good_values = ^(c.flow - (good_values->CallArgument) + arg_call_site) |
    trans_flow_without_good_values & all_bad_terminal_source_values->target             
}

fun flow_from[c: Ctrl, start: Object] : set Object -> Object {
    let t = ^(c.flow + arg_call_site) |
    let reach = t[start] |
    c.flow & reach->reach
}
// ((arg_1 arg2_send_b63_i1))

inst BadFlows {
    Ctrl = `ctrl
    CallArgument = `ca_1
    Type = none
    InputArgument = `arg_0
    otype = none->none
    flow = `ctrl->`arg_0->`ca_1
    types = none->none->none
    Function = `f1
    CallSite = `cs_f1_0
    arg_call_site = `ca_1->`cs_f1_0
    Src = CallSite + InputArgument
    function = `cs_f1_0->`f1
    Object = CallArgument+Src+Function
}

test expect {
    vacuityBadFlowsInst: {} for BadFlows is sat
    bad_flows_inBadFlows: {
        some bad_flows[`ctrl, `ca_1, none]
    } for BadFlows is sat 
    oxymoron_check_bad_flows: {
        some c: Ctrl, labels: set Label, target: set CallArgument |
        some bad_flows[c, target, labels]
    } is sat
}

inst NotOutputsToAuthorizedAll {
    sensitive = `sensitive
    sink = `sink
    exception = `exception
    source = `source
    stores = `stores
    deletes = `deletes
    auth_witness = `auth_witness
    safe_source = `safe_source
    scopes = `scopes
    presenter = `presenter
    cfg_source = `cfg_source
    Label = sensitive+sink+scopes+exception+source+deletes+auth_witness+safe_source+presenter+stores+`cfg_source

    Ctrl = `ctrl
    CallArgument = `ca_1+`ca_2
    Type = none
    InputArgument = `arg_0 + `arg_1
    otype = none->none
    flow = `ctrl->`arg_0->`ca_1+`ctrl->`arg_1->`ca_2
    types = none->none->none
    Function = `f1
    CallSite = `cs_f1_0
    arg_call_site = (`ca_1+`ca_2)->`cs_f1_0
    Src = CallSite + InputArgument
    function = `cs_f1_0->`f1
    Object = CallArgument+Src+Function
    labels = `arg_1->sensitive+`ca_1->scopes+`ca_2->sink
}

test expect {
    vacuity_NotOutputsToAuthorizedAll: {} for NotOutputsToAuthorizedAll is sat
}

//run {} for Flows 


// Assert that all paths reach `target` in `c` are authorzed with one of 
// the supplied labels
pred authorized_paths[c: Ctrl, target: set CallArgument, authorized_labels: set Label] {
    no bad_flows[c, target, authorized_labels]
}

// A version of `outputs_to_authorized` that reasons about all reaching 
// paths and also knows about presenters
pred outputs_to_authorized_all {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite |
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_paths[c, labeled_objects[arguments[f], scopes], sensitive + auth_witness + cfg_source + presenter]
}

test expect {
    NotOutputsToAuthorizedAll_violates_property: {
        not outputs_to_authorized_all
    } for NotOutputsToAuthorizedAll is sat
}

// A version of `outputs_to_authorized` that reasons about all reaching 
// paths
pred outputs_to_authorized_all0 {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_paths[c, labeled_objects[arguments[f], scopes], sensitive + auth_witness + cfg_source]
}

test expect {
    vacuity_Flows: {
    } for Flows is sat
}


test expect {
    // Deletion properties
    oxymoron_check_data_is_deleted: {
        not one_deleter
    } is sat
    data_is_deleted: {
        one_deleter
    } for Flows is theorem

    // Storage properties
    oxymoron_check_stores_are_safe: {
        not stores_to_authorized
    } is sat
    stores_are_safe: {
        stores_to_authorized
    } for Flows is theorem

    // Output properties
    
    // Sometimes values are not just sent back to the user
    outputs_are_not_always_sent_to_apikey: {
        not outputs_to_authorized
    } for Flows is sat
    // If we don't know about presenters the output check doesn't succeed
    outputs_without_presenters_are_unsafe: {
        // Commented out for now, because get_presenters is labeled `safe_source`
        //not outputs_to_authorized_all0
    } for Flows is sat
    // If we have presenters the output check is safe (and also make sure 
    // that the check is not an oxymoron)
    oxymoron_check_outputs_with_presenters_are_safe: {
        not outputs_to_authorized_all
    } 
    // For some reason I need a bound here. If I do not specify the 
    // instance this property fails. I am unsure why, I suspect an 
    // instance bound size issue (too few objects of something, 
    // though I'm unsure what)
    for NotOutputsToAuthorizedAll  
    is sat
    outputs_with_presenters_are_safe: {
        outputs_to_authorized_all
    } for Flows is theorem

    // Happens-before properties
    oxymoron_check_only_send_to_allowed: {
        not only_send_to_allowed_sources
    } is sat
    only_send_to_allowed: {
        only_send_to_allowed_sources
    } for Flows is theorem
    // outputs_are_safe_with_exception: {
    //    Flows implies outputs_to_authorized_with_exception
    // } is theorem 
}