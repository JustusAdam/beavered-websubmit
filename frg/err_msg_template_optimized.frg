
// "Optimized" version
pred find_erroneous_my_pred_int[ef: set Src->Sink] {
    some c : Ctrl | {
        let c_flow = flow_for_ctrl[c, flow] | {
            (ef in c_flow)
            (not predicate[c_flow])
            (predicate[(c_flow - ef)]) 
        }
    }
}

pred find_erroneous_my_pred {
    find_erroneous_my_pred_int[minimal_subflow]

    not {
        find_erroneous_my_pred_int[minimal_subflow2]
        #(minimal_subflow2) < #(minimal_subflow)
    }
}

