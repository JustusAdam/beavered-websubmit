
// "Optimized" version
pred find_erroneous_my_pred_int[ef: set Src->Sink] {
        let c_flow = flow | {
            (ef in c_flow)
            (not property[c_flow, labels])
            (property[(c_flow - ef), labels]) 
        }
}

pred find_erroneous_my_pred {
    find_erroneous_my_pred_int[minimal_subflow]

    not {
        find_erroneous_my_pred_int[minimal_subflow2]
        #(minimal_subflow2) < #(minimal_subflow)
    }
}

