#lang forge
open "props.frg"
// Every time the system sends a value, the receiver is derived from an 
// `auth_witness` labeled value (e.g. the user)
pred outputs_to_authorized {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c]
}

// currently unused
pred outputs_to_authorized_with_exception {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized[recipients[f, c], c] or exception in f.labels
}


// Values that are persent in the controller `c` which have the `labels`
// attached. It returns both values diectly labeled, as well as the 
// values whos types are labeled
fun c_values[c: Ctrl, labels: set Label] : set Object {
    c.types.(labeled_objects[Type, labels]) + labeled_objects[c.flow.CallArgument, labels]
}

// Calculate any flows in `c` that reach `target` but do not pass through 
// or originate from a source that is labeled with one of `authorized_labels`
fun unauthorized_paths[c: Ctrl, target: set CallArgument, authorized_labels: set Label] : set Src->CallArgument {
    let transitive_flow = ^(c.flow + arg_call_site) |
    let good_values = c_values[c, authorized_labels] |
    let terminal_values = (Src & transitive_flow.CallArgument) - transitive_flow[Src] |
    let all_bad_terminal_source_values = terminal_values - good_values |
    let trans_flow_without_good_values = ^(c.flow - (good_values->CallArgument) + arg_call_site) |
    trans_flow_without_good_values & all_bad_terminal_source_values->target             
}

// Just a debugging utility. Projects the `flow` relation for `c` 
// to everything reachable from `start`
fun flow_from[c: Ctrl, start: Object] : set Object -> Object {
    let t = ^(c.flow + arg_call_site) |
    let reach = t[start] |
    c.flow & reach->reach
}

// Test instances for the unauthorized_paths function
inst UnauthorizedPathsTestInst {
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
    vacuityUnauthorizedPathsTestInstInst: {} for UnauthorizedPathsTestInst is sat
    unauthorized_paths_inUnauthorizedPathsTestInst: {
        some unauthorized_paths[`ctrl, `ca_1, none]
    } for UnauthorizedPathsTestInst is sat 
    oxymoron_check_unauthorized_paths: {
        some c: Ctrl, labels: set Label, target: set CallArgument |
        some unauthorized_paths[c, target, labels]
    } is sat
}

// Instance for an oxymoron check for outputs_with_presenters_are_safe
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
    no unauthorized_paths[c, target, authorized_labels]
}

// A version of `outputs_to_authorized` that reasons about all reaching 
// paths and also knows about presenters
pred outputs_to_authorized_all {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite |
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_paths[c, labeled_objects[arguments[f], scopes], sensitive + auth_witness + cfg_source + presenter]
}


// A version of `outputs_to_authorized` that reasons about all reaching 
// paths
pred outputs_to_authorized_all0 {
    all c: Ctrl, a : labeled_objects[InputArgument + Type, sensitive], f : CallSite | 
        (some r : labeled_objects[arguments[f], sink] | flows_to[c, a, r]) 
        implies authorized_paths[c, labeled_objects[arguments[f], scopes], sensitive + auth_witness + cfg_source]
}



test expect {
    NotOutputsToAuthorizedAll_violates_property: {
        not outputs_to_authorized_all
    } for NotOutputsToAuthorizedAll is sat

    // Output properties
    outputs_with_presenters_are_safe: {
        outputs_to_authorized_all
    } for Flows is theorem
}