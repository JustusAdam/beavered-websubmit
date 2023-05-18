// Original version
pred find_erroneous_my_pred_int[ef: set Src->Sink] {
    some c : Ctrl | 
    let c_flow = flow_for_ctrl[c, flow] |
    {
		(ef in c_flow)
        (not property[c_flow, labels])
        (property[(c_flow - ef), labels]) 
    }
}

pred find_erroneous_my_pred {
    find_erroneous_my_pred_int[minimal_subflow]
}