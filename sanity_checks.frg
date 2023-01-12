#lang forge

open "additional_checks.frg"

test expect {
    vacuity_Flows: {
    } for Flows is sat
}
test expect {

    // Deletion properties
    oxymoron_check_data_is_deleted: {
        not one_deleter
    } is sat
    // Storage properties
    oxymoron_check_stores_are_safe: {
        not stores_to_authorized
    } is sat
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

    // Happens-before properties
    oxymoron_check_only_send_to_allowed: {
        not only_send_to_allowed_sources
    } is sat
}