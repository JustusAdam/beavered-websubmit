// "Minimal" version
pred find_erroneous_my_pred_int[ms: set Src->Sink] {
    some c : Ctrl |
    let c_flow = flow_for_ctrl[c, flow] {
		(ms in c_flow)
        (not property[flow, labels])
        (property[(c_flow - ms), labels]) 
   }
}

pred find_erroneous_my_pred {
    find_erroneous_my_pred_int[minimal_subflow]

    no src: Src, sink: Sink {
                    src->sink in minimal_subflow
        find_erroneous_my_pred_int[minimal_subflow - src->sink]
    }
}
